use reqwest::blocking::Client;
use log::debug;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use serde_json;

use crate::models::{SolanaSignature, RpcResponse};

pub fn fetch_transaction_history(solana_rpc_url: &str, solana_address: &str) -> Result<(), Box<dyn Error>> {

    // Create a client
    let client = Client::new();

    // Fetch the transaction signatures
    let signatures_payload = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "getSignaturesForAddress",
        "params": [
            solana_address,
            {
                "limit": 10
            }
        ]
    });

    let signatures_response = client.post(solana_rpc_url)
        .json(&signatures_payload)
        .send()?;

    if !signatures_response.status().is_success() {
        eprintln!("Error: Received HTTP {} for transaction signatures request", signatures_response.status());
        if let Ok(error_text) = signatures_response.text() {
            eprintln!("Response: {}", error_text);
        }
        std::process::exit(1);
    }

    let response_text = signatures_response.text()?;
    let signatures_rpc_response: RpcResponse<Vec<SolanaSignature>> = serde_json::from_str(&response_text)?;

    for signature in signatures_rpc_response.result {
        if let Some(_signature_block_time) = signature.block_time {
            let transaction_payload = serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "method": "getTransaction",
                "params": [
                    signature.signature,
                    "jsonParsed"
                ]
            });

            let transaction_response = client.post(solana_rpc_url)
                .json(&transaction_payload)
                .send()?;

            let transaction_response_text = transaction_response.text()?;

            // Assuming `signature.signature` is the transaction ID
            // Use only the first 10 characters of the signature for the filename
            let file_name = format!("serializations/{}.json", &signature.signature[0..10]);
            let mut file = File::create(&file_name)?;
            file.write_all(transaction_response_text.as_bytes())?;

            debug!("Transaction data written to file: {}", file_name);
            
        } else {
            debug!("No block time available for signature: {}", signature.signature);
        }
    }

   Ok(())
}