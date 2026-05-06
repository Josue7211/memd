use anyhow::{Context, bail};
use chrono::{DateTime, Utc};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SignedAuditEntry {
    pub entry_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub actor: String,
    pub action: String,
    pub item_id: String,
    pub context: String,
    pub payload_hash: String,
    pub public_key: String,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct AuditLog {
    pub entries: Vec<SignedAuditEntry>,
}

impl SignedAuditEntry {
    pub fn sign(
        actor: impl Into<String>,
        action: impl Into<String>,
        item_id: impl Into<String>,
        context: impl Into<String>,
        payload: impl AsRef<[u8]>,
        key_seed: impl AsRef<[u8]>,
    ) -> anyhow::Result<Self> {
        let signing_key = signing_key_from_seed(key_seed.as_ref());
        let verifying_key = signing_key.verifying_key();
        let payload_hash = to_hex(Sha256::digest(payload.as_ref()));
        let mut entry = SignedAuditEntry {
            entry_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            actor: actor.into(),
            action: action.into(),
            item_id: item_id.into(),
            context: context.into(),
            payload_hash,
            public_key: to_hex(verifying_key.to_bytes()),
            signature: String::new(),
        };
        let message = entry.signing_payload()?;
        entry.signature = to_hex(signing_key.sign(&message).to_bytes());
        Ok(entry)
    }

    pub fn verify(&self) -> anyhow::Result<bool> {
        let public_key = parse_32(&self.public_key).context("parse public key")?;
        let signature = parse_64(&self.signature).context("parse signature")?;
        let verifying_key = VerifyingKey::from_bytes(&public_key).context("load public key")?;
        let signature = Signature::from_bytes(&signature);
        Ok(verifying_key
            .verify(&self.signing_payload()?, &signature)
            .is_ok())
    }

    fn signing_payload(&self) -> anyhow::Result<Vec<u8>> {
        let payload = serde_json::json!({
            "entry_id": self.entry_id,
            "timestamp": self.timestamp,
            "actor": self.actor,
            "action": self.action,
            "item_id": self.item_id,
            "context": self.context,
            "payload_hash": self.payload_hash,
            "public_key": self.public_key,
        });
        serde_json::to_vec(&payload).context("serialize audit signing payload")
    }
}

impl AuditLog {
    pub fn append(&mut self, entry: SignedAuditEntry) -> anyhow::Result<()> {
        if !entry.verify()? {
            bail!("refuse unsigned or invalid audit entry");
        }
        self.entries.push(entry);
        Ok(())
    }

    pub fn verify_all(&self) -> anyhow::Result<bool> {
        for entry in &self.entries {
            if !entry.verify()? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub fn browse_since(&self, since: DateTime<Utc>) -> Vec<SignedAuditEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.timestamp >= since)
            .cloned()
            .collect()
    }

    pub fn explain(&self, item_id: &str) -> Vec<SignedAuditEntry> {
        self.entries
            .iter()
            .filter(|entry| entry.item_id == item_id)
            .cloned()
            .collect()
    }

    pub fn export_ndjson(&self) -> anyhow::Result<String> {
        let mut lines = Vec::new();
        for entry in &self.entries {
            lines.push(serde_json::to_string(entry)?);
        }
        Ok(format!("{}\n", lines.join("\n")))
    }

    pub fn import_ndjson(input: &str) -> anyhow::Result<Self> {
        let mut entries = Vec::new();
        for (idx, line) in input.lines().enumerate() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let entry: SignedAuditEntry = serde_json::from_str(line)
                .with_context(|| format!("parse audit line {}", idx + 1))?;
            entries.push(entry);
        }
        Ok(Self { entries })
    }
}

fn signing_key_from_seed(seed: &[u8]) -> SigningKey {
    let digest = Sha256::digest(seed);
    let mut bytes = [0_u8; 32];
    bytes.copy_from_slice(&digest[..32]);
    SigningKey::from_bytes(&bytes)
}

fn parse_32(value: &str) -> anyhow::Result<[u8; 32]> {
    let bytes = from_hex(value)?;
    let bytes: [u8; 32] = bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("expected 32 bytes"))?;
    Ok(bytes)
}

fn parse_64(value: &str) -> anyhow::Result<[u8; 64]> {
    let bytes = from_hex(value)?;
    let bytes: [u8; 64] = bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("expected 64 bytes"))?;
    Ok(bytes)
}

fn from_hex(value: &str) -> anyhow::Result<Vec<u8>> {
    if !value.len().is_multiple_of(2) {
        bail!("hex length must be even");
    }
    let mut out = Vec::with_capacity(value.len() / 2);
    for idx in (0..value.len()).step_by(2) {
        let byte = u8::from_str_radix(&value[idx..idx + 2], 16).context("parse hex byte")?;
        out.push(byte);
    }
    Ok(out)
}

fn to_hex(bytes: impl AsRef<[u8]>) -> String {
    bytes
        .as_ref()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signed_entry_verifies() {
        let entry = SignedAuditEntry::sign("codex", "read", "item-1", "ctx", b"payload", b"ws-key")
            .unwrap();
        assert!(entry.verify().unwrap());
    }

    #[test]
    fn tampered_entry_fails_verification() {
        let mut entry = SignedAuditEntry::sign(
            "codex",
            "correction",
            "item-1",
            "ctx",
            b"payload",
            b"ws-key",
        )
        .unwrap();
        entry.action = "tampered".to_string();
        assert!(!entry.verify().unwrap());
    }

    #[test]
    fn audit_log_exports_and_detects_tamper() {
        let mut log = AuditLog::default();
        log.append(
            SignedAuditEntry::sign(
                "codex",
                "promotion",
                "routine-1",
                "ctx",
                b"payload",
                b"ws-key",
            )
            .unwrap(),
        )
        .unwrap();
        let exported = log.export_ndjson().unwrap();
        assert!(
            AuditLog::import_ndjson(&exported)
                .unwrap()
                .verify_all()
                .unwrap()
        );
        let tampered = exported.replace("promotion", "deletion");
        assert!(
            !AuditLog::import_ndjson(&tampered)
                .unwrap()
                .verify_all()
                .unwrap()
        );
    }
}
