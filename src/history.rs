use reqwest::blocking::Client;
use prettytable::{Table, row};
use log::debug;
use chrono::DateTime;
use std::error::Error;

use crate::models::{SolanaSignature, SolanaTransactionDetails, RpcResponse};

pub fn fetch_transaction_history(client: &Client, solana_rpc_url: &str, solana_address: &str) -> Result<(), Box<dyn Error>> {
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
    debug!("{:#?}", response_text);
    let signatures_rpc_response: RpcResponse<Vec<SolanaSignature>> = serde_json::from_str(&response_text)?;

    let mut mint_table = Table::new();
    mint_table.add_row(row!["Mint", "New Account", "Owner", "Amount", "Decimals"]);

    let mut table = Table::new();
    table.add_row(row!["Transaction ID", "Sender", "Recipient", "Timestamp", "Amount", "Status"]);

    for signature in signatures_rpc_response.result {
        if let Some(signature_block_time) = signature.block_time {
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

            let transaction_status = transaction_response.status();
            let transaction_response_text = transaction_response.text()?;
            debug!("{:#?}", transaction_response_text); // Print the transaction response

            if transaction_status.is_success() {
                let transaction_rpc_response: RpcResponse<SolanaTransactionDetails> = serde_json::from_str(&transaction_response_text)?;

                let transaction_details = transaction_rpc_response.result;
                let status = if transaction_details.meta.status.err.is_none() {
                    "Success"
                } else {
                    "Failure"
                };

                let datetime = DateTime::from_timestamp(signature_block_time, 0).expect("Invalid timestamp");
                let formatted_time = datetime.format("%Y-%m-%d %H:%M:%S").to_string();

                let truncated_transaction_id = &signature.signature[0..5]; // Truncate the transaction ID to the first 5 characters

                // Handle multiple instructions
                for instruction in &transaction_details.transaction.message.instructions {
                    if let Some(parsed) = &instruction.parsed {
                        if let (Some(source), Some(destination), Some(lamports)) = (
                            &parsed.info.source,
                            &parsed.info.destination,
                            &parsed.info.lamports,
                        ) {
                            table.add_row(row![
                                truncated_transaction_id,
                                source,
                                destination,
                                formatted_time,
                                lamports,
                                status
                            ]);
                        } else if let Some(mint) = &parsed.info.mint {
                            let new_account = parsed.info.newAccount.clone().unwrap_or_else(|| "Unknown".to_string());
                            let owner = parsed.info.owner.clone().unwrap_or_else(|| "Unknown".to_string());
                            let amount = parsed.info.tokenAmount.as_ref().map_or_else(|| "Unknown".to_string(), |ta| ta.amount.clone());
                            let decimals = parsed.info.tokenAmount.as_ref().map_or_else(|| "Unknown".to_string(), |ta| ta.decimals.to_string());
                        
                            mint_table.add_row(row![
                                mint,
                                new_account,
                                owner,
                                amount,
                                decimals
                            ]);
                        } else {
                            debug!("Missing required fields in instruction for signature: {}", signature.signature);
                        }
                    }
                }
            } else {
                debug!("Failed to fetch transaction details for signature: {}", signature.signature);
            }
        } else {
            debug!("No block time available for signature: {}", signature.signature);
        }
    }

    println!("Transaction Table:");
    table.printstd();
    println!("\nMint Information Table:");
    mint_table.printstd();
    
    Ok(())
}