use std::error::Error;
use clap::Parser;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    signature::{Signer, write_keypair_file, read_keypair_file, Keypair},
    system_instruction,
    transaction::Transaction,
};
use spl_associated_token_account::{get_associated_token_address, instruction::create_associated_token_account};
use spl_token::instruction::transfer;

mod fungible_history;
mod fungible_token_transfer;
mod history;
mod models;
mod sol_history;
mod sol_transfer;

/// Simple program to fetch Solana transaction history
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Command to execute
    #[clap(subcommand)]
    command: Command,
}

#[derive(Parser, Debug)]
enum Command {
    /// Fetch transaction history and save each transaction to a file
    SaveHistory {
        /// Solana RPC URL
        #[arg(short = 'u', long = "url")]
        solana_rpc_url: String,

        /// Solana address
        #[arg(short = 'a', long = "address")]
        solana_address: String,
    },
    /// Fetch SOL transaction history
    SOLHistory {
        /// Solana RPC URL
        #[arg(short = 'u', long = "url")]
        solana_rpc_url: String,

        /// Solana address
        #[arg(short = 'a', long = "address")]
        solana_address: String,
    },
    /// Fetch fungible token transaction history
    FungibleHistory {
        /// Solana RPC URL
        #[arg(short = 'u', long = "url")]
        solana_rpc_url: String,

        /// Solana address
        #[arg(short = 'a', long = "address")]
        solana_address: String,

        /// Mint address of the fungible token
        #[arg(short = 'm', long = "mint")]
        mint_address: String,
    },
    /// Send SOL from one account to another
    Send {
        /// Solana RPC URL
        #[arg(short = 'u', long = "url")]
        solana_rpc_url: String,

        /// Sender's keypair file
        #[arg(short = 'k', long = "keypair")]
        keypair: String,

        /// Recipient's address
        #[arg(short = 'r', long = "recipient")]
        recipient: String,

        /// Amount of SOL to send
        #[arg(short = 'a', long = "amount")]
        amount: f64,
    },
    /// Send fungible tokens from one account to another
    SendFungible {
        /// Solana RPC URL
        #[arg(short = 'u', long = "url")]
        solana_rpc_url: String,

        /// Sender's keypair file
        #[arg(short = 'k', long = "keypair")]
        keypair: String,

        /// Recipient's address
        #[arg(short = 'r', long = "recipient")]
        recipient: String,

        /// Amount of the token to send
        #[arg(short = 'a', long = "amount")]
        amount: f64,

        /// Mint address of the fungible token
        #[arg(short = 'm', long = "mint")]
        mint_address: String,

        /// Decimal places for the token amount
        #[arg(short = 'd', long = "decimals", default_value_t = 9)]
        decimals: u32,
    },
    /// Create a new Solana keypair file
    GenerateKeypair {
        /// File path to save the keypair
        #[arg(short = 'f', long = "file")]
        file_path: String,
    },
    /// Inspect a keypair and print the Solana address
    Inspect {
        /// Keypair file to inspect
        #[arg(short = 'k', long = "keypair")]
        keypair_file: String,
    },
}

fn main() -> Result<(), Box<dyn Error>> {
    // Initialize the logger
    env_logger::init();

    // Parse arguments
    let args = Args::parse();

    match args.command {
        Command::SOLHistory {
            solana_rpc_url,
            solana_address,
        } => {
            sol_history::print_sol_transfer_history(&solana_rpc_url, &solana_address)?;
        }
        Command::SaveHistory {
            solana_rpc_url,
            solana_address,
        } => {
            history::fetch_transaction_history(&solana_rpc_url, &solana_address)?;
        }
        Command::FungibleHistory {
            solana_rpc_url,
            solana_address,
            mint_address,
        } => {
            // Logic to fetch fungible token transaction history
            fungible_history::print_fungible_transfer_history(
                &solana_rpc_url,
                &solana_address,
                &mint_address,
            )?;
        }
        Command::Send {
            solana_rpc_url,
            keypair,
            recipient,
            amount,
        } => {
            println!(
                "\nSend {:?} SOL from {} to {} via {}\n",
                amount, keypair, recipient, solana_rpc_url
            );

            let rpc_client = RpcClient::new(solana_rpc_url);
            let sender_keypair = read_keypair_file(&keypair)?;
            let recipient_pubkey = recipient.parse()?;
            let lamports = (amount * 1_000_000_000.0) as u64; // Convert SOL to lamports

            let recent_blockhash = rpc_client.get_latest_blockhash()?;
            let transfer_instruction =
                system_instruction::transfer(&sender_keypair.pubkey(), &recipient_pubkey, lamports);
            let transaction = Transaction::new_signed_with_payer(
                &[transfer_instruction],
                Some(&sender_keypair.pubkey()),
                &[&sender_keypair],
                recent_blockhash,
            );

            let signature = rpc_client.send_and_confirm_transaction(&transaction)?;
            println!("Transaction sent successfully. Signature: {}", signature);
        }
        Command::SendFungible {
            solana_rpc_url,
            keypair,
            recipient,
            amount,
            mint_address,
            decimals,
        } => {
            println!(
                "\nSend {:?} tokens from {} to {} via {} using mint {} with {} decimal places\n",
                amount, keypair, recipient, solana_rpc_url, mint_address, decimals
            );

            let rpc_client = RpcClient::new(solana_rpc_url);
            let sender_keypair = read_keypair_file(&keypair)?;
            let recipient_pubkey = recipient.parse()?;
            let token_program_id = spl_token_2022::id();
            let mint_pubkey = mint_address.parse()?;
            let sender_token_account = get_associated_token_address(&sender_keypair.pubkey(), &mint_pubkey);
            let recipient_token_account = get_associated_token_address(&recipient_pubkey, &mint_pubkey);
            let amount = (amount * 10f64.powi(decimals as i32)) as u64; // Convert amount using specified decimal places

            // Create associated token account for recipient if it doesn't exist
            let mut instructions = vec![];
            if rpc_client.get_account(&recipient_token_account).is_err() {
                let create_account_instruction = create_associated_token_account(
                    &sender_keypair.pubkey(),
                    &recipient_pubkey,
                    &mint_pubkey,
                    &token_program_id,
                );
                instructions.push(create_account_instruction);
            }

            // Transfer instruction
            let transfer_instruction = transfer(
                &token_program_id,
                &sender_token_account,
                &recipient_token_account,
                &sender_keypair.pubkey(),
                &[&sender_keypair.pubkey()],
                amount,
            )?;
            instructions.push(transfer_instruction);

            let recent_blockhash = rpc_client.get_latest_blockhash()?;
            let transaction = Transaction::new_signed_with_payer(
                &instructions,
                Some(&sender_keypair.pubkey()),
                &[&sender_keypair],
                recent_blockhash,
            );

            let signature = rpc_client.send_and_confirm_transaction(&transaction)?;
            println!("Transaction sent successfully. Signature: {}", signature);
        }
        Command::GenerateKeypair { file_path } => {
            let new_keypair = Keypair::new();
            write_keypair_file(&new_keypair, &file_path)?;
            println!("New keypair generated and saved to {}", file_path);
        }
        Command::Inspect { keypair_file } => {
            let keypair = read_keypair_file(&keypair_file)?;
            println!("Solana Address: {}", keypair.pubkey());
        }
    }

    Ok(())
}