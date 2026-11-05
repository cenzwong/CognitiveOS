use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", content = "args", rename_all = "snake_case")]
pub enum CognitiveInstruction {
    MemoryRead {
        address: String,
    },
    MemorySearch {
        query: String,
        limit: u32,
    },
    MemoryWrite {
        address: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        expected_version: Option<u64>,
        content: Value,
    },
    ToolExecute {
        tool_id: String,
        arguments: Value,
    },
    RequestInput {
        question: String,
    },
    RequestApproval {
        action: String,
        reason: String,
    },
    Yield {
        reason: String,
    },
    Complete {
        result: Value,
        #[serde(default)]
        evidence_refs: Vec<String>,
    },
    Fail {
        reason: String,
        recoverable: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelDecision {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    pub instruction: CognitiveInstruction,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelResponseContract {
    pub schema_version: u32,
    pub decision: ModelDecision,
}
