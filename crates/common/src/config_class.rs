use serde::{Deserialize, Serialize};

/// The comparability key for a machine: which corpus rows (and golden
/// baselines) it may be matched against. A ticket is matched only against like
/// configs, so the config class is a column on every corpus row.
///
/// On a CEC build the BOM revision anchors the class; on a bare box the class
/// is a stable content hash over the normalized hardware/software inventory.
/// The two unify as "BOM revision when present, else derived hash".
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfigClass {
    /// A CEC build, anchored to its BOM revision.
    BomRevision(String),
    /// A bare box: an order-independent content hash over normalized inventory
    /// entries (e.g. CIM hardware and driver inventory). The hash carries no
    /// identity data — only what the entries themselves expose.
    DerivedHash(String),
}

impl ConfigClass {
    /// The class for a CEC build with a known BOM revision.
    pub fn from_bom(revision: impl Into<String>) -> Self {
        ConfigClass::BomRevision(revision.into())
    }

    /// Derive the class for a bare box from its inventory entries. Entries are
    /// normalized (trimmed, lowercased) and hashed order-independently, so the
    /// same inventory always yields the same class regardless of enumeration
    /// order.
    pub fn from_inventory<I, S>(entries: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let normalized: Vec<String> = entries
            .into_iter()
            .map(|entry| entry.as_ref().trim().to_lowercase())
            .collect();
        let keys: Vec<&str> = normalized.iter().map(String::as_str).collect();
        ConfigClass::DerivedHash(crate::hash::fingerprint_of(
            crate::hash::FingerprintDomain::Config,
            &keys,
        ))
    }

    /// The comparable key string: the BOM revision or the derived hash.
    pub fn key(&self) -> &str {
        match self {
            ConfigClass::BomRevision(revision) => revision,
            ConfigClass::DerivedHash(hash) => hash,
        }
    }
}
