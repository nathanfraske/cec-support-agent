//! Host inventory → config class, as a pluggable seam.
//!
//! The comparability `ConfigClass` is derived from a host's *inventory keys* —
//! identity-free, normalized strings like `"os:windows 11"` or `"gpu:rtx-4070"`.
//! This trait lets a richer inventory source (e.g. a device-inventory tool driving
//! the engine over a process boundary) replace the coarse os/arch/family default
//! **without the engine depending on that tool**: a provider yields only `String`s,
//! never a foreign type, and the engine re-derives the class itself.

use crate::ConfigClass;

/// A source of de-identified, comparability config keys for a host.
pub trait InventoryProvider {
    /// Identity-free, normalized inventory keys (e.g. `"os:windows 11"`).
    ///
    /// MUST NOT contain identity (hostname, MAC, IP, serials). The keys are hashed
    /// order-independently into the [`ConfigClass`], so they scope retrieval but are
    /// never themselves stored on a row; callers feeding external keys are still
    /// responsible for de-identifying them at the boundary.
    fn inventory_keys(&self) -> Vec<String>;

    /// The config class for this host: an order-independent hash of the keys.
    fn config_class(&self) -> ConfigClass {
        ConfigClass::from_inventory(self.inventory_keys())
    }
}

/// The default provider: the coarse `os`/`arch`/`family` triple. Standalone — no
/// external dependency — so the engine cold-starts with no inventory tooling. This
/// is the engine's historical behavior, preserved byte-for-byte.
pub struct CoarseHostInventory;

impl InventoryProvider for CoarseHostInventory {
    fn inventory_keys(&self) -> Vec<String> {
        vec![
            format!("os:{}", std::env::consts::OS),
            format!("arch:{}", std::env::consts::ARCH),
            format!("family:{}", std::env::consts::FAMILY),
        ]
    }
}

/// Inventory keys supplied by an external caller (e.g. a device-inventory tool's
/// *already de-identified, allowlisted* keys, handed over a process boundary). The
/// engine trusts only the shape (`String`s) and re-derives the class; it never
/// links the producing tool. A de-identification regression test guards this path.
pub struct ExternalInventory {
    keys: Vec<String>,
}

impl ExternalInventory {
    /// Build from caller-provided keys. Blank lines and surrounding whitespace are
    /// dropped; the caller is responsible for the keys being identity-free.
    pub fn new<I, S>(keys: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let keys = keys
            .into_iter()
            .map(Into::into)
            .map(|k| k.trim().to_string())
            .filter(|k| !k.is_empty())
            .collect();
        Self { keys }
    }

    /// Whether any keys were provided (an empty external set should fall back to the
    /// coarse default rather than hash an empty inventory).
    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }
}

impl InventoryProvider for ExternalInventory {
    fn inventory_keys(&self) -> Vec<String> {
        self.keys.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coarse_inventory_is_os_arch_family() {
        let keys = CoarseHostInventory.inventory_keys();
        assert_eq!(keys.len(), 3);
        assert!(keys[0].starts_with("os:"));
        assert!(keys[1].starts_with("arch:"));
        assert!(keys[2].starts_with("family:"));
        // The class is a pure function of the keys.
        assert_eq!(
            CoarseHostInventory.config_class(),
            ConfigClass::from_inventory(keys)
        );
    }

    #[test]
    fn external_inventory_trims_and_drops_blanks() {
        let ext = ExternalInventory::new(["  os:windows 11  ", "", "  ", "gpu:rtx-4070"]);
        assert_eq!(ext.inventory_keys(), vec!["os:windows 11", "gpu:rtx-4070"]);
        assert!(!ext.is_empty());
        assert!(ExternalInventory::new(Vec::<String>::new()).is_empty());
    }

    #[test]
    fn external_keys_are_hashed_not_stored() {
        // The config class is a hash; raw keys (which a caller must keep
        // identity-free) never appear verbatim in it.
        let ext = ExternalInventory::new(["os:windows 11", "gpu:rtx-4070"]);
        let class = ext.config_class();
        assert!(!class.key().contains("windows"));
        assert!(!class.key().contains("rtx"));
        // Order-independence: same facts, any order → same class.
        let reordered = ExternalInventory::new(["gpu:rtx-4070", "os:windows 11"]);
        assert_eq!(class, reordered.config_class());
    }
}
