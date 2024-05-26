use crate::fungible_token_transfer::is_fungible_token_transaction;
use crate::fungible_token_transfer::FungibleTokenTransfer;
use crate::models::RpcResponse;
use crate::models::SolanaSignature;
use chrono::{TimeZone, Utc};
use prettytable::{row, Cell, Row, Table};
use reqwest::blocking::Client;
use serde_json::Value;
use std::error::Error;

pub fn print_fungible_transfer_history(
    solana_rpc_url: &str,
    solana_address: &str,
    token_mint: &str,
) -> Result<(), Box<dyn Error>> {
    println!(
        "\nFetching fungible token {} transaction history for {} via {}\n",
        token_mint, solana_address, solana_rpc_url
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

        if is_fungible_token_transaction(&json) {
            if let Some(transfer) = FungibleTokenTransfer::from_json(&json, token_mint) {
                #[allow(deprecated)]
                let timestamp = Utc.timestamp(transfer.timestamp as i64, 0);
                table.add_row(Row::new(vec![
                    Cell::new(&transfer.transaction_id[0..10]),
                    Cell::new(&transfer.sender),
                    Cell::new(&transfer.receiver),
                    Cell::new(&transfer.amount),
                    Cell::new(&timestamp.to_string()),
                ]));
            }
        }
    }

    table.printstd();
    Ok(())
}
