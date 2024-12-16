use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::File;
use std::io::Write;
use dotenv::dotenv;
use std::env;
use std::path::Path;
use chrono::Utc;
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
struct TokenBalance {
    #[serde(rename = "tokenId")]
    token_id: String,
    balance: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct OwnerWithBalance {
    #[serde(rename = "ownerAddress")]
    owner_address: String,
    #[serde(rename = "tokenBalances")]
    token_balances: Vec<TokenBalance>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AlchemyResponse {
    #[serde(rename = "pageKey")]
    page_key: Option<String>,
    owners: Option<Vec<OwnerWithBalance>>,
    result: Option<Vec<String>>,  // Some responses might return just a list of addresses
}

#[derive(Debug, Serialize, Deserialize)]
struct ScanState {
    last_processed_block: u64,
    last_save_time: chrono::DateTime<Utc>,
    total_holders: u64,
    holders: HashSet<String>,
    last_page_key: Option<String>,
}

impl Default for ScanState {
    fn default() -> Self {
        ScanState {
            last_processed_block: 0,
            last_save_time: Utc::now(),
            total_holders: 0,
            holders: HashSet::new(),
            last_page_key: None,
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    
    // Get Alchemy API key from environment variable
    let api_key = env::var("ALCHEMY_API_KEY").expect("ALCHEMY_API_KEY must be set");
    
    // Load existing state or create new one
    let mut state = load_state().unwrap_or_default();
    
    // Initialize HTTP client with longer timeout
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;
    
    let mut page_key = state.last_page_key.clone();
    let contract_address = "0xC36442b4a4522E871399CD717aBDD847Ab11FE88";

    println!("Starting with {} existing holders", state.holders.len());
    println!("Last page key: {:?}", page_key);
    
    let mut page_count = 0;
    
    // Fetch all pages of owners
    loop {
        page_count += 1;
        println!("\nFetching page {}", page_count);
        
        let url = if let Some(key) = &page_key {
            format!(
                "https://opt-mainnet.g.alchemy.com/nft/v3/{}/getOwnersForContract?contractAddress={}&withTokenBalances=true&pageKey={}",
                api_key, contract_address, key
            )
        } else {
            format!(
                "https://opt-mainnet.g.alchemy.com/nft/v3/{}/getOwnersForContract?contractAddress={}&withTokenBalances=true",
                api_key, contract_address
            )
        };

        println!("Requesting URL: {}", url);

        let response = client
            .get(&url)
            .header("accept", "application/json")
            .send()
            .await?;
            
        println!("Response status: {}", response.status());
        
        let response_text = response.text().await?;
        
        // Try to parse as raw JSON first
        let raw_json: Value = match serde_json::from_str(&response_text) {
            Ok(json) => {
                println!("Raw JSON response structure:");
                println!("{}", serde_json::to_string_pretty(&json)?);
                json
            },
            Err(e) => {
                println!("Failed to parse response as JSON: {}", e);
                println!("Raw response: {}", response_text);
                break;
            }
        };

        // Try to get owners from different possible response formats
        let mut new_owners = Vec::new();

        if let Some(owners) = raw_json.get("owners").and_then(|o| o.as_array()) {
            println!("Found {} owners in response", owners.len());
            for owner in owners {
                if let Some(addr) = owner.get("ownerAddress").and_then(|a| a.as_str()) {
                    new_owners.push(addr.to_string());
                }
            }
        } else if let Some(result) = raw_json.get("result").and_then(|r| r.as_array()) {
            println!("Found {} addresses in result", result.len());
            for addr in result {
                if let Some(addr_str) = addr.as_str() {
                    new_owners.push(addr_str.to_string());
                }
            }
        }

        println!("Parsed {} new owners", new_owners.len());

        if new_owners.is_empty() {
            println!("No owners found in response");
            break;
        }

        // Add the new owners to our state
        let initial_count = state.holders.len();
        for owner in &new_owners {
            state.holders.insert(owner.clone());
        }
        let new_count = state.holders.len();
        println!("Added {} new unique owners", new_count - initial_count);
        
        // Update state
        state.total_holders = state.holders.len() as u64;
        state.last_save_time = Utc::now();
        
        println!("Current unique owners count: {}", state.holders.len());
        
        // Try to get the next page key
        page_key = raw_json.get("pageKey")
            .and_then(|k| k.as_str())
            .map(String::from);
            
        // Save the page key in state
        state.last_page_key = page_key.clone();
        
        // Save progress after each page
        save_state(&state)?;
        save_holders_to_file(&state.holders)?;
        
        if page_key.is_none() {
            println!("No more pages to fetch");
            break;
        }
        
        // Add a delay between requests to avoid rate limiting
        println!("Waiting before next request...");
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }

    println!("\nScan complete!");
    println!("Results saved to data/state.json and data/uniswap_v3_holders.txt");
    println!("Total unique holders: {}", state.total_holders);
    println!("Total pages processed: {}", page_count);

    Ok(())
}

fn save_state(state: &ScanState) -> Result<()> {
    std::fs::create_dir_all("data")?;
    let mut state_file = File::create("data/state.json")?;
    serde_json::to_writer_pretty(&mut state_file, &state)?;
    Ok(())
}

fn save_holders_to_file(holders: &HashSet<String>) -> Result<()> {
    std::fs::create_dir_all("data")?;
    let mut holders_file = File::create("data/uniswap_v3_holders.txt")?;
    let mut holders_vec: Vec<_> = holders.iter().collect();
    holders_vec.sort(); // Sort addresses for consistent output
    for holder in holders_vec {
        writeln!(holders_file, "{}", holder)?;
    }
    Ok(())
}

fn load_state() -> Result<ScanState> {
    let state_path = Path::new("data/state.json");
    if state_path.exists() {
        let file = File::open(state_path)?;
        Ok(serde_json::from_reader(file)?)
    } else {
        Ok(ScanState::default())
    }
}
