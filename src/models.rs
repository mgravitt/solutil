#![allow(dead_code)]
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SolanaSignature {
    pub signature: String,
    pub slot: u64,
    #[serde(rename = "blockTime")]
    pub block_time: Option<i64>,
    #[serde(rename = "confirmationStatus")]
    pub confirmation_status: Option<String>,
    pub err: Option<serde_json::Value>,
    pub memo: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SolanaTransactionDetails {
    #[serde(rename = "meta")]
    pub meta: TransactionMeta,
    #[serde(rename = "transaction")]
    pub transaction: Transaction,
    pub instructions: Vec<ParsedInstruction>,

}

#[derive(Debug, Deserialize)]
pub struct TransactionMeta {
    #[serde(rename = "postBalances")]
    pub post_balances: Vec<u64>,
    #[serde(rename = "preBalances")]
    pub pre_balances: Vec<u64>,
    #[serde(rename = "status")]
    pub status: Status,
}

#[derive(Debug, Deserialize)]
pub struct Status {
    #[serde(rename = "Ok")]
    pub ok: Option<serde_json::Value>,
    #[serde(rename = "Err")]
    pub err: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct Transaction {
    #[serde(rename = "message")]
    pub message: Message,
}

#[derive(Debug, Deserialize)]
pub struct Message {
    #[serde(rename = "accountKeys")]
    pub account_keys: Vec<AccountKey>,
    #[serde(rename = "instructions")]
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Deserialize)]
pub struct AccountKey {
    pub pubkey: String,
    pub signer: bool,
    pub source: String,
    pub writable: bool,
}

#[derive(Debug, Deserialize)]
pub struct Instruction {
    #[serde(rename = "parsed")]
    pub parsed: Option<ParsedInstruction>,
}

#[derive(Debug, Deserialize)]
pub struct ParsedInstruction {
    #[serde(rename = "info")]
    pub info: Info,
    pub parsed: Option<InstructionInfo>,
}


#[derive(Debug, Deserialize)]
pub struct InstructionInfo {
    pub info: InfoDetails,
    pub r#type: String,
}

#[derive(Debug, Deserialize)]
pub struct InfoDetails {
    pub mint: Option<String>,
    #[serde(rename = "newAccount")]
    pub new_account: Option<String>,
    pub owner: Option<String>,
    pub source: Option<String>,
    #[serde(rename = "tokenAmount")]
    pub token_amount: Option<UiTokenAmount>,
}

#[derive(Debug, Deserialize)]
pub struct UiTokenAmount {
    pub amount: String,
    pub decimals: u8,
    #[serde(rename = "uiAmount")]
    pub ui_amount: f64,
    #[serde(rename = "uiAmountString")]
    pub ui_amount_string: String,
}

#[derive(Debug, Deserialize)]
pub struct Info {
    #[serde(rename = "destination")]
    pub destination: Option<String>,
    #[serde(rename = "source")]
    pub source: Option<String>,
    #[serde(rename = "lamports")]
    pub lamports: Option<u64>,
    // Add new fields
    pub mint: Option<String>,
    #[serde(rename = "newAccount")]
    pub new_account: Option<String>,
    pub owner: Option<String>,
    #[serde(rename = "tokenAmount")]
    pub token_amount: Option<UiTokenAmount>,
}

#[derive(Debug, Deserialize)]
pub struct RpcResponse<T> {
    pub result: T,
}

