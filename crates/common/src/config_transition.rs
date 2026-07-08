//! Config-transition detection: what changed between a machine's PRIOR and
//! CURRENT inventory, as a structured, comparable delta.
//!
//! [`ConfigClass`](crate::ConfigClass) collapses an inventory to one opaque hash
//! — two configs are "same" or "different" and you cannot see WHAT moved. A large
//! share of experience-heavy fixes, though, are triggered by a specific
//! TRANSITION (a part swapped, an OS updated) rather than a symptom appearing
//! from nowhere: swap an RTX 5070 for a 5080 and you must run a display-driver
//! clean-uninstall even after a fresh install. Recognizing that needs the
//! structured, per-category delta this module computes.
//!
//! This is the detection PRIMITIVE. It takes a `prior` and a `current` inventory
//! (the same identity-free `category:value` keys the `ConfigClass` already
//! consumes) and classifies each per-category change. Who STORES a machine's
//! prior inventory (the config ledger) and how a transition keys the
//! corpus/retrieval are separate, deferred concerns (see FOLLOWUPS); the
//! primitive is useful immediately (a tech supplies "was a 5070, now a 5080")
//! and composes with them later.
//!
//! **Not a symptom token.** A transition is its own typed concept, deliberately
//! NOT forced into the closed de-id symptom grammar — a transition is not a
//! symptom, and widening the frozen vocabulary to carry one would conflate the
//! two. The corpus-keying integration (recording a transition on a row) is a
//! schema decision left for when the owner sees this shape.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// A machine's inventory parsed into `category → sorted values`. Built from the
/// same identity-free `category:value` keys the [`ConfigClass`](crate::ConfigClass)
/// derives from (`"gpu:rtx-4070"`, `"os:windows 11"`), so it adds no new
/// collection surface — the caller is still responsible for the keys being
/// identity-free, exactly as for the config class.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct StructuredInventory {
    by_category: BTreeMap<String, Vec<String>>,
}

impl StructuredInventory {
    /// Parse identity-free `category:value` keys. Each key is split on the FIRST
    /// `:` into (category, value); a key with no `:` is kept under the empty
    /// category (kept, never silently dropped). Category and value are trimmed
    /// and lowercased for stable comparison; blank entries are dropped; duplicate
    /// values under one category collapse. This mirrors the normalization
    /// `ConfigClass::from_inventory` applies, so the two agree on what "the same
    /// inventory" means.
    pub fn from_keys<I, S>(keys: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut by_category: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for key in keys {
            let key = key.as_ref().trim();
            if key.is_empty() {
                continue;
            }
            let (category, value) = match key.split_once(':') {
                Some((c, v)) => (c.trim().to_lowercase(), v.trim().to_lowercase()),
                None => (String::new(), key.to_lowercase()),
            };
            if value.is_empty() {
                continue;
            }
            let values = by_category.entry(category).or_default();
            if !values.contains(&value) {
                values.push(value);
            }
        }
        for values in by_category.values_mut() {
            values.sort_unstable();
        }
        Self { by_category }
    }

    /// The categories present, sorted.
    pub fn categories(&self) -> impl Iterator<Item = &str> {
        self.by_category.keys().map(String::as_str)
    }

    /// The (sorted) values recorded under `category`, or empty if absent.
    pub fn values(&self, category: &str) -> &[String] {
        self.by_category
            .get(category)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// Whether the inventory is empty (no categories).
    pub fn is_empty(&self) -> bool {
        self.by_category.is_empty()
    }
}

/// Whether a per-category change stayed WITHIN a family (same vendor/line — e.g.
/// an NVIDIA RTX 5070 → 5080) or crossed families (NVIDIA → AMD, Windows → Linux).
/// The family of a value is its leading alphanumeric run
/// (`"rtx-5070"` → `"rtx"`, `"nvidia rtx 5080"` → `"nvidia"`), so a swap within
/// the same line reads as `WithinFamily`. A within-family swap is exactly the
/// experience-heavy case (the 5070→5080 DDU lesson); a cross-family swap is a
/// bigger change and rarely carries the same workflow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FamilyRelation {
    /// Both sides share a family token — a swap within the same line.
    WithinFamily,
    /// The family token changed — a cross-family swap.
    CrossFamily,
}

/// One category's change between prior and current inventory.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum CategoryDelta {
    /// The category exists in both, with a different value set.
    Changed {
        /// The inventory category (e.g. `"gpu"`).
        category: String,
        /// The prior value set (sorted).
        from: Vec<String>,
        /// The current value set (sorted).
        to: Vec<String>,
        /// Whether the change stayed within a family (see [`FamilyRelation`]).
        relation: FamilyRelation,
    },
    /// The category is present now but was not before (a part added).
    Added {
        /// The inventory category.
        category: String,
        /// The values now present (sorted).
        values: Vec<String>,
    },
    /// The category was present before but is not now (a part removed).
    Removed {
        /// The inventory category.
        category: String,
        /// The values that were present (sorted).
        values: Vec<String>,
    },
}

impl CategoryDelta {
    /// The category this delta concerns.
    pub fn category(&self) -> &str {
        match self {
            CategoryDelta::Changed { category, .. }
            | CategoryDelta::Added { category, .. }
            | CategoryDelta::Removed { category, .. } => category,
        }
    }

    /// A short descriptive label for logs/priming — e.g. `"gpu:changed:within_family"`,
    /// `"ram:added"`, `"os:changed:cross_family"`. This is a human/priming string,
    /// NOT a de-id symptom token (a transition is not a symptom).
    pub fn label(&self) -> String {
        match self {
            CategoryDelta::Changed {
                category, relation, ..
            } => {
                let rel = match relation {
                    FamilyRelation::WithinFamily => "within_family",
                    FamilyRelation::CrossFamily => "cross_family",
                };
                format!("{category}:changed:{rel}")
            }
            CategoryDelta::Added { category, .. } => format!("{category}:added"),
            CategoryDelta::Removed { category, .. } => format!("{category}:removed"),
        }
    }
}

/// The structured transition between a machine's prior and current inventory —
/// the per-category deltas, in category order. Empty when nothing changed.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ConfigTransition {
    deltas: Vec<CategoryDelta>,
}

impl ConfigTransition {
    /// Compute the transition from `prior` to `current`. Categories present in
    /// both with differing value sets are `Changed`; those only in `current` are
    /// `Added`; those only in `prior` are `Removed`. Deltas come back in category
    /// order (deterministic, so two runs over the same inputs agree).
    pub fn between(prior: &StructuredInventory, current: &StructuredInventory) -> Self {
        let mut deltas = Vec::new();
        // Union of categories, in sorted order (BTreeMap keys are sorted; merge).
        let mut categories: Vec<&str> = prior.categories().chain(current.categories()).collect();
        categories.sort_unstable();
        categories.dedup();

        for category in categories {
            let before = prior.values(category);
            let after = current.values(category);
            match (before.is_empty(), after.is_empty()) {
                (true, true) => {}
                (true, false) => deltas.push(CategoryDelta::Added {
                    category: category.to_string(),
                    values: after.to_vec(),
                }),
                (false, true) => deltas.push(CategoryDelta::Removed {
                    category: category.to_string(),
                    values: before.to_vec(),
                }),
                (false, false) => {
                    if before != after {
                        deltas.push(CategoryDelta::Changed {
                            category: category.to_string(),
                            from: before.to_vec(),
                            to: after.to_vec(),
                            relation: family_relation(before, after),
                        });
                    }
                }
            }
        }
        Self { deltas }
    }

    /// The per-category deltas.
    pub fn deltas(&self) -> &[CategoryDelta] {
        &self.deltas
    }

    /// Whether nothing changed (no deltas).
    pub fn is_empty(&self) -> bool {
        self.deltas.is_empty()
    }

    /// Descriptive labels for every delta (see [`CategoryDelta::label`]).
    pub fn labels(&self) -> Vec<String> {
        self.deltas.iter().map(CategoryDelta::label).collect()
    }
}

/// The family token of a value: its leading run of ASCII alphanumeric characters,
/// lowercased. `"rtx-5070"` → `"rtx"`, `"nvidia rtx 5080"` → `"nvidia"`,
/// `"windows 11"` → `"windows"`. Empty if the value has no leading alphanumeric
/// run.
fn family_of(value: &str) -> &str {
    let end = value
        .find(|c: char| !c.is_ascii_alphanumeric())
        .unwrap_or(value.len());
    &value[..end]
}

/// Two value sets are [`FamilyRelation::WithinFamily`] iff their family-token sets
/// are equal and non-empty — every value on both sides shares a line. The common
/// singular case (one value each side) reduces to "same leading token"; a swap
/// that adds/removes a family, or a set with no family token, is `CrossFamily`.
fn family_relation(before: &[String], after: &[String]) -> FamilyRelation {
    let families = |vals: &[String]| -> std::collections::BTreeSet<String> {
        vals.iter()
            .map(|v| family_of(v).to_string())
            .filter(|f| !f.is_empty())
            .collect()
    };
    let bf = families(before);
    let af = families(after);
    if !bf.is_empty() && bf == af {
        FamilyRelation::WithinFamily
    } else {
        FamilyRelation::CrossFamily
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_change_is_an_empty_transition() {
        let a = StructuredInventory::from_keys(["os:windows 11", "gpu:rtx-4070"]);
        let b = StructuredInventory::from_keys(["gpu:rtx-4070", "os:windows 11"]);
        let t = ConfigTransition::between(&a, &b);
        assert!(
            t.is_empty(),
            "identical inventory (any order) → no transition"
        );
    }

    #[test]
    fn a_within_family_gpu_swap_is_detected() {
        // The owner's 5070 → 5080 case: same line (rtx), different card.
        let prior = StructuredInventory::from_keys(["os:windows 11", "gpu:rtx-5070"]);
        let current = StructuredInventory::from_keys(["os:windows 11", "gpu:rtx-5080"]);
        let t = ConfigTransition::between(&prior, &current);
        assert_eq!(t.deltas().len(), 1, "only the gpu changed");
        match &t.deltas()[0] {
            CategoryDelta::Changed {
                category,
                from,
                to,
                relation,
            } => {
                assert_eq!(category, "gpu");
                assert_eq!(from, &["rtx-5070"]);
                assert_eq!(to, &["rtx-5080"]);
                assert_eq!(*relation, FamilyRelation::WithinFamily);
            }
            other => panic!("expected a Changed gpu delta, got {other:?}"),
        }
        assert_eq!(t.labels(), vec!["gpu:changed:within_family"]);
    }

    #[test]
    fn a_cross_family_gpu_swap_is_detected() {
        // NVIDIA line → AMD line: family token changes (rtx → rx).
        let prior = StructuredInventory::from_keys(["gpu:rtx-5070"]);
        let current = StructuredInventory::from_keys(["gpu:rx-7800"]);
        let t = ConfigTransition::between(&prior, &current);
        match &t.deltas()[0] {
            CategoryDelta::Changed { relation, .. } => {
                assert_eq!(*relation, FamilyRelation::CrossFamily);
            }
            other => panic!("expected Changed, got {other:?}"),
        }
    }

    #[test]
    fn added_and_removed_categories_are_distinguished() {
        let prior = StructuredInventory::from_keys(["gpu:rtx-5070", "ram:16gb"]);
        let current = StructuredInventory::from_keys(["gpu:rtx-5070", "nic:intel-i225"]);
        let t = ConfigTransition::between(&prior, &current);
        let labels = t.labels();
        assert!(
            labels.contains(&"nic:added".to_string()),
            "nic added: {labels:?}"
        );
        assert!(
            labels.contains(&"ram:removed".to_string()),
            "ram removed: {labels:?}"
        );
        assert_eq!(t.deltas().len(), 2, "gpu unchanged, so only add+remove");
    }

    #[test]
    fn an_os_upgrade_within_the_same_family_is_within_family() {
        // "windows 11" → "windows 12": same leading token → within family.
        let prior = StructuredInventory::from_keys(["os:windows 11"]);
        let current = StructuredInventory::from_keys(["os:windows 12"]);
        let t = ConfigTransition::between(&prior, &current);
        match &t.deltas()[0] {
            CategoryDelta::Changed { relation, .. } => {
                assert_eq!(*relation, FamilyRelation::WithinFamily)
            }
            other => panic!("expected Changed, got {other:?}"),
        }
    }

    #[test]
    fn family_of_takes_the_leading_alphanumeric_run() {
        assert_eq!(family_of("rtx-5070"), "rtx");
        assert_eq!(family_of("nvidia rtx 5080"), "nvidia");
        assert_eq!(family_of("windows 11"), "windows");
        assert_eq!(family_of("-leading"), "");
    }

    #[test]
    fn keys_without_a_colon_are_kept_not_dropped() {
        // A bare key is kept under the empty category rather than silently lost,
        // so a malformed inventory still produces an honest (if coarse) delta.
        let prior = StructuredInventory::from_keys(["mystery"]);
        let current = StructuredInventory::from_keys(["mystery", "gpu:rtx-5080"]);
        let t = ConfigTransition::between(&prior, &current);
        assert_eq!(t.labels(), vec!["gpu:added"], "the bare key is unchanged");
        assert_eq!(prior.values(""), &["mystery"]);
    }
}
