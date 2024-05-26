use serde_json::Value;

#[derive(Debug)]
pub struct SolTransfer {
    pub transaction_id: String,
    pub sender: String,
    pub receiver: String,
    pub amount: u64,
    pub timestamp: u64,
}

impl SolTransfer {
    pub fn from_json(json: &Value) -> Result<Self, &'static str> {
        let transaction_id = json.pointer("/result/transaction/signatures/0")
            .and_then(|v| v.as_str())
            .ok_or("transaction_id not found")?
            .to_string();

        let sender = json.pointer("/result/transaction/message/accountKeys/0/pubkey")
            .and_then(|v| v.as_str())
            .ok_or("sender not found")?
            .to_string();

        let receiver = json.pointer("/result/transaction/message/instructions/0/parsed/info/destination")
            .and_then(|v| v.as_str())
            .ok_or("receiver not found")?
            .to_string();

        let amount = json.pointer("/result/transaction/message/instructions/0/parsed/info/lamports")
            .and_then(|v| v.as_u64())
            .ok_or("amount not found")?;

        let timestamp = json.pointer("/result/blockTime")
            .and_then(|v| v.as_u64())
            .ok_or("timestamp not found")?;

        Ok(SolTransfer {
            transaction_id,
            sender,
            receiver,
            amount,
            timestamp,
        })
    }
}

pub fn is_sol_transfer(json: &Value) -> bool {
    json.pointer("/result/transaction/message/instructions")
        .and_then(|v| v.as_array())
        .map_or(false, |instructions| {
            instructions.iter().any(|instruction| {
                instruction.pointer("/parsed/info/lamports").is_some() &&
                instruction.pointer("/parsed/info/destination").is_some()
            })
        })
}