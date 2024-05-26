use crate::models::RpcResponse;
use crate::models::SolanaSignature;
use crate::sol_transfer::is_sol_transfer;
use crate::sol_transfer::SolTransfer;
use prettytable::{row, Table};
use reqwest::blocking::Client;
use serde_json::Value;
use chrono::{TimeZone, Utc};

/// Fetches and displays the SOL transaction history for a given Solana address.
pub fn print_sol_transfer_history(
    solana_rpc_url: &str,
    solana_address: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!(
        "Print sol history for {} via {}",
        solana_address, solana_rpc_url
    );

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

    let signatures_response = client
        .post(solana_rpc_url)
        .json(&signatures_payload)
        .send()?;

    if !signatures_response.status().is_success() {
        eprintln!(
            "Error: Received HTTP {} for transaction signatures request",
            signatures_response.status()
        );
        if let Ok(error_text) = signatures_response.text() {
            eprintln!("Response: {}", error_text);
        }
        std::process::exit(1);
    }

    let response_text = signatures_response.text()?;
    let signatures_rpc_response: RpcResponse<Vec<SolanaSignature>> =
        serde_json::from_str(&response_text)?;

    let mut table = Table::new();
    table.add_row(row!["Tx ID", "Sender", "Receiver", "Amount", "Timestamp"]);

    for signature in signatures_rpc_response.result {
        let transaction_payload = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "getTransaction",
            "params": [
                signature.signature,
                "jsonParsed"
            ]
        });

        let transaction_response = client
            .post(solana_rpc_url)
            .json(&transaction_payload)
            .send()?;
        let transaction_response_text = transaction_response.text()?;
        let json: Value = serde_json::from_str(&transaction_response_text)?;

        if is_sol_transfer(&json) {
            match SolTransfer::from_json(&json) {
               
                Ok(sol_transfer) => {
                    #[allow(deprecated)]
                    let timestamp = Utc.timestamp(sol_transfer.timestamp as i64, 0);
                    table.add_row(row!(
                        sol_transfer.transaction_id[0..10], 
                        sol_transfer.sender, 
                        sol_transfer.receiver, 
                        sol_transfer.amount as f64 / 1_000_000_000.0, 
                        timestamp.to_string()
                    ));
                },
                Err(e) => {
                    eprintln!("Error parsing SOL transfer: {}", e);
                }
            }
        }
    }

    table.printstd();

    Ok(())
}
