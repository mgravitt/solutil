use serde_json::Value;

#[derive(Debug)]
pub struct FungibleTokenTransfer {
    pub transaction_id: String,
    pub sender: String,
    pub receiver: String,
    pub amount: String,
    pub timestamp: u64,
}

impl FungibleTokenTransfer {
    pub fn from_json(json: &Value, mint_to_match: &str) -> Option<Self> {
        if let Some(instructions) = json.pointer("/result/transaction/message/instructions") {
            if let Some(instructions_array) = instructions.as_array() {
                for instruction in instructions_array {
                    if let Some(parsed) = instruction.pointer("/parsed/info") {
                        if let Some(mint) = parsed.pointer("/mint").and_then(|v| v.as_str()) {
                            if mint == mint_to_match {
                                if let Some(amount) = parsed.pointer("/tokenAmount/uiAmountString").and_then(|v| v.as_str()) {
                                    if let Some(sender) = parsed.pointer("/source").or_else(|| parsed.pointer("/account")).and_then(|v| v.as_str()) {
                                        if let Some(receiver) = parsed.pointer("/destination").or_else(|| parsed.pointer("/account")).and_then(|v| v.as_str()) {
                                            let transaction_id = json.pointer("/result/transaction/signatures/0").and_then(|v| v.as_str())?.to_string();
                                            let timestamp = json.pointer("/result/blockTime").and_then(|v| v.as_u64())?;

                                            return Some(FungibleTokenTransfer {
                                                transaction_id,
                                                sender: sender.to_string(),
                                                receiver: receiver.to_string(),
                                                amount: amount.to_string(),
                                                timestamp,
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }
}

pub fn is_fungible_token_transaction(json: &Value) -> bool {
    if let Some(instructions) = json.pointer("/result/transaction/message/instructions") {
        if let Some(instructions_array) = instructions.as_array() {
            for instruction in instructions_array {
                if let Some(parsed) = instruction.pointer("/parsed/info") {
                    let mint = parsed.pointer("/mint").and_then(|v| v.as_str());
                    let token_amount = parsed.pointer("/tokenAmount");

                    if mint.is_some() && token_amount.is_some() {
                        return true;
                    }
                }
            }
        }
    }
    false
}
