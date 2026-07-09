//! Structured symptom extraction.
//!
//! De-identification by structured extraction, not scrubbing: a fault
//! signature is built from a fixed vocabulary of fault terms, structured codes
//! (hex stop codes and WER buckets, `event`/`xid`/`bucket`-prefixed numeric
//! ids), a FROZEN dictionary of Windows bugcheck/stop-code names, and a FROZEN
//! allowlist of OS/driver module names. Everything else in the input —
//! hostnames, usernames, paths, serials, asset tags, in-house binaries — is
//! dropped because it is not a member of a closed list, so the signature is
//! content-free by construction rather than by a scrub pass that has to
//! enumerate what to remove.
//!
//! The two former SHAPE heuristics (`is_stop_code_name` kept any
//! `ALL_CAPS_UNDERSCORE` token; `module_name` kept any `stem.exe`) were
//! denylists-of-shape, not dictionaries: they admitted asset tags, AD groups,
//! and custom binaries by shape (`docs/corpus-leak-prevention.md` §2 Layer
//! 1c/C5). They are replaced here by [`STOP_CODE_NAMES`] and [`MODULE_NAMES`],
//! curated and conservative — a missing entry is a false-negative (a real
//! symptom dropped), an over-broad entry is a leak, so the bias is toward
//! genuinely OS/Microsoft-published names only.

use crate::fault::Symptom;

/// Fault-domain vocabulary. A token is kept verbatim only when it appears
/// here. Sorted; `binary_search` relies on it (enforced by test).
const VOCABULARY: &[&str] = &[
    "bluescreen",
    "boot",
    "bsod",
    "bucket",
    "bugcheck",
    "corrupt",
    "corruption",
    "crash",
    "crashes",
    "crashing",
    "disk",
    "driver",
    "error",
    "event",
    "freeze",
    "frozen",
    "hang",
    "kernel",
    "login",
    "logon",
    "loop",
    "memory",
    "overheat",
    "power",
    "reboot",
    "restart",
    "shutdown",
    "slow",
    "smart",
    "stop",
    "tdr",
    "thermal",
    "timeout",
    "update",
    "voltage",
    "wer",
    "whea",
    "xid",
];

/// Vocabulary words that may carry a numeric id: a decimal token directly
/// after one of these is kept as `prefix_number` (e.g. `event_4101`,
/// `xid_79`, `power_41`). A bare number with no such prefix is dropped — it
/// could be anything, including part of a hostname.
const ID_PREFIXES: &[&str] = &[
    "bucket", "bugcheck", "error", "event", "id", "kernel", "power", "stop", "xid",
];

/// FROZEN Windows bugcheck / stop-code names, in Microsoft's canonical
/// `ALL_CAPS_UNDERSCORE` casing. A crash report names one of these verbatim and
/// users are told to type it exactly, so it is genuine, low-cardinality
/// evidence — unlike an arbitrary `ALL_CAPS_UNDERSCORE` token (an asset tag, an
/// AD group, a service name), which is identity. Membership is the allowlist;
/// the shape is not. Conservative and Microsoft-published only.
///
/// This is the de-id MIRROR of the [`crate::stop_codes::STOP_CODES`] table (same
/// 379 names, sorted here by name for binary search). The two are welded by the
/// `stop_code_names_mirror_the_stop_code_table` drift test — add a code to the
/// table and this list must grow with it, exactly as `deid::ACTION_VOCABULARY`
/// tracks the dispatcher registry. MUST stay sorted (binary search) — enforced.
const STOP_CODE_NAMES: &[&str] = &[
    "ABNORMAL_RESET_DETECTED",
    "ACPI_BIOS_ERROR",
    "ACPI_BIOS_FATAL_ERROR",
    "ACPI_DRIVER_INTERNAL",
    "ACPI_FIRMWARE_WATCHDOG_TIMEOUT",
    "ACTIVE_EX_WORKER_THREAD_TERMINATION",
    "AGP_GART_CORRUPTION",
    "AGP_ILLEGALLY_REPROGRAMMED",
    "AGP_INTERNAL",
    "AGP_INVALID_ACCESS",
    "APC_INDEX_MISMATCH",
    "APP_TAGGING_INITIALIZATION_FAILED",
    "ASSIGN_DRIVE_LETTERS_FAILED",
    "ATDISK_DRIVER_INTERNAL",
    "ATTEMPTED_EXECUTE_OF_NOEXECUTE_MEMORY",
    "ATTEMPTED_SWITCH_FROM_DPC",
    "ATTEMPTED_WRITE_TO_CM_PROTECTED_STORAGE",
    "ATTEMPTED_WRITE_TO_READONLY_MEMORY",
    "BAD_EXHANDLE",
    "BAD_OBJECT_HEADER",
    "BAD_POOL_CALLER",
    "BAD_POOL_HEADER",
    "BAD_SYSTEM_CONFIG_INFO",
    "BC_BLUETOOTH_VERIFIER_FAULT",
    "BC_BTHMINI_VERIFIER_FAULT",
    "BGI_DETECTED_VIOLATION",
    "BITLOCKER_FATAL_ERROR",
    "BOUND_IMAGE_UNSUPPORTED",
    "BUGCODE_ID_DRIVER",
    "BUGCODE_MBBADAPTER_DRIVER",
    "BUGCODE_NDIS_DRIVER",
    "BUGCODE_USB3_DRIVER",
    "BUGCODE_USB_DRIVER",
    "BUGCODE_WIFIADAPTER_DRIVER",
    "CACHE_INITIALIZATION_FAILED",
    "CACHE_MANAGER",
    "CANCEL_STATE_IN_COMPLETED_IRP",
    "CANNOT_WRITE_CONFIGURATION",
    "CDFS_FILE_SYSTEM",
    "CHIPSET_DETECTED_ERROR",
    "CID_HANDLE_CREATION",
    "CID_HANDLE_DELETION",
    "CLOCK_WATCHDOG_TIMEOUT",
    "CLUSTER_CSV_CLUSSVC_DISCONNECT_WATCHDOG",
    "CNSS_FILE_SYSTEM_FILTER",
    "CONFIG_INITIALIZATION_FAILED",
    "CONFIG_LIST_FAILED",
    "COREMSGCALL_INTERNAL_ERROR",
    "COREMSG_INTERNAL_ERROR",
    "CORRUPT_ACCESS_TOKEN",
    "CREATE_DELETE_LOCK_NOT_LOCKED",
    "CRITICAL_INITIALIZATION_FAILURE",
    "CRITICAL_OBJECT_TERMINATION",
    "CRITICAL_PROCESS_DIED",
    "CRITICAL_SERVICE_FAILED",
    "CRITICAL_STRUCTURE_CORRUPTION",
    "CRYPTO_LIBRARY_INTERNAL_ERROR",
    "CRYPTO_SELF_TEST_FAILURE",
    "DAM_WATCHDOG_TIMEOUT",
    "DATA_BUS_ERROR",
    "DATA_COHERENCY_EXCEPTION",
    "DEREF_UNKNOWN_LOGON_SESSION",
    "DEVICE_QUEUE_NOT_BUSY",
    "DEVICE_REFERENCE_COUNT_NOT_ZERO",
    "DFS_FILE_SYSTEM",
    "DIRTY_MAPPED_PAGES_CONGESTION",
    "DIRTY_NOWRITE_PAGES_CONGESTION",
    "DISORDERLY_SHUTDOWN",
    "DMA_COMMON_BUFFER_VECTOR_ERROR",
    "DPC_WATCHDOG_TIMEOUT",
    "DPC_WATCHDOG_VIOLATION",
    "DRIVER_CAUGHT_MODIFYING_FREED_POOL",
    "DRIVER_CORRUPTED_EXPOOL",
    "DRIVER_CORRUPTED_MMPOOL",
    "DRIVER_CORRUPTED_SYSPTES",
    "DRIVER_INVALID_STACK_ACCESS",
    "DRIVER_IRQL_NOT_LESS_OR_EQUAL",
    "DRIVER_LEFT_LOCKED_PAGES_IN_PROCESS",
    "DRIVER_OVERRAN_STACK_BUFFER",
    "DRIVER_PAGE_FAULT_BEYOND_END_OF_ALLOCATION",
    "DRIVER_PAGE_FAULT_IN_FREED_SPECIAL_POOL",
    "DRIVER_PNP_WATCHDOG",
    "DRIVER_PORTION_MUST_BE_NONPAGED",
    "DRIVER_POWER_STATE_FAILURE",
    "DRIVER_RETURNED_HOLDING_CANCEL_LOCK",
    "DRIVER_RETURNED_STATUS_REPARSE_FOR_VOLUME_OPEN",
    "DRIVER_UNLOADED_WITHOUT_CANCELLING_PENDING_OPERATIONS",
    "DRIVER_UNMAPPING_INVALID_VIEW",
    "DRIVER_USED_EXCESSIVE_PTES",
    "DRIVER_VERIFIER_DETECTED_VIOLATION",
    "DRIVER_VERIFIER_DMA_VIOLATION",
    "DRIVER_VERIFIER_IOMANAGER_VIOLATION",
    "DRIVER_VIOLATION",
    "DRIVE_EXTENDER",
    "DYNAMIC_ADD_PROCESSOR_MISMATCH",
    "EFS_FATAL_ERROR",
    "ELAM_DRIVER_DETECTED_FATAL_ERROR",
    "EMPTY_THREAD_REAPER_LIST",
    "EM_INITIALIZATION_FAILURE",
    "END_OF_NT_EVALUATION_PERIOD",
    "ERESOURCE_INVALID_RELEASE",
    "EVENT_TRACING_FATAL_ERROR",
    "EXCEPTION_ON_INVALID_STACK",
    "EXCEPTION_SCOPE_INVALID",
    "EXFAT_FILE_SYSTEM",
    "FAST_ERESOURCE_PRECONDITION_VIOLATION",
    "FATAL_ABNORMAL_RESET_ERROR",
    "FATAL_UNHANDLED_HARD_ERROR",
    "FAT_FILE_SYSTEM",
    "FAULTY_HARDWARE_CORRUPTED_PAGE",
    "FILE_INITIALIZATION_FAILED",
    "FILE_SYSTEM",
    "FLOPPY_INTERNAL_ERROR",
    "FLTMGR_FILE_SYSTEM",
    "FSRTL_EXTRA_CREATE_PARAMETER_VIOLATION",
    "FTDISK_INTERNAL_ERROR",
    "GPIO_CONTROLLER_DRIVER_ERROR",
    "HAL1_INITIALIZATION_FAILED",
    "HAL_BLOCKED_PROCESSOR_INTERNAL_ERROR",
    "HAL_ILLEGAL_IOMMU_PAGE_FAULT",
    "HAL_INITIALIZATION_FAILED",
    "HAL_IOMMU_INTERNAL_ERROR",
    "HAL_MEMORY_ALLOCATION",
    "HANDLE_ERROR_ON_CRITICAL_THREAD",
    "HARDWARE_INTERRUPT_STORM",
    "HARDWARE_WATCHDOG_TIMEOUT",
    "HTTP_DRIVER_CORRUPTED",
    "HYPERGUARD_VIOLATION",
    "HYPERVISOR_ERROR",
    "ILLEGAL_ATS_INITIALIZATION",
    "ILLEGAL_IOMMU_PAGE_FAULT",
    "IMPERSONATING_WORKER_THREAD",
    "INACCESSIBLE_BOOT_DEVICE",
    "INCONSISTENT_IRP",
    "INSTALL_MORE_MEMORY",
    "INSTRUCTION_BUS_ERROR",
    "INSTRUCTION_COHERENCY_EXCEPTION",
    "INSUFFICIENT_SYSTEM_MAP_REGS",
    "INTERNAL_POWER_ERROR",
    "INTERRUPT_EXCEPTION_NOT_HANDLED",
    "INTERRUPT_UNWIND_ATTEMPTED",
    "INVALID_AFFINITY_SET",
    "INVALID_CALLBACK_STACK_ADDRESS",
    "INVALID_CANCEL_OF_FILE_OPEN",
    "INVALID_DATA_ACCESS_TRAP",
    "INVALID_DRIVER_HANDLE",
    "INVALID_EXTENDED_PROCESSOR_STATE",
    "INVALID_FLOATING_POINT_STATE",
    "INVALID_HIBERNATED_STATE",
    "INVALID_IO_BOOST_STATE",
    "INVALID_KERNEL_HANDLE",
    "INVALID_KERNEL_STACK_ADDRESS",
    "INVALID_MDL_RANGE",
    "INVALID_PROCESS_ATTACH_ATTEMPT",
    "INVALID_PROCESS_DETACH_ATTEMPT",
    "INVALID_PUSH_LOCK_FLAGS",
    "INVALID_REGION_OR_SEGMENT",
    "INVALID_RUNDOWN_PROTECTION_FLAGS",
    "INVALID_SILO_DETACH",
    "INVALID_SLOT_ALLOCATOR_FLAGS",
    "INVALID_SOFTWARE_INTERRUPT",
    "INVALID_WORK_QUEUE_ITEM",
    "IO1_INITIALIZATION_FAILED",
    "IPI_WATCHDOG_TIMEOUT",
    "IRQL_GT_ZERO_AT_SYSTEM_SERVICE",
    "IRQL_NOT_DISPATCH_LEVEL",
    "IRQL_NOT_GREATER_OR_EQUAL",
    "IRQL_NOT_LESS_OR_EQUAL",
    "IRQL_UNEXPECTED_VALUE",
    "KASAN_ENLIGHTENMENT_VIOLATION",
    "KASAN_ILLEGAL_ACCESS",
    "KERNEL_APC_PENDING_DURING_EXIT",
    "KERNEL_AUTO_BOOST_INVALID_LOCK_RELEASE",
    "KERNEL_AUTO_BOOST_LOCK_ACQUISITION_WITH_RAISED_IRQL",
    "KERNEL_DATA_INPAGE_ERROR",
    "KERNEL_LOCK_ENTRY_LEAKED_ON_THREAD_TERMINATION",
    "KERNEL_MODE_EXCEPTION_NOT_HANDLED",
    "KERNEL_MODE_EXCEPTION_NOT_HANDLED_M",
    "KERNEL_MODE_HEAP_CORRUPTION",
    "KERNEL_PARTITION_REFERENCE_VIOLATION",
    "KERNEL_SECURITY_CHECK_FAILURE",
    "KERNEL_STACK_INPAGE_ERROR",
    "KERNEL_STACK_LOCKED_AT_EXIT",
    "KERNEL_STORAGE_SLOT_IN_USE",
    "KERNEL_THREAD_PRIORITY_FLOOR_VIOLATION",
    "KERNEL_WMI_INTERNAL",
    "KMODE_EXCEPTION_NOT_HANDLED",
    "LAST_CHANCE_CALLED_FROM_KMODE",
    "LM_SERVER_INTERNAL_ERROR",
    "LOADER_BLOCK_MISMATCH",
    "LOADER_ROLLBACK_DETECTED",
    "LOCKED_PAGES_TRACKER_CORRUPTION",
    "LPC_INITIALIZATION_FAILED",
    "MACHINE_CHECK_EXCEPTION",
    "MAILSLOT_FILE_SYSTEM",
    "MANUALLY_INITIATED_CRASH",
    "MANUALLY_INITIATED_CRASH1",
    "MANUALLY_INITIATED_POWER_BUTTON_HOLD",
    "MAXIMUM_WAIT_OBJECTS_EXCEEDED",
    "MBR_CHECKSUM_MISMATCH",
    "MEMORY1_INITIALIZATION_FAILED",
    "MEMORY_IMAGE_CORRUPT",
    "MEMORY_MANAGEMENT",
    "MICROCODE_REVISION_MISMATCH",
    "MISMATCHED_HAL",
    "MSRPC_STATE_VIOLATION",
    "MUI_NO_VALID_SYSTEM_LANGUAGE",
    "MULTIPLE_IRP_COMPLETE_REQUESTS",
    "MULTIPROCESSOR_CONFIGURATION_NOT_SUPPORTED",
    "MUP_FILE_SYSTEM",
    "MUST_SUCCEED_POOL_EMPTY",
    "MUTEX_ALREADY_OWNED",
    "MUTEX_LEVEL_NUMBER_VIOLATION",
    "NDIS_INTERNAL_ERROR",
    "NETIO_INVALID_POOL_CALLER",
    "NETWORK_BOOT_DUPLICATE_ADDRESS",
    "NETWORK_BOOT_INITIALIZATION_FAILED",
    "NMI_HARDWARE_FAILURE",
    "NMR_INVALID_STATE",
    "NO_BOOT_DEVICE",
    "NO_EXCEPTION_HANDLING_SUPPORT",
    "NO_MORE_IRP_STACK_LOCATIONS",
    "NO_MORE_SYSTEM_PTES",
    "NO_PAGES_AVAILABLE",
    "NO_SPIN_LOCK_AVAILABLE",
    "NO_SUCH_PARTITION",
    "NO_USER_MODE_CONTEXT",
    "NPFS_FILE_SYSTEM",
    "NTFS_FILE_SYSTEM",
    "OBJECT1_INITIALIZATION_FAILED",
    "OBJECT_INITIALIZATION_FAILED",
    "OS_DATA_TAMPERING",
    "PAGE_FAULT_BEYOND_END_OF_ALLOCATION",
    "PAGE_FAULT_IN_FREED_SPECIAL_POOL",
    "PAGE_FAULT_IN_NONPAGED_AREA",
    "PAGE_FAULT_WITH_INTERRUPTS_OFF",
    "PAGE_NOT_ZERO",
    "PANIC_STACK_SWITCH",
    "PASSIVE_INTERRUPT_ERROR",
    "PCI_BUS_DRIVER_INTERNAL",
    "PCI_VERIFIER_DETECTED_VIOLATION",
    "PDC_WATCHDOG_TIMEOUT",
    "PFN_LIST_CORRUPT",
    "PFN_REFERENCE_COUNT",
    "PFN_SHARE_COUNT",
    "PF_DETECTED_CORRUPTION",
    "PHASE0_EXCEPTION",
    "PHASE0_INITIALIZATION_FAILED",
    "PHASE1_INITIALIZATION_FAILED",
    "PINBALL_FILE_SYSTEM",
    "PNP_DETECTED_FATAL_ERROR",
    "POOL_CORRUPTION_IN_FILE_AREA",
    "PORT_DRIVER_INTERNAL",
    "PP0_INITIALIZATION_FAILED",
    "PP1_INITIALIZATION_FAILED",
    "PROCESS1_INITIALIZATION_FAILED",
    "PROCESSOR_DRIVER_INTERNAL",
    "PROCESSOR_START_TIMEOUT",
    "PROCESS_HAS_LOCKED_PAGES",
    "PROCESS_INITIALIZATION_FAILED",
    "PROFILER_CONFIGURATION_ILLEGAL",
    "QUOTA_UNDERFLOW",
    "RAMDISK_BOOT_INITIALIZATION_FAILED",
    "RDR_FILE_SYSTEM",
    "RECURSIVE_NMI",
    "REFERENCE_BY_POINTER",
    "REFMON_INITIALIZATION_FAILED",
    "REFS_FILE_SYSTEM",
    "REF_UNKNOWN_LOGON_SESSION",
    "REGISTRY_ERROR",
    "REGISTRY_FILTER_DRIVER_EXCEPTION",
    "RESERVE_QUEUE_OVERFLOW",
    "RESOURCE_MANAGER_EXCEPTION_NOT_HANDLED",
    "RESOURCE_NOT_OWNED",
    "RESOURCE_OWNER_POINTER_INVALID",
    "SCSI_DISK_DRIVER_INTERNAL",
    "SCSI_VERIFIER_DETECTED_VIOLATION",
    "SDBUS_INTERNAL_ERROR",
    "SECURE_BOOT_VIOLATION",
    "SECURE_FAULT_UNHANDLED",
    "SECURE_KERNEL_ERROR",
    "SECURE_PCI_CONFIG_SPACE_ACCESS_VIOLATION",
    "SECURITY1_INITIALIZATION_FAILED",
    "SECURITY_INITIALIZATION_FAILED",
    "SECURITY_SYSTEM",
    "SERIAL_DRIVER_INTERNAL",
    "SESSION1_INITIALIZATION_FAILED",
    "SESSION2_INITIALIZATION_FAILED",
    "SESSION3_INITIALIZATION_FAILED",
    "SESSION4_INITIALIZATION_FAILED",
    "SESSION5_INITIALIZATION_FAILED",
    "SESSION_HAS_VALID_SPECIAL_POOL_ON_EXIT",
    "SESSION_HAS_VALID_VIEWS_ON_EXIT",
    "SETUP_FAILURE",
    "SET_ENV_VAR_FAILED",
    "SET_OF_INVALID_CONTEXT",
    "SHARED_RESOURCE_CONV_ERROR",
    "SOC_CRITICAL_DEVICE_REMOVED",
    "SOC_SUBSYSTEM_FAILURE",
    "SPECIAL_POOL_DETECTED_MEMORY_CORRUPTION",
    "SPIN_LOCK_ALREADY_OWNED",
    "SPIN_LOCK_INIT_FAILURE",
    "SPIN_LOCK_NOT_OWNED",
    "STATUS_CANNOT_LOAD_REGISTRY_FILE",
    "STATUS_IMAGE_CHECKSUM_MISMATCH",
    "STORAGE_DEVICE_ABNORMALITY_DETECTED",
    "STORAGE_MINIPORT_ERROR",
    "STORE_DATA_STRUCTURE_CORRUPTION",
    "STREAMS_INTERNAL_ERROR",
    "SYMBOLIC_INITIALIZATION_FAILED",
    "SYNTHETIC_WATCHDOG_TIMEOUT",
    "SYSTEM_EXIT_OWNED_MUTEX",
    "SYSTEM_LICENSE_VIOLATION",
    "SYSTEM_PTE_MISUSE",
    "SYSTEM_SCAN_AT_RAISED_IRQL_CAUGHT_IMPROPER_DRIVER_UNLOAD",
    "SYSTEM_SERVICE_EXCEPTION",
    "SYSTEM_THREAD_EXCEPTION_NOT_HANDLED",
    "SYSTEM_THREAD_EXCEPTION_NOT_HANDLED_M",
    "SYSTEM_UNWIND_PREVIOUS_USER",
    "TARGET_MDL_TOO_SMALL",
    "TCPIP_AOAC_NIC_ACTIVE_REFERENCE_LEAK",
    "TERMINAL_SERVER_DRIVER_MADE_INCORRECT_MEMORY_REFERENCE",
    "THIRD_PARTY_FILE_SYSTEM_FAILURE",
    "THREAD_NOT_MUTEX_OWNER",
    "THREAD_STUCK_IN_DEVICE_DRIVER",
    "THREAD_STUCK_IN_DEVICE_DRIVER_M",
    "THREAD_TERMINATE_HELD_MUTEX",
    "TIMER_OR_DPC_INVALID",
    "TOO_MANY_RECURSIVE_FAULTS",
    "TRAP_CAUSE_UNKNOWN",
    "TTM_FATAL_ERROR",
    "TTM_WATCHDOG_TIMEOUT",
    "UCMUCSI_FAILURE",
    "UDFS_FILE_SYSTEM",
    "UNEXPECTED_INITIALIZATION_CALL",
    "UNEXPECTED_KERNEL_MODE_TRAP",
    "UNEXPECTED_KERNEL_MODE_TRAP_M",
    "UNEXPECTED_STORE_EXCEPTION",
    "UNMOUNTABLE_BOOT_VOLUME",
    "UNSUPPORTED_INSTRUCTION_MODE",
    "UNSUPPORTED_PROCESSOR",
    "UNWIND_ON_INVALID_STACK",
    "UP_DRIVER_ON_MP_SYSTEM",
    "USER_MODE_HEALTH_MONITOR",
    "VHD_BOOT_HOST_VOLUME_NOT_ENOUGH_SPACE",
    "VHD_BOOT_INITIALIZATION_FAILED",
    "VIDEO_DRIVER_DEBUG_REPORT_REQUEST",
    "VIDEO_DRIVER_INIT_FAILURE",
    "VIDEO_DWMINIT_TIMEOUT_FALLBACK_BDD",
    "VIDEO_DXGKRNL_FATAL_ERROR",
    "VIDEO_DXGKRNL_SYSMM_FATAL_ERROR",
    "VIDEO_MEMORY_MANAGEMENT_INTERNAL",
    "VIDEO_SCHEDULER_INTERNAL_ERROR",
    "VIDEO_SHADOW_DRIVER_FATAL_ERROR",
    "VIDEO_TDR_FAILURE",
    "VIDEO_TDR_TIMEOUT_DETECTED",
    "VOLSNAP_OVERLAPPED_TABLE_ACCESS",
    "WDF_VIOLATION",
    "WFP_INVALID_OPERATION",
    "WHEA_INTERNAL_ERROR",
    "WHEA_UNCORRECTABLE_ERROR",
    "WIN32K_ATOMIC_CHECK_FAILURE",
    "WIN32K_CALLOUT_WATCHDOG_BUGCHECK",
    "WIN32K_CRITICAL_FAILURE",
    "WIN32K_HANDLE_MANAGER",
    "WIN32K_POWER_WATCHDOG_TIMEOUT",
    "WIN32K_SECURITY_FAILURE",
    "WINLOGON_FATAL_ERROR",
    "WORKER_INVALID",
    "WORKER_THREAD_INVALID_STATE",
    "WORKER_THREAD_RETURNED_AT_BAD_IRQL",
    "WORKER_THREAD_RETURNED_WHILE_ATTACHED_TO_SILO",
    "WORKER_THREAD_RETURNED_WITH_BAD_IO_PRIORITY",
    "WORKER_THREAD_RETURNED_WITH_BAD_PAGING_IO_PRIORITY",
    "WORKER_THREAD_RETURNED_WITH_NON_DEFAULT_WORKLOAD_CLASS",
    "WORKER_THREAD_RETURNED_WITH_SYSTEM_PAGE_PRIORITY_ACTIVE",
    "WORKER_THREAD_TEST_CONDITION",
    "XBOX_ERACTRL_CS_TIMEOUT",
    "XNS_INTERNAL_ERROR",
];

/// FROZEN OS / driver module-name allowlist (lowercase filenames). A crash
/// implicates one of these Windows kernel/OS/GPU-driver binaries; an arbitrary
/// `stem.exe` — a custom or in-house binary (`acmecorp_agent.dll`), a
/// user-named executable — is NOT a symptom, it is identity. Membership is the
/// allowlist; the `.exe/.dll/.sys` suffix is not. Conservative: only
/// well-known Windows/vendor modules. MUST stay sorted — enforced by test.
const MODULE_NAMES: &[&str] = &[
    "acpi.sys",
    "afd.sys",
    "atikmdag.sys",
    "audiodg.exe",
    "bthport.sys",
    "bthusb.sys",
    "classpnp.sys",
    "cng.sys",
    "combase.dll",
    "conhost.exe",
    "csrss.exe",
    "ctfmon.exe",
    "disk.sys",
    "dllhost.exe",
    "dwm.exe",
    "dxgkrnl.sys",
    "dxgmms1.sys",
    "dxgmms2.sys",
    "explorer.exe",
    "fastfat.sys",
    "fltmgr.sys",
    "fontdrvhost.exe",
    "gdi32.dll",
    "hal.dll",
    "http.sys",
    "iastor.sys",
    "iastorac.sys",
    "igdkmd64.sys",
    "kernel32.dll",
    "kernelbase.dll",
    "ksecdd.sys",
    "lsass.exe",
    "msrpc.sys",
    "msvcrt.dll",
    "ndis.sys",
    "netio.sys",
    "ntdll.dll",
    "ntfs.sys",
    "ntoskrnl.exe",
    "nvlddmkm.sys",
    "nvme.sys",
    "ole32.dll",
    "pci.sys",
    "rundll32.exe",
    "runtimebroker.exe",
    "searchindexer.exe",
    "services.exe",
    "sihost.exe",
    "smss.exe",
    "spoolsv.exe",
    "storahci.sys",
    "storport.sys",
    "svchost.exe",
    "taskhostw.exe",
    "tcpip.sys",
    "usbhub.sys",
    "usbxhci.sys",
    "user32.dll",
    "volmgr.sys",
    "volsnap.sys",
    "watchdog.sys",
    "wdf01000.sys",
    "win32k.sys",
    "win32kbase.sys",
    "win32kfull.sys",
    "wininit.exe",
    "winlogon.exe",
    "wuauclt.exe",
];

/// Extract the structured, de-identified symptoms from free text.
///
/// The result is sorted and deduplicated, so the same evidence always produces
/// the same symptom set (and therefore the same
/// [`FaultSignature`](crate::FaultSignature) fingerprint). Every token it emits
/// is a member of the closed grammar ([`is_symptom_token`]) — the extractor and
/// the grammar are the same allowlist, so a value round-trips iff the extractor
/// itself would have produced it.
pub fn extract_symptoms(text: &str) -> Vec<Symptom> {
    let lowered = text.to_lowercase();
    let mut symptoms: Vec<String> = Vec::new();
    let mut previous_kept: Option<&str> = None;

    for raw in lowered.split(|c: char| !(c.is_ascii_alphanumeric() || c == '.' || c == '_')) {
        let token = raw.trim_matches(|c| c == '.' || c == '_');
        if token.is_empty() {
            previous_kept = None;
            continue;
        }

        if is_hex_code(token) {
            symptoms.push(token.to_string());
            previous_kept = Some(token);
        } else if MODULE_NAMES.binary_search(&token).is_ok() {
            symptoms.push(token.to_string());
            previous_kept = None;
        } else if is_stop_code_token(token) {
            // A bugcheck name is a whole token in its own right; it never
            // prefixes a following number.
            symptoms.push(token.to_string());
            previous_kept = None;
        } else if VOCABULARY.binary_search(&token).is_ok() {
            symptoms.push(token.to_string());
            previous_kept = Some(token);
        } else if is_decimal_id(token) {
            // A bare number is kept only in the context of an id-bearing
            // vocabulary word; otherwise it is dropped as potential identity.
            if let Some(prefix) = previous_kept.filter(|p| ID_PREFIXES.contains(p)) {
                symptoms.push(format!("{prefix}_{token}"));
            }
            previous_kept = None;
        } else {
            previous_kept = None;
        }
    }

    symptoms.sort_unstable();
    symptoms.dedup();
    symptoms.into_iter().map(Symptom).collect()
}

/// Whether `token` is a single, canonical (lowercase) member of the closed de-id
/// symptom grammar:
///
/// `VOCABULARY ∪ 0x-hex ∪ <known-prefix>_<digits> ∪ STOP_CODE_NAMES ∪
/// MODULE_NAMES`
///
/// This is the exact set [`extract_symptoms`] can emit as a single token, so it
/// is the membership predicate the de-id symptom mint ([`crate`] re-exported to
/// `deid::symptom`) and the read-side deserialization guards check: a stored or
/// served symptom is admissible iff it is something the extractor itself would
/// have produced. An identity-shaped token (`desktop-nathan01`, an asset tag, a
/// custom binary) is not a member and is refused.
pub fn is_symptom_token(token: &str) -> bool {
    is_hex_code(token)
        || VOCABULARY.binary_search(&token).is_ok()
        || MODULE_NAMES.binary_search(&token).is_ok()
        || is_stop_code_token(token)
        || is_prefixed_id(token)
}

/// A `0x`-prefixed hex code (stop code, WER bucket, bugcheck parameter).
fn is_hex_code(token: &str) -> bool {
    token.len() > 2 && token.starts_with("0x") && token[2..].chars().all(|c| c.is_ascii_hexdigit())
}

/// Whether `token` (in the canonical lowercase symptom form) names a member of
/// the frozen [`STOP_CODE_NAMES`] dictionary. Bugcheck names are published in
/// `ALL_CAPS`; the stored/extracted symptom form is lowercase, so probe the
/// dictionary with an uppercase copy. Case-insensitive against a closed list is
/// strictly safer than the old ALL-CAPS *shape* rule — a lowercase username in a
/// path (`john_smith`) is not in the list and is refused regardless of casing.
fn is_stop_code_token(token: &str) -> bool {
    // Fast reject: bugcheck names are ASCII letters, digits, and underscores.
    if token.is_empty()
        || !token
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_')
    {
        return false;
    }
    let upper = token.to_ascii_uppercase();
    STOP_CODE_NAMES.binary_search(&upper.as_str()).is_ok()
}

/// A `<known-prefix>_<digits>` id (`event_41`, `xid_79`, `power_41`): a
/// vocabulary id-prefix, an underscore, then a short decimal id. This is the one
/// grammar member the extractor produces from TWO input tokens, so it does not
/// round-trip as a single token through `extract_symptoms` — the grammar admits
/// it directly.
fn is_prefixed_id(token: &str) -> bool {
    match token.split_once('_') {
        Some((prefix, digits)) => ID_PREFIXES.contains(&prefix) && is_decimal_id(digits),
        None => false,
    }
}

/// A short, purely decimal token — an event id, never a serial-length number.
fn is_decimal_id(token: &str) -> bool {
    (1..=6).contains(&token.len()) && token.chars().all(|c| c.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strings(text: &str) -> Vec<String> {
        extract_symptoms(text).into_iter().map(|s| s.0).collect()
    }

    #[test]
    fn vocabulary_is_sorted_for_binary_search() {
        let mut sorted = VOCABULARY.to_vec();
        sorted.sort_unstable();
        assert_eq!(VOCABULARY, sorted.as_slice());
    }

    #[test]
    fn dictionaries_are_sorted_for_binary_search() {
        // Strictly ascending, not merely `sort()`-stable: `<` also rejects a
        // DUPLICATE entry, which a `sort()`-then-equal check silently accepts
        // (and which the set-based drift test cannot see either, since a set
        // dedupes). A dup is harmless to `binary_search` but is dead weight and
        // masks a copy-paste error — reject it here (blind-audit F3, 2026-07-09).
        assert!(
            STOP_CODE_NAMES.windows(2).all(|w| w[0] < w[1]),
            "STOP_CODE_NAMES must be strictly ascending (sorted + no duplicates)"
        );
        assert!(
            MODULE_NAMES.windows(2).all(|w| w[0] < w[1]),
            "MODULE_NAMES must be strictly ascending (sorted + no duplicates)"
        );
        // The module allowlist is lowercase; the stop-code dictionary uppercase.
        for m in MODULE_NAMES {
            assert_eq!(*m, m.to_lowercase(), "module {m:?} must be lowercase");
        }
        for s in STOP_CODE_NAMES {
            assert_eq!(*s, s.to_uppercase(), "stop code {s:?} must be uppercase");
        }
    }

    /// Defense-in-depth against a FUTURE over-admission (blind-audit F1,
    /// 2026-07-09). The stop-code table (`stop_codes::STOP_CODES`) is a data
    /// catalog; the drift test only proves this de-id allowlist *mirrors* it, not
    /// that any given name is SAFE to admit as a symptom. Without an independent
    /// gate, a later edit adding a generic/vendor name to the table (a bare
    /// `PRINTNIGHTMARE`, an OEM `ACME_FLEET_WATCHDOG`) would silently widen the
    /// PII boundary with a still-green drift test. Every real Microsoft bug-check
    /// name is a long `SNAKE_CASE` kernel identifier — at least two `[A-Z0-9]+`
    /// segments joined by underscores. This asserts that shape, so a short or
    /// single-word admission fails the build regardless of the table.
    #[test]
    fn stop_code_names_have_microsoft_bugcheck_shape() {
        for name in STOP_CODE_NAMES {
            assert!(name.len() >= 8, "stop-code name {name:?} implausibly short");
            let segments: Vec<&str> = name.split('_').collect();
            assert!(
                segments.len() >= 2,
                "stop-code name {name:?} must be underscore-separated SNAKE_CASE, \
                 not a single generic word"
            );
            assert!(
                name.starts_with(|c: char| c.is_ascii_uppercase()),
                "stop-code name {name:?} must start with an uppercase letter"
            );
            for seg in segments {
                assert!(
                    !seg.is_empty()
                        && seg
                            .bytes()
                            .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit()),
                    "stop-code name {name:?} has an empty or non-[A-Z0-9] segment {seg:?}"
                );
            }
        }
    }

    /// The de-id allowlist and the authoritative [`crate::stop_codes`] table are
    /// two views of the SAME set of published bugcheck names. If they diverge,
    /// either a real crash symptom is silently dropped (name in the table but not
    /// admitted) or an unreviewed name is admitted (in the allowlist but not the
    /// table). This weld — mirroring `deid::ACTION_VOCABULARY` ↔ the dispatcher
    /// registry — makes any divergence a build failure with a pointer to the fix.
    #[test]
    fn stop_code_names_mirror_the_stop_code_table() {
        use std::collections::BTreeSet;
        let allowlist: BTreeSet<&str> = STOP_CODE_NAMES.iter().copied().collect();
        let table: BTreeSet<&str> = crate::stop_codes::names().collect();
        let missing_from_allowlist: Vec<_> = table.difference(&allowlist).collect();
        let missing_from_table: Vec<_> = allowlist.difference(&table).collect();
        assert!(
            missing_from_allowlist.is_empty() && missing_from_table.is_empty(),
            "STOP_CODE_NAMES (de-id) and stop_codes::STOP_CODES have drifted.\n  \
             in the table but NOT admitted by de-id (add to STOP_CODE_NAMES): {missing_from_allowlist:?}\n  \
             admitted by de-id but NOT in the table (add to STOP_CODES or remove here): {missing_from_table:?}"
        );
    }

    #[test]
    fn keeps_structured_evidence() {
        let symptoms = strings("explorer.exe crashes on login with WER bucket 0x1234");
        assert!(symptoms.contains(&"explorer.exe".to_string()));
        assert!(symptoms.contains(&"crashes".to_string()));
        assert!(symptoms.contains(&"login".to_string()));
        assert!(symptoms.contains(&"wer".to_string()));
        assert!(symptoms.contains(&"0x1234".to_string()));
    }

    #[test]
    fn keeps_stop_code_names_from_the_frozen_dictionary() {
        let symptoms =
            strings("blue screen said WHEA_UNCORRECTABLE_ERROR then CRITICAL_PROCESS_DIED");
        assert!(symptoms.contains(&"whea_uncorrectable_error".to_string()));
        assert!(symptoms.contains(&"critical_process_died".to_string()));
        // Two-segment bugcheck names in the dictionary qualify too.
        assert!(strings("MEMORY_MANAGEMENT bsod").contains(&"memory_management".to_string()));
        // Lowercase snake_case (a username in a path) is not in the dictionary.
        assert!(!strings("C:\\Users\\john_smith\\ntoskrnl crashed")
            .iter()
            .any(|s| s.contains("john")));
        // A made-up ALL-CAPS token is NOT a bugcheck name — the C5 fix: the old
        // shape heuristic kept any ALL_CAPS_UNDERSCORE token (asset tags, AD
        // groups). The dictionary refuses it.
        assert!(strings("asset RIG_NATHAN_DESK failed").is_empty());
        assert!(strings("group DOMAIN_ADMINS_EAST").is_empty());
    }

    #[test]
    fn keeps_only_allowlisted_module_names() {
        // A real OS/driver module survives.
        assert!(strings("fault in nvlddmkm.sys during tdr").contains(&"nvlddmkm.sys".to_string()));
        // A custom / in-house binary by shape does NOT — the C5 fix: the old
        // rule kept any `stem.exe/.dll/.sys`, so `acmecorp_agent.dll` (an
        // in-house binary name = identity) or a user-named exe rode through.
        assert!(strings("acmecorp_agent.dll loaded").is_empty());
        assert!(strings("C:\\Users\\nathan\\app.exe crashed")
            .iter()
            .all(|s| s != "app.exe"));
    }

    #[test]
    fn binds_numeric_ids_to_their_prefix() {
        let symptoms = strings("Kernel-Power event 41 after Xid 79");
        assert!(symptoms.contains(&"event_41".to_string()));
        assert!(symptoms.contains(&"xid_79".to_string()));
        assert!(symptoms.contains(&"kernel".to_string()));
        assert!(symptoms.contains(&"power".to_string()));
    }

    #[test]
    fn drops_identity_bearing_text() {
        let symptoms = strings(
            "DESKTOP-NATHAN01 crash in C:\\Users\\nathan\\AppData\\app.exe, \
             contact nathan@example.com or 192.168.1.20, serial SN12345678",
        );
        let joined = symptoms.join(" ");
        assert!(!joined.contains("nathan"), "username leaked: {joined}");
        assert!(!joined.contains("desktop"), "hostname leaked: {joined}");
        assert!(!joined.contains("example"), "email leaked: {joined}");
        assert!(!joined.contains("192"), "address leaked: {joined}");
        assert!(!joined.contains("sn12345678"), "serial leaked: {joined}");
        // The structured evidence survives extraction; the non-allowlisted
        // `app.exe` does NOT (it is a potential in-house/user binary name).
        assert!(symptoms.contains(&"crash".to_string()));
        assert!(!symptoms.contains(&"app.exe".to_string()));
    }

    #[test]
    fn bare_numbers_without_an_id_prefix_are_dropped() {
        assert!(strings("machine 1234 in room 42").is_empty());
    }

    #[test]
    fn dotted_hostnames_are_not_module_names() {
        assert!(strings("host01.corp.example.sys").is_empty());
    }

    #[test]
    fn is_symptom_token_matches_what_the_extractor_emits() {
        // Every member class round-trips through the grammar predicate.
        for token in [
            "crash",
            "0x1234",
            "event_41",
            "xid_79",
            "explorer.exe",
            "ntoskrnl.exe",
            "whea_uncorrectable_error",
            "memory_management",
        ] {
            assert!(is_symptom_token(token), "{token:?} should be a member");
        }
        // Identity shapes are refused.
        for token in [
            "desktop-nathan01",
            "nathan",
            "rig_nathan_desk",
            "acmecorp_agent.dll",
            "app.exe",
            "sn12345678",
            "boot_loop",
            "explorer.exe crashes",
        ] {
            assert!(!is_symptom_token(token), "{token:?} must not be a member");
        }
        // Everything the extractor emits is a grammar member (self-consistency).
        for s in extract_symptoms(
            "explorer.exe crashes on login, WER bucket 0x1234, Kernel-Power event 41, \
             WHEA_UNCORRECTABLE_ERROR, nvlddmkm.sys tdr",
        ) {
            assert!(
                is_symptom_token(&s.0),
                "extractor emitted a non-grammar token: {:?}",
                s.0
            );
        }
    }

    #[test]
    fn is_deterministic_and_order_independent() {
        let a = extract_symptoms("crash on boot, WHEA error");
        let b = extract_symptoms("WHEA error; boot crash");
        assert_eq!(a, b);
    }
}
