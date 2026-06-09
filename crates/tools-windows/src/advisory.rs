//! Firmware and driver support advisories.
//!
//! When the evidence implicates the board or a driver, the most useful
//! deliverable is often not an executable plan but a precise advisory: the
//! exact board and BIOS the machine has, where its vendor publishes
//! downloads, the exact term to search, and what to do — because BIOS and
//! firmware are outside the agent's executable vocabulary by design
//! (advisory-only, never agent-executed; a restore point does not cover
//! firmware).
//!
//! De-identification posture: a [`BoardIdentity`] carries configuration only
//! (manufacturer, product, versions) — never serial numbers or service tags.
//! The advisory is ticket context for the user's screen; it does not enter
//! the corpus. Vendor links point at stable support landing pages, not
//! per-model deep links that rot.

use common::Fluency;
use serde::{Deserialize, Serialize};

/// The board and firmware identity of a machine: configuration fields only,
/// no identity-bearing fields (no serial numbers, no asset or service tags).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoardIdentity {
    /// Board (motherboard) manufacturer, e.g. "ASUSTeK COMPUTER INC.".
    pub manufacturer: String,
    /// Board product/model, e.g. "PRIME X570-PRO".
    pub product: String,
    /// Board hardware revision, when reported.
    pub version: String,
    /// Installed BIOS/UEFI version string (SMBIOS).
    pub bios_version: String,
    /// BIOS release date, when reported (yyyy-MM-dd).
    pub bios_date: String,
    /// System manufacturer (differs from the board's on OEM machines).
    pub system_manufacturer: String,
    /// System model (the support key on OEM machines).
    pub system_model: String,
}

impl BoardIdentity {
    /// Parse the structured payload of the `board_info` tool. Returns `None`
    /// when the payload carries no board section at all (e.g. an unsupported
    /// host); missing individual fields become empty strings.
    pub fn from_tool_data(data: &serde_json::Value) -> Option<Self> {
        let board = data.get("board")?;
        let text = |value: &serde_json::Value, key: &str| -> String {
            value
                .get(key)
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .trim()
                .to_string()
        };
        let bios = data.get("bios").cloned().unwrap_or_default();
        let system = data.get("system").cloned().unwrap_or_default();
        Some(Self {
            manufacturer: text(board, "Manufacturer"),
            product: text(board, "Product"),
            version: text(board, "Version"),
            bios_version: text(&bios, "SMBIOSBIOSVersion"),
            bios_date: text(&bios, "ReleaseDate"),
            system_manufacturer: text(&system, "Manufacturer"),
            system_model: text(&system, "Model"),
        })
    }
}

/// A support advisory: where the vendor publishes downloads for this machine,
/// what to search for, and numbered plain-language steps.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SupportAdvisory {
    /// Recognized vendor name ("ASUS", "Dell", …) or "unknown vendor".
    pub vendor: String,
    /// The vendor's stable support/download landing page.
    pub download_page: String,
    /// The exact model string to search for on that page.
    pub search_term: String,
    /// Numbered plain-language steps for the user.
    pub steps: Vec<String>,
}

/// Vendors whose *system* model — not the bare board — is the support key,
/// matched against the system manufacturer.
const OEM_VENDORS: &[(&str, &str, &str)] = &[
    ("dell", "Dell", "https://www.dell.com/support/home/"),
    ("hp", "HP", "https://support.hp.com/drivers"),
    ("hewlett", "HP", "https://support.hp.com/drivers"),
    ("lenovo", "Lenovo", "https://pcsupport.lenovo.com/"),
];

/// Motherboard vendors for self-built machines, matched against the board
/// manufacturer.
const BOARD_VENDORS: &[(&str, &str, &str)] = &[
    (
        "asus",
        "ASUS",
        "https://www.asus.com/support/download-center/",
    ),
    ("msi", "MSI", "https://www.msi.com/support/download"),
    ("micro-star", "MSI", "https://www.msi.com/support/download"),
    (
        "gigabyte",
        "Gigabyte",
        "https://www.gigabyte.com/Support/Consumer/Download",
    ),
    (
        "asrock",
        "ASRock",
        "https://www.asrock.com/support/index.asp",
    ),
];

/// Build the firmware (BIOS/UEFI) advisory for a board, in the register the
/// person's own explanation earned.
///
/// OEM machines are directed by system model; self-built machines by board
/// manufacturer and product. Unrecognized vendors get a generic but still
/// precise advisory (the exact strings to search the web for). The guided
/// register teaches each step; the technical register is measured and
/// compact — but the safety floor (never agent-flashed, never cut power
/// mid-update) is stated in both, because the register calibrates teaching,
/// never safety.
pub fn firmware_advisory(board: &BoardIdentity, fluency: Fluency) -> SupportAdvisory {
    let system_make = board.system_manufacturer.to_lowercase();
    let board_make = board.manufacturer.to_lowercase();

    let (vendor, page, search_term, oem) = if let Some((_, name, page)) = OEM_VENDORS
        .iter()
        .find(|(needle, _, _)| system_make.contains(needle))
    {
        (
            name.to_string(),
            page.to_string(),
            board.system_model.clone(),
            true,
        )
    } else if let Some((_, name, page)) = BOARD_VENDORS
        .iter()
        .find(|(needle, _, _)| board_make.contains(needle))
    {
        (
            name.to_string(),
            page.to_string(),
            board.product.clone(),
            false,
        )
    } else {
        (
            "unknown vendor".to_string(),
            String::new(),
            format!("{} {}", board.manufacturer, board.product)
                .trim()
                .to_string(),
            false,
        )
    };

    let installed = match (board.bios_version.is_empty(), board.bios_date.is_empty()) {
        (false, false) => format!("{} (released {})", board.bios_version, board.bios_date),
        (false, true) => board.bios_version.clone(),
        _ => "unknown".to_string(),
    };

    if fluency == Fluency::Technical {
        // Measured register: the same facts and the same safety floor, none
        // of the teaching.
        let mut texts: Vec<String> = Vec::new();
        if page.is_empty() {
            texts.push(format!(
                "Search \"{search_term} support download\" — manufacturer's own site only."
            ));
        } else {
            texts.push(format!(
                "{page} -> search \"{search_term}\" -> BIOS/UEFI downloads."
            ));
        }
        if oem {
            texts.push(
                "Service Tag is on the chassis sticker; the agent does not collect it.".to_string(),
            );
        }
        texts.push(format!(
            "Installed: {installed}. Flash only if the vendor lists newer, per their \
             instructions (USB flasher in setup). Do not cut power mid-flash."
        ));
        texts.push(
            "The agent never flashes firmware (advisory-only; restore points do not \
             cover it)."
                .to_string(),
        );
        return SupportAdvisory {
            vendor,
            download_page: page,
            search_term,
            steps: texts
                .into_iter()
                .enumerate()
                .map(|(i, text)| format!("{}. {text}", i + 1))
                .collect(),
        };
    }

    // Guided register: written for someone who has never updated a BIOS —
    // every step says what the user will see, what the words mean, and what
    // a result looks like.
    let mut texts: Vec<String> = Vec::new();
    if page.is_empty() {
        texts.push(format!(
            "In your web browser, search for \"{search_term} support download\" and \
             open the result that is on the manufacturer's own website. Avoid \
             third-party download sites — they often bundle unwanted software."
        ));
    } else {
        texts.push(format!(
            "Open this web page in your browser: {page} — that is {vendor}'s official \
             download site."
        ));
    }
    texts.push(format!(
        "On that page, type your exact model into the search box: \"{search_term}\" \
         — then click the matching model in the results. (This text was read \
         directly from your computer, so you can copy it as-is.)"
    ));
    if oem {
        texts.push(
            "If the page asks for a 'Service Tag' or serial number, it is printed on \
             a sticker on the machine itself — usually on the bottom of a laptop, or \
             the back or side of a desktop. The support agent never reads or sends \
             this number; only you see it."
                .to_string(),
        );
    }
    texts.push(
        "Find the downloads section called 'BIOS' or 'UEFI'. (The BIOS is the \
         built-in startup software on the computer's main board — updating it can \
         fix crashes, restarts, and compatibility problems.) You may need to click a \
         tab named 'Support', 'Drivers', or 'Downloads' first."
            .to_string(),
    );
    texts.push(format!(
        "Look at the newest entry in that list — each one shows a version number and \
         a date. Your computer currently has version {installed}. If the newest \
         version number on the page is higher than yours, an update is available. If \
         they match, you already have the latest and can stop here."
    ));
    texts.push(
        "If an update is available, click Download and save the file — but do not \
         open or run it yet. On the same page, open the maker's update instructions \
         (often a link named 'How to update BIOS'). Most boards update from a USB \
         stick using a built-in updater that you reach by pressing a key shown on \
         screen (often Del or F2) while the computer is starting up."
            .to_string(),
    );
    texts.push(
        "Safety notes: follow the maker's steps exactly, and never turn the computer \
         off while the update is running — losing power mid-update can leave the \
         board unusable. The support agent will never install a BIOS update for you \
         automatically: this step is always yours to do, at your own pace."
            .to_string(),
    );

    SupportAdvisory {
        vendor,
        download_page: page,
        search_term,
        steps: texts
            .into_iter()
            .enumerate()
            .map(|(i, text)| format!("{}. {text}", i + 1))
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn diy_board() -> BoardIdentity {
        BoardIdentity::from_tool_data(&serde_json::json!({
            "board": {
                "Manufacturer": "ASUSTeK COMPUTER INC.",
                "Product": "PRIME X570-PRO",
                "Version": "Rev X.0x"
            },
            "bios": { "SMBIOSBIOSVersion": "4021", "ReleaseDate": "2021-03-08" },
            "system": { "Manufacturer": "System manufacturer", "Model": "System Product Name" }
        }))
        .expect("parses")
    }

    #[test]
    fn parses_the_board_info_payload() {
        let board = diy_board();
        assert_eq!(board.product, "PRIME X570-PRO");
        assert_eq!(board.bios_version, "4021");
        assert!(BoardIdentity::from_tool_data(&serde_json::json!({})).is_none());
    }

    #[test]
    fn diy_boards_are_directed_by_board_vendor_and_product() {
        let advisory = firmware_advisory(&diy_board(), Fluency::Guided);
        assert_eq!(advisory.vendor, "ASUS");
        assert!(advisory.download_page.contains("asus.com"));
        assert_eq!(advisory.search_term, "PRIME X570-PRO");
        let joined = advisory.steps.join(" ");
        assert!(joined.contains("4021 (released 2021-03-08)"), "{joined}");
        assert!(joined.contains("never install a BIOS update for you automatically"));
    }

    #[test]
    fn the_advisory_explains_its_own_terms_for_non_technical_users() {
        let joined = firmware_advisory(&diy_board(), Fluency::Guided)
            .steps
            .join(" ");
        // BIOS is defined, not assumed.
        assert!(
            joined.contains("built-in startup software"),
            "BIOS must be explained: {joined}"
        );
        // "Compare versions" is spelled out as a concrete observation.
        assert!(joined.contains("version number and a date"), "{joined}");
        assert!(
            joined.contains("higher than yours"),
            "what 'newer' means must be concrete: {joined}"
        );
        assert!(
            joined.contains("If they match, you already have the latest"),
            "the no-action case must be stated: {joined}"
        );
        // The dangerous part carries the why, not just the rule.
        assert!(joined.contains("losing power mid-update"), "{joined}");
        // Where to find the model string is explained too.
        assert!(
            joined.contains("read directly from your computer"),
            "{joined}"
        );
    }

    #[test]
    fn oem_machines_are_directed_by_system_model() {
        let mut board = diy_board();
        board.system_manufacturer = "Dell Inc.".into();
        board.system_model = "OptiPlex 7090".into();
        let advisory = firmware_advisory(&board, Fluency::Guided);
        assert_eq!(advisory.vendor, "Dell");
        assert!(advisory.download_page.contains("dell.com"));
        assert_eq!(advisory.search_term, "OptiPlex 7090");
        // The service tag is read from the sticker, never collected.
        assert!(advisory.steps.join(" ").contains("sticker"));
    }

    #[test]
    fn unknown_vendors_still_get_a_precise_advisory() {
        let mut board = diy_board();
        board.manufacturer = "Shenzhen Example Tech".into();
        board.system_manufacturer = "Example".into();
        let advisory = firmware_advisory(&board, Fluency::Guided);
        assert_eq!(advisory.vendor, "unknown vendor");
        assert!(advisory.download_page.is_empty());
        assert!(advisory.steps[0].contains("Shenzhen Example Tech PRIME X570-PRO"));
        assert!(advisory.steps[0].contains("Avoid third-party download sites"));
    }

    #[test]
    fn the_technical_register_is_measured_but_keeps_the_safety_floor() {
        let guided = firmware_advisory(&diy_board(), Fluency::Guided);
        let technical = firmware_advisory(&diy_board(), Fluency::Technical);
        // Measured: fewer, shorter steps; the teaching is gone.
        assert!(technical.steps.len() < guided.steps.len());
        let joined = technical.steps.join(" ");
        assert!(
            !joined.contains("built-in startup software"),
            "a fluent reporter does not need BIOS defined: {joined}"
        );
        // Same facts: vendor page, exact model, installed version.
        assert!(
            joined.contains("asrock") || joined.contains("asus.com"),
            "{joined}"
        );
        assert!(joined.contains("PRIME X570-PRO"));
        assert!(joined.contains("4021 (released 2021-03-08)"));
        // Same safety floor: never agent-flashed, never cut power.
        assert!(joined.contains("never flashes firmware"), "{joined}");
        assert!(joined.contains("Do not cut power"), "{joined}");
    }

    #[test]
    fn the_advisory_never_carries_identity_fields() {
        // BoardIdentity has no serial/tag fields by construction; the
        // serialized advisory must not mention any either.
        let advisory = firmware_advisory(&diy_board(), Fluency::Guided);
        let json = serde_json::to_string(&advisory)
            .expect("serializes")
            .to_lowercase();
        assert!(!json.contains("serial"));
        assert!(!json.contains("service tag"));
    }
}
