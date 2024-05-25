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
    pub fn from_json(json: &Value) -> Option<Self> {
        let transaction_id = json.pointer("/result/transaction/signatures/0")?.as_str()?.to_string();
        let sender = json.pointer("/result/transaction/message/accountKeys/0/pubkey")?.as_str()?.to_string();
        let receiver = json.pointer("/result/transaction/message/instructions/0/parsed/info/destination")?.as_str()?.to_string();
        let amount = json.pointer("/result/transaction/message/instructions/0/parsed/info/lamports")?.as_u64()?;
        let timestamp = json.pointer("/result/blockTime")?.as_u64()?;

        Some(SolTransfer {
            transaction_id,
            sender,
            receiver,
            amount,
            timestamp,
        })
    }
}

pub fn is_sol_transfer(json: &Value) -> bool {
    if let Some(instructions) = json.pointer("/result/transaction/message/instructions") {
        if let Some(instructions_array) = instructions.as_array() {
            for instruction in instructions_array {
                if let Some(parsed) = instruction.pointer("/parsed/info") {
                    let lamports = parsed.pointer("/lamports");
                    let destination = parsed.pointer("/destination");

                    if lamports.is_some() && destination.is_some() {
                        return true;
                    }
                }
            }
        }
    }
    false
}
