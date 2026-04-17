use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FileOp {
    Read,
    Edit,
    Write,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileInteractionEntry {
    pub path: String,
    pub op: FileOp,
    pub count: u32,
    pub last_ts_ms: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entry_round_trips_through_json() {
        let entry = FileInteractionEntry {
            path: "crates/memd-core/src/lib.rs".into(),
            op: FileOp::Read,
            count: 3,
            last_ts_ms: 1_700_000_000_000,
        };
        let json = serde_json::to_string(&entry).unwrap();
        let parsed: FileInteractionEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, entry);
    }
}
