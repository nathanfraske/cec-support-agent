// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2026 The cec-support-agent authors
//! Windows bug-check (stop) code knowledge — the authoritative in-code
//! mapping from a numeric stop code to its Microsoft symbolic name and a
//! plain-English meaning.
//!
//! This is the SINGLE SOURCE OF TRUTH for stop-code names in the engine. The
//! de-identification allowlist [`crate::extract`]`::STOP_CODE_NAMES` is a mirror
//! of the [`STOP_CODES`]`[].name` set here (a drift test in `extract` enforces
//! the two never diverge), the same way `deid::ACTION_VOCABULARY` mirrors the
//! dispatcher registry.
//!
//! Code and symbolic name are verbatim from Microsoft's official *Bug Check
//! Code Reference* (updated 2025-07-24); the `meaning` is a plain-English
//! paraphrase for operator-facing readability and is NOT security-relevant
//! (it never feeds de-identification). Regenerate from
//! `docs/research/windows-stop-codes.md`, which is the reviewed source of both
//! this table and the doc.

/// One Windows bug-check (stop) code: its numeric value, Microsoft symbolic
/// name, and a plain-English meaning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StopCode {
    /// The numeric bug-check code (e.g. `0x0000000A`).
    pub code: u32,
    /// Microsoft's symbolic name, `ALL_CAPS_UNDERSCORE` (e.g.
    /// `IRQL_NOT_LESS_OR_EQUAL`). Verbatim from the official reference.
    pub name: &'static str,
    /// A plain-English paraphrase of what the crash means. Operator-facing
    /// readability only; the per-code Microsoft Learn page is authoritative for
    /// exact parameters and cause.
    pub meaning: &'static str,
}

/// All 379 Windows bug-check codes, **sorted ascending by `code`** so
/// [`by_code`] can binary-search. Generated from
/// `docs/research/windows-stop-codes.md`.
pub static STOP_CODES: &[StopCode] = &[
    StopCode { code: 0x00000001, name: "APC_INDEX_MISMATCH", meaning: "Internal Windows fault: apc index mismatch. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000002, name: "DEVICE_QUEUE_NOT_BUSY", meaning: "Internal Windows fault: device queue not busy. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000003, name: "INVALID_AFFINITY_SET", meaning: "Internal Windows fault: invalid affinity set. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000004, name: "INVALID_DATA_ACCESS_TRAP", meaning: "Internal Windows fault: invalid data access trap. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000005, name: "INVALID_PROCESS_ATTACH_ATTEMPT", meaning: "Internal Windows fault: invalid process attach attempt. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000006, name: "INVALID_PROCESS_DETACH_ATTEMPT", meaning: "Internal Windows fault: invalid process detach attempt. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000007, name: "INVALID_SOFTWARE_INTERRUPT", meaning: "Internal Windows fault: invalid software interrupt. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000008, name: "IRQL_NOT_DISPATCH_LEVEL", meaning: "A driver accessed memory at the wrong interrupt level — a driver bug." },
    StopCode { code: 0x00000009, name: "IRQL_NOT_GREATER_OR_EQUAL", meaning: "A driver accessed memory at the wrong interrupt level — a driver bug." },
    StopCode { code: 0x0000000a, name: "IRQL_NOT_LESS_OR_EQUAL", meaning: "A driver tried to access memory it shouldn't (bad driver or faulty RAM)." },
    StopCode { code: 0x0000000b, name: "NO_EXCEPTION_HANDLING_SUPPORT", meaning: "Internal Windows fault: no exception handling support. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x0000000c, name: "MAXIMUM_WAIT_OBJECTS_EXCEEDED", meaning: "Internal Windows fault: maximum wait objects exceeded. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x0000000d, name: "MUTEX_LEVEL_NUMBER_VIOLATION", meaning: "A kernel threading/synchronization fault — almost always a driver bug. (Internal.)" },
    StopCode { code: 0x0000000e, name: "NO_USER_MODE_CONTEXT", meaning: "Internal Windows fault: no user mode context. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x0000000f, name: "SPIN_LOCK_ALREADY_OWNED", meaning: "A kernel threading/synchronization fault — almost always a driver bug. (Internal.)" },
    StopCode { code: 0x00000010, name: "SPIN_LOCK_NOT_OWNED", meaning: "A kernel threading/synchronization fault — almost always a driver bug. (Internal.)" },
    StopCode { code: 0x00000011, name: "THREAD_NOT_MUTEX_OWNER", meaning: "A kernel threading/synchronization fault — almost always a driver bug. (Internal.)" },
    StopCode { code: 0x00000012, name: "TRAP_CAUSE_UNKNOWN", meaning: "Internal Windows fault: trap cause unknown. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000013, name: "EMPTY_THREAD_REAPER_LIST", meaning: "Internal Windows fault: empty thread reaper list. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000014, name: "CREATE_DELETE_LOCK_NOT_LOCKED", meaning: "Internal Windows fault: create delete lock not locked. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000015, name: "LAST_CHANCE_CALLED_FROM_KMODE", meaning: "Internal Windows fault: last chance called from kmode. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000016, name: "CID_HANDLE_CREATION", meaning: "Internal Windows fault: cid handle creation. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000017, name: "CID_HANDLE_DELETION", meaning: "Internal Windows fault: cid handle deletion. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000018, name: "REFERENCE_BY_POINTER", meaning: "Internal Windows fault: reference by pointer. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000019, name: "BAD_POOL_HEADER", meaning: "Kernel memory pool is corrupted — usually a driver, sometimes bad RAM." },
    StopCode { code: 0x0000001a, name: "MEMORY_MANAGEMENT", meaning: "The memory manager found an inconsistency — frequently failing RAM or a driver." },
    StopCode { code: 0x0000001b, name: "PFN_SHARE_COUNT", meaning: "A memory-management fault — often a driver bug or failing RAM." },
    StopCode { code: 0x0000001c, name: "PFN_REFERENCE_COUNT", meaning: "A memory-management fault — often a driver bug or failing RAM." },
    StopCode { code: 0x0000001d, name: "NO_SPIN_LOCK_AVAILABLE", meaning: "A kernel threading/synchronization fault — almost always a driver bug. (Internal.)" },
    StopCode { code: 0x0000001e, name: "KMODE_EXCEPTION_NOT_HANDLED", meaning: "A kernel component threw an error nothing handled — driver, hardware, or BIOS." },
    StopCode { code: 0x0000001f, name: "SHARED_RESOURCE_CONV_ERROR", meaning: "Internal Windows fault: shared resource conv error. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000020, name: "KERNEL_APC_PENDING_DURING_EXIT", meaning: "Internal Windows fault: kernel apc pending during exit. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000021, name: "QUOTA_UNDERFLOW", meaning: "Internal Windows fault: quota underflow. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000022, name: "FILE_SYSTEM", meaning: "Internal Windows fault: file system. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000023, name: "FAT_FILE_SYSTEM", meaning: "The FAT file-system driver hit a fatal error — often disk corruption." },
    StopCode { code: 0x00000024, name: "NTFS_FILE_SYSTEM", meaning: "The NTFS file-system driver hit a fatal error — often disk corruption or a failing drive." },
    StopCode { code: 0x00000025, name: "NPFS_FILE_SYSTEM", meaning: "The NPFS file-system driver hit a fatal error." },
    StopCode { code: 0x00000026, name: "CDFS_FILE_SYSTEM", meaning: "The CDFS file-system driver hit a fatal error." },
    StopCode { code: 0x00000027, name: "RDR_FILE_SYSTEM", meaning: "The RDR file-system driver hit a fatal error." },
    StopCode { code: 0x00000028, name: "CORRUPT_ACCESS_TOKEN", meaning: "Internal Windows fault: corrupt access token. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000029, name: "SECURITY_SYSTEM", meaning: "Internal Windows fault: security system. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x0000002a, name: "INCONSISTENT_IRP", meaning: "A kernel threading/synchronization fault — almost always a driver bug. (Internal.)" },
    StopCode { code: 0x0000002b, name: "PANIC_STACK_SWITCH", meaning: "Internal Windows fault: panic stack switch. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x0000002c, name: "PORT_DRIVER_INTERNAL", meaning: "A device-driver fault — the name points at the driver/component involved. (Driver-related.)" },
    StopCode { code: 0x0000002d, name: "SCSI_DISK_DRIVER_INTERNAL", meaning: "A device-driver fault — the name points at the driver/component involved. (Driver-related.)" },
    StopCode { code: 0x0000002e, name: "DATA_BUS_ERROR", meaning: "Internal Windows fault: data bus error. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x0000002f, name: "INSTRUCTION_BUS_ERROR", meaning: "Internal Windows fault: instruction bus error. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000030, name: "SET_OF_INVALID_CONTEXT", meaning: "Internal Windows fault: set of invalid context. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000031, name: "PHASE0_INITIALIZATION_FAILED", meaning: "A Windows internal component (Phase0) failed to initialize during startup. (Rare/internal.)" },
    StopCode { code: 0x00000032, name: "PHASE1_INITIALIZATION_FAILED", meaning: "A Windows internal component (Phase1) failed to initialize during startup. (Rare/internal.)" },
    StopCode { code: 0x00000033, name: "UNEXPECTED_INITIALIZATION_CALL", meaning: "Internal Windows fault: unexpected initialization call. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000034, name: "CACHE_MANAGER", meaning: "Internal Windows fault: cache manager. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000035, name: "NO_MORE_IRP_STACK_LOCATIONS", meaning: "A kernel threading/synchronization fault — almost always a driver bug. (Internal.)" },
    StopCode { code: 0x00000036, name: "DEVICE_REFERENCE_COUNT_NOT_ZERO", meaning: "Internal Windows fault: device reference count not zero. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000037, name: "FLOPPY_INTERNAL_ERROR", meaning: "Internal Windows fault: floppy internal error. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000038, name: "SERIAL_DRIVER_INTERNAL", meaning: "A device-driver fault — the name points at the driver/component involved. (Driver-related.)" },
    StopCode { code: 0x00000039, name: "SYSTEM_EXIT_OWNED_MUTEX", meaning: "A kernel threading/synchronization fault — almost always a driver bug. (Internal.)" },
    StopCode { code: 0x0000003a, name: "SYSTEM_UNWIND_PREVIOUS_USER", meaning: "Internal Windows fault: system unwind previous user. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x0000003b, name: "SYSTEM_SERVICE_EXCEPTION", meaning: "A crash during a system call — often a driver, anti-virus, or corrupted system file." },
    StopCode { code: 0x0000003c, name: "INTERRUPT_UNWIND_ATTEMPTED", meaning: "A memory-management fault — often a driver bug or failing RAM." },
    StopCode { code: 0x0000003d, name: "INTERRUPT_EXCEPTION_NOT_HANDLED", meaning: "Internal Windows fault: interrupt exception not handled. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x0000003e, name: "MULTIPROCESSOR_CONFIGURATION_NOT_SUPPORTED", meaning: "Internal Windows fault: multiprocessor configuration not supported. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x0000003f, name: "NO_MORE_SYSTEM_PTES", meaning: "A memory-management fault — often a driver bug or failing RAM." },
    StopCode { code: 0x00000040, name: "TARGET_MDL_TOO_SMALL", meaning: "Internal Windows fault: target mdl too small. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000041, name: "MUST_SUCCEED_POOL_EMPTY", meaning: "A memory-management fault — often a driver bug or failing RAM." },
    StopCode { code: 0x00000042, name: "ATDISK_DRIVER_INTERNAL", meaning: "A device-driver fault — the name points at the driver/component involved. (Driver-related.)" },
    StopCode { code: 0x00000043, name: "NO_SUCH_PARTITION", meaning: "Internal Windows fault: no such partition. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000044, name: "MULTIPLE_IRP_COMPLETE_REQUESTS", meaning: "A kernel threading/synchronization fault — almost always a driver bug. (Internal.)" },
    StopCode { code: 0x00000045, name: "INSUFFICIENT_SYSTEM_MAP_REGS", meaning: "Internal Windows fault: insufficient system map regs. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000046, name: "DEREF_UNKNOWN_LOGON_SESSION", meaning: "Internal Windows fault: deref unknown logon session. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000047, name: "REF_UNKNOWN_LOGON_SESSION", meaning: "Internal Windows fault: ref unknown logon session. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000048, name: "CANCEL_STATE_IN_COMPLETED_IRP", meaning: "A kernel threading/synchronization fault — almost always a driver bug. (Internal.)" },
    StopCode { code: 0x00000049, name: "PAGE_FAULT_WITH_INTERRUPTS_OFF", meaning: "A memory-management fault — often a driver bug or failing RAM." },
    StopCode { code: 0x0000004a, name: "IRQL_GT_ZERO_AT_SYSTEM_SERVICE", meaning: "A driver accessed memory at the wrong interrupt level — a driver bug." },
    StopCode { code: 0x0000004b, name: "STREAMS_INTERNAL_ERROR", meaning: "Internal Windows fault: streams internal error. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x0000004c, name: "FATAL_UNHANDLED_HARD_ERROR", meaning: "Internal Windows fault: fatal unhandled hard error. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x0000004d, name: "NO_PAGES_AVAILABLE", meaning: "Internal Windows fault: no pages available. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x0000004e, name: "PFN_LIST_CORRUPT", meaning: "The memory page tracking list is corrupted — typically failing RAM or a driver." },
    StopCode { code: 0x0000004f, name: "NDIS_INTERNAL_ERROR", meaning: "Internal Windows fault: ndis internal error. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000050, name: "PAGE_FAULT_IN_NONPAGED_AREA", meaning: "Windows referenced memory that isn't there — bad driver, anti-virus, or failing RAM." },
    StopCode { code: 0x00000051, name: "REGISTRY_ERROR", meaning: "The registry hit a fatal error — often disk corruption or a damaged hive." },
    StopCode { code: 0x00000052, name: "MAILSLOT_FILE_SYSTEM", meaning: "The MAILSLOT file-system driver hit a fatal error." },
    StopCode { code: 0x00000053, name: "NO_BOOT_DEVICE", meaning: "No bootable device was found." },
    StopCode { code: 0x00000054, name: "LM_SERVER_INTERNAL_ERROR", meaning: "Internal Windows fault: lm server internal error. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000055, name: "DATA_COHERENCY_EXCEPTION", meaning: "Internal Windows fault: data coherency exception. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000056, name: "INSTRUCTION_COHERENCY_EXCEPTION", meaning: "Internal Windows fault: instruction coherency exception. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000057, name: "XNS_INTERNAL_ERROR", meaning: "Internal Windows fault: xns internal error. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000058, name: "FTDISK_INTERNAL_ERROR", meaning: "Internal Windows fault: ftdisk internal error. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000059, name: "PINBALL_FILE_SYSTEM", meaning: "The PINBALL file-system driver hit a fatal error." },
    StopCode { code: 0x0000005a, name: "CRITICAL_SERVICE_FAILED", meaning: "Internal Windows fault: critical service failed. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x0000005b, name: "SET_ENV_VAR_FAILED", meaning: "Internal Windows fault: set env var failed. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x0000005c, name: "HAL_INITIALIZATION_FAILED", meaning: "A Windows internal component (Hal) failed to initialize during startup. (Rare/internal.)" },
    StopCode { code: 0x0000005d, name: "UNSUPPORTED_PROCESSOR", meaning: "Internal Windows fault: unsupported processor. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x0000005e, name: "OBJECT_INITIALIZATION_FAILED", meaning: "A Windows internal component (Object) failed to initialize during startup. (Rare/internal.)" },
    StopCode { code: 0x0000005f, name: "SECURITY_INITIALIZATION_FAILED", meaning: "A Windows internal component (Security) failed to initialize during startup. (Rare/internal.)" },
    StopCode { code: 0x00000060, name: "PROCESS_INITIALIZATION_FAILED", meaning: "A Windows internal component (Process) failed to initialize during startup. (Rare/internal.)" },
    StopCode { code: 0x00000061, name: "HAL1_INITIALIZATION_FAILED", meaning: "A Windows internal component (Hal1) failed to initialize during startup. (Rare/internal.)" },
    StopCode { code: 0x00000062, name: "OBJECT1_INITIALIZATION_FAILED", meaning: "A Windows internal component (Object1) failed to initialize during startup. (Rare/internal.)" },
    StopCode { code: 0x00000063, name: "SECURITY1_INITIALIZATION_FAILED", meaning: "A Windows internal component (Security1) failed to initialize during startup. (Rare/internal.)" },
    StopCode { code: 0x00000064, name: "SYMBOLIC_INITIALIZATION_FAILED", meaning: "A Windows internal component (Symbolic) failed to initialize during startup. (Rare/internal.)" },
    StopCode { code: 0x00000065, name: "MEMORY1_INITIALIZATION_FAILED", meaning: "A Windows internal component (Memory1) failed to initialize during startup. (Rare/internal.)" },
    StopCode { code: 0x00000066, name: "CACHE_INITIALIZATION_FAILED", meaning: "A Windows internal component (Cache) failed to initialize during startup. (Rare/internal.)" },
    StopCode { code: 0x00000067, name: "CONFIG_INITIALIZATION_FAILED", meaning: "A Windows internal component (Config) failed to initialize during startup. (Rare/internal.)" },
    StopCode { code: 0x00000068, name: "FILE_INITIALIZATION_FAILED", meaning: "A Windows internal component (File) failed to initialize during startup. (Rare/internal.)" },
    StopCode { code: 0x00000069, name: "IO1_INITIALIZATION_FAILED", meaning: "A Windows internal component (Io1) failed to initialize during startup. (Rare/internal.)" },
    StopCode { code: 0x0000006a, name: "LPC_INITIALIZATION_FAILED", meaning: "A Windows internal component (Lpc) failed to initialize during startup. (Rare/internal.)" },
    StopCode { code: 0x0000006b, name: "PROCESS1_INITIALIZATION_FAILED", meaning: "A Windows internal component (Process1) failed to initialize during startup. (Rare/internal.)" },
    StopCode { code: 0x0000006c, name: "REFMON_INITIALIZATION_FAILED", meaning: "A Windows internal component (Refmon) failed to initialize during startup. (Rare/internal.)" },
    StopCode { code: 0x0000006d, name: "SESSION1_INITIALIZATION_FAILED", meaning: "A Windows internal component (Session1) failed to initialize during startup. (Rare/internal.)" },
    StopCode { code: 0x0000006e, name: "SESSION2_INITIALIZATION_FAILED", meaning: "A Windows internal component (Session2) failed to initialize during startup. (Rare/internal.)" },
    StopCode { code: 0x0000006f, name: "SESSION3_INITIALIZATION_FAILED", meaning: "A Windows internal component (Session3) failed to initialize during startup. (Rare/internal.)" },
    StopCode { code: 0x00000070, name: "SESSION4_INITIALIZATION_FAILED", meaning: "A Windows internal component (Session4) failed to initialize during startup. (Rare/internal.)" },
    StopCode { code: 0x00000071, name: "SESSION5_INITIALIZATION_FAILED", meaning: "A Windows internal component (Session5) failed to initialize during startup. (Rare/internal.)" },
    StopCode { code: 0x00000072, name: "ASSIGN_DRIVE_LETTERS_FAILED", meaning: "Internal Windows fault: assign drive letters failed. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000073, name: "CONFIG_LIST_FAILED", meaning: "Internal Windows fault: config list failed. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000074, name: "BAD_SYSTEM_CONFIG_INFO", meaning: "Internal Windows fault: bad system config info. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000075, name: "CANNOT_WRITE_CONFIGURATION", meaning: "Internal Windows fault: cannot write configuration. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000076, name: "PROCESS_HAS_LOCKED_PAGES", meaning: "Internal Windows fault: process has locked pages. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000077, name: "KERNEL_STACK_INPAGE_ERROR", meaning: "Windows couldn't read the kernel stack from disk — failing drive or controller." },
    StopCode { code: 0x00000078, name: "PHASE0_EXCEPTION", meaning: "Internal Windows fault: phase0 exception. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000079, name: "MISMATCHED_HAL", meaning: "Internal Windows fault: mismatched hal. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x0000007a, name: "KERNEL_DATA_INPAGE_ERROR", meaning: "Windows couldn't read a page from disk — failing drive, cable, or RAM." },
    StopCode { code: 0x0000007b, name: "INACCESSIBLE_BOOT_DEVICE", meaning: "Windows can't reach the boot drive — controller mode change, driver, or dying disk." },
    StopCode { code: 0x0000007c, name: "BUGCODE_NDIS_DRIVER", meaning: "A device-driver fault — the name points at the driver/component involved. (Driver-related.)" },
    StopCode { code: 0x0000007d, name: "INSTALL_MORE_MEMORY", meaning: "Internal Windows fault: install more memory. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x0000007e, name: "SYSTEM_THREAD_EXCEPTION_NOT_HANDLED", meaning: "A system thread crashed unhandled — usually a driver (often names it)." },
    StopCode { code: 0x0000007f, name: "UNEXPECTED_KERNEL_MODE_TRAP", meaning: "The CPU hit an unexpected trap — hardware fault, RAM, or a driver." },
    StopCode { code: 0x00000080, name: "NMI_HARDWARE_FAILURE", meaning: "A hardware failure raised a non-maskable interrupt." },
    StopCode { code: 0x00000081, name: "SPIN_LOCK_INIT_FAILURE", meaning: "A kernel threading/synchronization fault — almost always a driver bug. (Internal.)" },
    StopCode { code: 0x00000082, name: "DFS_FILE_SYSTEM", meaning: "The DFS file-system driver hit a fatal error." },
    StopCode { code: 0x00000085, name: "SETUP_FAILURE", meaning: "Internal Windows fault: setup failure. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x0000008b, name: "MBR_CHECKSUM_MISMATCH", meaning: "Internal Windows fault: mbr checksum mismatch. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x0000008e, name: "KERNEL_MODE_EXCEPTION_NOT_HANDLED", meaning: "An unhandled kernel error — commonly a bad driver or hardware." },
    StopCode { code: 0x0000008f, name: "PP0_INITIALIZATION_FAILED", meaning: "A Windows internal component (Pp0) failed to initialize during startup. (Rare/internal.)" },
    StopCode { code: 0x00000090, name: "PP1_INITIALIZATION_FAILED", meaning: "A Windows internal component (Pp1) failed to initialize during startup. (Rare/internal.)" },
    StopCode { code: 0x00000092, name: "UP_DRIVER_ON_MP_SYSTEM", meaning: "A device-driver fault — the name points at the driver/component involved. (Driver-related.)" },
    StopCode { code: 0x00000093, name: "INVALID_KERNEL_HANDLE", meaning: "Internal Windows fault: invalid kernel handle. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000094, name: "KERNEL_STACK_LOCKED_AT_EXIT", meaning: "Internal Windows fault: kernel stack locked at exit. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000096, name: "INVALID_WORK_QUEUE_ITEM", meaning: "Internal Windows fault: invalid work queue item. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000097, name: "BOUND_IMAGE_UNSUPPORTED", meaning: "Internal Windows fault: bound image unsupported. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000098, name: "END_OF_NT_EVALUATION_PERIOD", meaning: "Internal Windows fault: end of nt evaluation period. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000099, name: "INVALID_REGION_OR_SEGMENT", meaning: "Internal Windows fault: invalid region or segment. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x0000009a, name: "SYSTEM_LICENSE_VIOLATION", meaning: "Internal Windows fault: system license violation. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x0000009b, name: "UDFS_FILE_SYSTEM", meaning: "The UDFS file-system driver hit a fatal error." },
    StopCode { code: 0x0000009c, name: "MACHINE_CHECK_EXCEPTION", meaning: "The CPU reported an unrecoverable hardware fault (heat, power, or failing part)." },
    StopCode { code: 0x0000009e, name: "USER_MODE_HEALTH_MONITOR", meaning: "Internal Windows fault: user mode health monitor. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x0000009f, name: "DRIVER_POWER_STATE_FAILURE", meaning: "A driver didn't complete a power (sleep/wake) transition in time." },
    StopCode { code: 0x000000a0, name: "INTERNAL_POWER_ERROR", meaning: "The power manager hit a fatal error — often during sleep/hibernate." },
    StopCode { code: 0x000000a1, name: "PCI_BUS_DRIVER_INTERNAL", meaning: "A device-driver fault — the name points at the driver/component involved. (Driver-related.)" },
    StopCode { code: 0x000000a2, name: "MEMORY_IMAGE_CORRUPT", meaning: "Internal Windows fault: memory image corrupt. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x000000a3, name: "ACPI_DRIVER_INTERNAL", meaning: "A device-driver fault — the name points at the driver/component involved. (Driver-related.)" },
    StopCode { code: 0x000000a4, name: "CNSS_FILE_SYSTEM_FILTER", meaning: "Internal Windows fault: cnss file system filter. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x000000a5, name: "ACPI_BIOS_ERROR", meaning: "The motherboard firmware (ACPI/BIOS) is faulty or incompatible — update the BIOS." },
    StopCode { code: 0x000000a7, name: "BAD_EXHANDLE", meaning: "Internal Windows fault: bad exhandle. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x000000ac, name: "HAL_MEMORY_ALLOCATION", meaning: "A low-level hardware-abstraction/processor init fault. (Rare/internal.)" },
    StopCode { code: 0x000000ad, name: "VIDEO_DRIVER_DEBUG_REPORT_REQUEST", meaning: "A device-driver fault — the name points at the driver/component involved. (Driver-related.)" },
    StopCode { code: 0x000000b1, name: "BGI_DETECTED_VIOLATION", meaning: "Internal Windows fault: bgi detected violation. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x000000b4, name: "VIDEO_DRIVER_INIT_FAILURE", meaning: "A device-driver fault — the name points at the driver/component involved. (Driver-related.)" },
    StopCode { code: 0x000000b8, name: "ATTEMPTED_SWITCH_FROM_DPC", meaning: "A memory-management fault — often a driver bug or failing RAM." },
    StopCode { code: 0x000000b9, name: "CHIPSET_DETECTED_ERROR", meaning: "Internal Windows fault: chipset detected error. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x000000ba, name: "SESSION_HAS_VALID_VIEWS_ON_EXIT", meaning: "Internal Windows fault: session has valid views on exit. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x000000bb, name: "NETWORK_BOOT_INITIALIZATION_FAILED", meaning: "A Windows internal component (Network Boot) failed to initialize during startup. (Rare/internal.)" },
    StopCode { code: 0x000000bc, name: "NETWORK_BOOT_DUPLICATE_ADDRESS", meaning: "Internal Windows fault: network boot duplicate address. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x000000bd, name: "INVALID_HIBERNATED_STATE", meaning: "Internal Windows fault: invalid hibernated state. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x000000be, name: "ATTEMPTED_WRITE_TO_READONLY_MEMORY", meaning: "A driver tried to write to read-only memory — a driver bug." },
    StopCode { code: 0x000000bf, name: "MUTEX_ALREADY_OWNED", meaning: "A kernel threading/synchronization fault — almost always a driver bug. (Internal.)" },
    StopCode { code: 0x000000c1, name: "SPECIAL_POOL_DETECTED_MEMORY_CORRUPTION", meaning: "A memory-management fault — often a driver bug or failing RAM." },
    StopCode { code: 0x000000c2, name: "BAD_POOL_CALLER", meaning: "A driver made an illegal memory-pool request." },
    StopCode { code: 0x000000c4, name: "DRIVER_VERIFIER_DETECTED_VIOLATION", meaning: "Driver Verifier caught a misbehaving driver (names it)." },
    StopCode { code: 0x000000c5, name: "DRIVER_CORRUPTED_EXPOOL", meaning: "A driver corrupted the kernel memory pool." },
    StopCode { code: 0x000000c6, name: "DRIVER_CAUGHT_MODIFYING_FREED_POOL", meaning: "A device-driver fault — the name points at the driver/component involved. (Driver-related.)" },
    StopCode { code: 0x000000c7, name: "TIMER_OR_DPC_INVALID", meaning: "Internal Windows fault: timer or dpc invalid. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x000000c8, name: "IRQL_UNEXPECTED_VALUE", meaning: "A driver accessed memory at the wrong interrupt level — a driver bug." },
    StopCode { code: 0x000000c9, name: "DRIVER_VERIFIER_IOMANAGER_VIOLATION", meaning: "Driver Verifier caught a driver misusing the I/O manager." },
    StopCode { code: 0x000000ca, name: "PNP_DETECTED_FATAL_ERROR", meaning: "Internal Windows fault: pnp detected fatal error. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x000000cb, name: "DRIVER_LEFT_LOCKED_PAGES_IN_PROCESS", meaning: "A device-driver fault — the name points at the driver/component involved. (Driver-related.)" },
    StopCode { code: 0x000000cc, name: "PAGE_FAULT_IN_FREED_SPECIAL_POOL", meaning: "A memory-management fault — often a driver bug or failing RAM." },
    StopCode { code: 0x000000cd, name: "PAGE_FAULT_BEYOND_END_OF_ALLOCATION", meaning: "A memory-management fault — often a driver bug or failing RAM." },
    StopCode { code: 0x000000ce, name: "DRIVER_UNLOADED_WITHOUT_CANCELLING_PENDING_OPERATIONS", meaning: "A device-driver fault — the name points at the driver/component involved. (Driver-related.)" },
    StopCode { code: 0x000000cf, name: "TERMINAL_SERVER_DRIVER_MADE_INCORRECT_MEMORY_REFERENCE", meaning: "A device-driver fault — the name points at the driver/component involved. (Driver-related.)" },
    StopCode { code: 0x000000d0, name: "DRIVER_CORRUPTED_MMPOOL", meaning: "A device-driver fault — the name points at the driver/component involved. (Driver-related.)" },
    StopCode { code: 0x000000d1, name: "DRIVER_IRQL_NOT_LESS_OR_EQUAL", meaning: "A specific driver accessed invalid memory — usually names the faulting driver." },
    StopCode { code: 0x000000d2, name: "BUGCODE_ID_DRIVER", meaning: "A device-driver fault — the name points at the driver/component involved. (Driver-related.)" },
    StopCode { code: 0x000000d3, name: "DRIVER_PORTION_MUST_BE_NONPAGED", meaning: "A device-driver fault — the name points at the driver/component involved. (Driver-related.)" },
    StopCode { code: 0x000000d4, name: "SYSTEM_SCAN_AT_RAISED_IRQL_CAUGHT_IMPROPER_DRIVER_UNLOAD", meaning: "A device-driver fault — the name points at the driver/component involved. (Driver-related.)" },
    StopCode { code: 0x000000d5, name: "DRIVER_PAGE_FAULT_IN_FREED_SPECIAL_POOL", meaning: "A device-driver fault — the name points at the driver/component involved. (Driver-related.)" },
    StopCode { code: 0x000000d6, name: "DRIVER_PAGE_FAULT_BEYOND_END_OF_ALLOCATION", meaning: "A device-driver fault — the name points at the driver/component involved. (Driver-related.)" },
    StopCode { code: 0x000000d7, name: "DRIVER_UNMAPPING_INVALID_VIEW", meaning: "A device-driver fault — the name points at the driver/component involved. (Driver-related.)" },
    StopCode { code: 0x000000d8, name: "DRIVER_USED_EXCESSIVE_PTES", meaning: "A device-driver fault — the name points at the driver/component involved. (Driver-related.)" },
    StopCode { code: 0x000000d9, name: "LOCKED_PAGES_TRACKER_CORRUPTION", meaning: "Internal Windows fault: locked pages tracker corruption. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x000000da, name: "SYSTEM_PTE_MISUSE", meaning: "A memory-management fault — often a driver bug or failing RAM." },
    StopCode { code: 0x000000db, name: "DRIVER_CORRUPTED_SYSPTES", meaning: "A device-driver fault — the name points at the driver/component involved. (Driver-related.)" },
    StopCode { code: 0x000000dc, name: "DRIVER_INVALID_STACK_ACCESS", meaning: "A device-driver fault — the name points at the driver/component involved. (Driver-related.)" },
    StopCode { code: 0x000000de, name: "POOL_CORRUPTION_IN_FILE_AREA", meaning: "A memory-management fault — often a driver bug or failing RAM." },
    StopCode { code: 0x000000df, name: "IMPERSONATING_WORKER_THREAD", meaning: "A kernel threading/synchronization fault — almost always a driver bug. (Internal.)" },
    StopCode { code: 0x000000e0, name: "ACPI_BIOS_FATAL_ERROR", meaning: "A fatal motherboard-firmware (ACPI) error — update the BIOS." },
    StopCode { code: 0x000000e1, name: "WORKER_THREAD_RETURNED_AT_BAD_IRQL", meaning: "A kernel threading/synchronization fault — almost always a driver bug. (Internal.)" },
    StopCode { code: 0x000000e2, name: "MANUALLY_INITIATED_CRASH", meaning: "A crash triggered on purpose (keyboard/debugger) — not a fault." },
    StopCode { code: 0x000000e3, name: "RESOURCE_NOT_OWNED", meaning: "Internal Windows fault: resource not owned. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x000000e4, name: "WORKER_INVALID", meaning: "Internal Windows fault: worker invalid. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x000000e6, name: "DRIVER_VERIFIER_DMA_VIOLATION", meaning: "A device-driver fault — the name points at the driver/component involved. (Driver-related.)" },
    StopCode { code: 0x000000e7, name: "INVALID_FLOATING_POINT_STATE", meaning: "Internal Windows fault: invalid floating point state. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x000000e8, name: "INVALID_CANCEL_OF_FILE_OPEN", meaning: "Internal Windows fault: invalid cancel of file open. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x000000e9, name: "ACTIVE_EX_WORKER_THREAD_TERMINATION", meaning: "A kernel threading/synchronization fault — almost always a driver bug. (Internal.)" },
    StopCode { code: 0x000000ea, name: "THREAD_STUCK_IN_DEVICE_DRIVER", meaning: "A driver looped and hung the system — often the display driver." },
    StopCode { code: 0x000000eb, name: "DIRTY_MAPPED_PAGES_CONGESTION", meaning: "Internal Windows fault: dirty mapped pages congestion. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x000000ec, name: "SESSION_HAS_VALID_SPECIAL_POOL_ON_EXIT", meaning: "A memory-management fault — often a driver bug or failing RAM." },
    StopCode { code: 0x000000ed, name: "UNMOUNTABLE_BOOT_VOLUME", meaning: "The boot volume can't be mounted — file-system corruption on the system drive." },
    StopCode { code: 0x000000ef, name: "CRITICAL_PROCESS_DIED", meaning: "A process Windows can't run without died (e.g. csrss, wininit) — often corruption." },
    StopCode { code: 0x000000f0, name: "STORAGE_MINIPORT_ERROR", meaning: "A storage controller (miniport) driver failed." },
    StopCode { code: 0x000000f1, name: "SCSI_VERIFIER_DETECTED_VIOLATION", meaning: "Internal Windows fault: scsi verifier detected violation. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x000000f2, name: "HARDWARE_INTERRUPT_STORM", meaning: "A device flooded the CPU with interrupts — a faulty device or driver." },
    StopCode { code: 0x000000f3, name: "DISORDERLY_SHUTDOWN", meaning: "Internal Windows fault: disorderly shutdown. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x000000f4, name: "CRITICAL_OBJECT_TERMINATION", meaning: "A critical system object was terminated — similar to CRITICAL_PROCESS_DIED." },
    StopCode { code: 0x000000f5, name: "FLTMGR_FILE_SYSTEM", meaning: "The file-system filter manager failed — often a filter driver (AV/backup)." },
    StopCode { code: 0x000000f6, name: "PCI_VERIFIER_DETECTED_VIOLATION", meaning: "Internal Windows fault: pci verifier detected violation. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x000000f7, name: "DRIVER_OVERRAN_STACK_BUFFER", meaning: "A driver overran a stack buffer — a driver bug (or attack)." },
    StopCode { code: 0x000000f8, name: "RAMDISK_BOOT_INITIALIZATION_FAILED", meaning: "A Windows internal component (Ramdisk Boot) failed to initialize during startup. (Rare/internal.)" },
    StopCode { code: 0x000000f9, name: "DRIVER_RETURNED_STATUS_REPARSE_FOR_VOLUME_OPEN", meaning: "A device-driver fault — the name points at the driver/component involved. (Driver-related.)" },
    StopCode { code: 0x000000fa, name: "HTTP_DRIVER_CORRUPTED", meaning: "A device-driver fault — the name points at the driver/component involved. (Driver-related.)" },
    StopCode { code: 0x000000fc, name: "ATTEMPTED_EXECUTE_OF_NOEXECUTE_MEMORY", meaning: "Code tried to run from non-executable memory — driver bug or malware." },
    StopCode { code: 0x000000fd, name: "DIRTY_NOWRITE_PAGES_CONGESTION", meaning: "Internal Windows fault: dirty nowrite pages congestion. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x000000fe, name: "BUGCODE_USB_DRIVER", meaning: "A device-driver fault — the name points at the driver/component involved. (Driver-related.)" },
    StopCode { code: 0x000000ff, name: "RESERVE_QUEUE_OVERFLOW", meaning: "Internal Windows fault: reserve queue overflow. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000100, name: "LOADER_BLOCK_MISMATCH", meaning: "Internal Windows fault: loader block mismatch. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000101, name: "CLOCK_WATCHDOG_TIMEOUT", meaning: "A CPU core stopped responding — often a CPU/hardware or firmware problem." },
    StopCode { code: 0x00000102, name: "DPC_WATCHDOG_TIMEOUT", meaning: "A deferred routine ran too long — driver or firmware issue." },
    StopCode { code: 0x00000103, name: "MUP_FILE_SYSTEM", meaning: "The MUP file-system driver hit a fatal error." },
    StopCode { code: 0x00000104, name: "AGP_INVALID_ACCESS", meaning: "A graphics/display subsystem fault — usually the GPU driver. (Video-related.)" },
    StopCode { code: 0x00000105, name: "AGP_GART_CORRUPTION", meaning: "A graphics/display subsystem fault — usually the GPU driver. (Video-related.)" },
    StopCode { code: 0x00000106, name: "AGP_ILLEGALLY_REPROGRAMMED", meaning: "A graphics/display subsystem fault — usually the GPU driver. (Video-related.)" },
    StopCode { code: 0x00000108, name: "THIRD_PARTY_FILE_SYSTEM_FAILURE", meaning: "Internal Windows fault: third party file system failure. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000109, name: "CRITICAL_STRUCTURE_CORRUPTION", meaning: "Internal Windows fault: critical structure corruption. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x0000010a, name: "APP_TAGGING_INITIALIZATION_FAILED", meaning: "A Windows internal component (App Tagging) failed to initialize during startup. (Rare/internal.)" },
    StopCode { code: 0x0000010c, name: "FSRTL_EXTRA_CREATE_PARAMETER_VIOLATION", meaning: "Internal Windows fault: fsrtl extra create parameter violation. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x0000010d, name: "WDF_VIOLATION", meaning: "Internal Windows fault: wdf violation. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x0000010e, name: "VIDEO_MEMORY_MANAGEMENT_INTERNAL", meaning: "The GPU memory manager failed — display driver or GPU fault." },
    StopCode { code: 0x0000010f, name: "RESOURCE_MANAGER_EXCEPTION_NOT_HANDLED", meaning: "Internal Windows fault: resource manager exception not handled. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000111, name: "RECURSIVE_NMI", meaning: "Internal Windows fault: recursive nmi. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000112, name: "MSRPC_STATE_VIOLATION", meaning: "Internal Windows fault: msrpc state violation. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000113, name: "VIDEO_DXGKRNL_FATAL_ERROR", meaning: "The DirectX graphics kernel failed — display driver issue." },
    StopCode { code: 0x00000114, name: "VIDEO_SHADOW_DRIVER_FATAL_ERROR", meaning: "A device-driver fault — the name points at the driver/component involved. (Driver-related.)" },
    StopCode { code: 0x00000115, name: "AGP_INTERNAL", meaning: "A graphics/display subsystem fault — usually the GPU driver. (Video-related.)" },
    StopCode { code: 0x00000116, name: "VIDEO_TDR_FAILURE", meaning: "The graphics driver stopped responding and was reset — GPU driver or overheating GPU." },
    StopCode { code: 0x00000117, name: "VIDEO_TDR_TIMEOUT_DETECTED", meaning: "The GPU didn't respond in time and was reset." },
    StopCode { code: 0x00000119, name: "VIDEO_SCHEDULER_INTERNAL_ERROR", meaning: "The GPU scheduler hit a fatal error — usually the display driver." },
    StopCode { code: 0x0000011a, name: "EM_INITIALIZATION_FAILURE", meaning: "A Windows internal component (Em) failed to initialize during startup. (Rare/internal.)" },
    StopCode { code: 0x0000011b, name: "DRIVER_RETURNED_HOLDING_CANCEL_LOCK", meaning: "A device-driver fault — the name points at the driver/component involved. (Driver-related.)" },
    StopCode { code: 0x0000011c, name: "ATTEMPTED_WRITE_TO_CM_PROTECTED_STORAGE", meaning: "A memory-management fault — often a driver bug or failing RAM." },
    StopCode { code: 0x0000011d, name: "EVENT_TRACING_FATAL_ERROR", meaning: "Internal Windows fault: event tracing fatal error. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x0000011e, name: "TOO_MANY_RECURSIVE_FAULTS", meaning: "Internal Windows fault: too many recursive faults. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x0000011f, name: "INVALID_DRIVER_HANDLE", meaning: "A device-driver fault — the name points at the driver/component involved. (Driver-related.)" },
    StopCode { code: 0x00000120, name: "BITLOCKER_FATAL_ERROR", meaning: "A fatal BitLocker drive-encryption error." },
    StopCode { code: 0x00000121, name: "DRIVER_VIOLATION", meaning: "A device-driver fault — the name points at the driver/component involved. (Driver-related.)" },
    StopCode { code: 0x00000122, name: "WHEA_INTERNAL_ERROR", meaning: "The hardware-error reporting subsystem itself failed." },
    StopCode { code: 0x00000123, name: "CRYPTO_SELF_TEST_FAILURE", meaning: "A kernel security/integrity check failed. (Security/internal.)" },
    StopCode { code: 0x00000124, name: "WHEA_UNCORRECTABLE_ERROR", meaning: "A fatal hardware error — heat, failing CPU/RAM, or unstable overclock." },
    StopCode { code: 0x00000125, name: "NMR_INVALID_STATE", meaning: "Internal Windows fault: nmr invalid state. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000126, name: "NETIO_INVALID_POOL_CALLER", meaning: "A memory-management fault — often a driver bug or failing RAM." },
    StopCode { code: 0x00000127, name: "PAGE_NOT_ZERO", meaning: "Internal Windows fault: page not zero. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000128, name: "WORKER_THREAD_RETURNED_WITH_BAD_IO_PRIORITY", meaning: "A kernel threading/synchronization fault — almost always a driver bug. (Internal.)" },
    StopCode { code: 0x00000129, name: "WORKER_THREAD_RETURNED_WITH_BAD_PAGING_IO_PRIORITY", meaning: "A kernel threading/synchronization fault — almost always a driver bug. (Internal.)" },
    StopCode { code: 0x0000012a, name: "MUI_NO_VALID_SYSTEM_LANGUAGE", meaning: "Internal Windows fault: mui no valid system language. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x0000012b, name: "FAULTY_HARDWARE_CORRUPTED_PAGE", meaning: "Hardware (usually RAM) corrupted a memory page." },
    StopCode { code: 0x0000012c, name: "EXFAT_FILE_SYSTEM", meaning: "The exFAT file-system driver hit a fatal error." },
    StopCode { code: 0x0000012d, name: "VOLSNAP_OVERLAPPED_TABLE_ACCESS", meaning: "Internal Windows fault: volsnap overlapped table access. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x0000012e, name: "INVALID_MDL_RANGE", meaning: "Internal Windows fault: invalid mdl range. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x0000012f, name: "VHD_BOOT_INITIALIZATION_FAILED", meaning: "A Windows internal component (Vhd Boot) failed to initialize during startup. (Rare/internal.)" },
    StopCode { code: 0x00000130, name: "DYNAMIC_ADD_PROCESSOR_MISMATCH", meaning: "Internal Windows fault: dynamic add processor mismatch. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000131, name: "INVALID_EXTENDED_PROCESSOR_STATE", meaning: "Internal Windows fault: invalid extended processor state. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000132, name: "RESOURCE_OWNER_POINTER_INVALID", meaning: "Internal Windows fault: resource owner pointer invalid. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000133, name: "DPC_WATCHDOG_VIOLATION", meaning: "A driver (often storage/SSD firmware) hung the CPU too long — update drivers/firmware." },
    StopCode { code: 0x00000134, name: "DRIVE_EXTENDER", meaning: "Internal Windows fault: drive extender. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000135, name: "REGISTRY_FILTER_DRIVER_EXCEPTION", meaning: "A device-driver fault — the name points at the driver/component involved. (Driver-related.)" },
    StopCode { code: 0x00000136, name: "VHD_BOOT_HOST_VOLUME_NOT_ENOUGH_SPACE", meaning: "Internal Windows fault: vhd boot host volume not enough space. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000137, name: "WIN32K_HANDLE_MANAGER", meaning: "Internal Windows fault: win32k handle manager. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000138, name: "GPIO_CONTROLLER_DRIVER_ERROR", meaning: "A device-driver fault — the name points at the driver/component involved. (Driver-related.)" },
    StopCode { code: 0x00000139, name: "KERNEL_SECURITY_CHECK_FAILURE", meaning: "Windows detected a corrupted data structure — driver bug or memory corruption." },
    StopCode { code: 0x0000013a, name: "KERNEL_MODE_HEAP_CORRUPTION", meaning: "The kernel heap is corrupted — almost always a driver bug." },
    StopCode { code: 0x0000013b, name: "PASSIVE_INTERRUPT_ERROR", meaning: "Internal Windows fault: passive interrupt error. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x0000013c, name: "INVALID_IO_BOOST_STATE", meaning: "Internal Windows fault: invalid io boost state. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x0000013d, name: "CRITICAL_INITIALIZATION_FAILURE", meaning: "A Windows internal component (Critical) failed to initialize during startup. (Rare/internal.)" },
    StopCode { code: 0x00000140, name: "STORAGE_DEVICE_ABNORMALITY_DETECTED", meaning: "The storage stack detected an abnormal drive condition." },
    StopCode { code: 0x00000143, name: "PROCESSOR_DRIVER_INTERNAL", meaning: "A device-driver fault — the name points at the driver/component involved. (Driver-related.)" },
    StopCode { code: 0x00000144, name: "BUGCODE_USB3_DRIVER", meaning: "A device-driver fault — the name points at the driver/component involved. (Driver-related.)" },
    StopCode { code: 0x00000145, name: "SECURE_BOOT_VIOLATION", meaning: "A Secure Boot policy check failed — boot integrity problem." },
    StopCode { code: 0x00000147, name: "ABNORMAL_RESET_DETECTED", meaning: "Internal Windows fault: abnormal reset detected. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000149, name: "REFS_FILE_SYSTEM", meaning: "The ReFS file-system driver hit a fatal error." },
    StopCode { code: 0x0000014a, name: "KERNEL_WMI_INTERNAL", meaning: "Internal Windows fault: kernel wmi internal. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x0000014b, name: "SOC_SUBSYSTEM_FAILURE", meaning: "Internal Windows fault: soc subsystem failure. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x0000014c, name: "FATAL_ABNORMAL_RESET_ERROR", meaning: "Internal Windows fault: fatal abnormal reset error. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x0000014d, name: "EXCEPTION_SCOPE_INVALID", meaning: "Internal Windows fault: exception scope invalid. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x0000014e, name: "SOC_CRITICAL_DEVICE_REMOVED", meaning: "Internal Windows fault: soc critical device removed. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x0000014f, name: "PDC_WATCHDOG_TIMEOUT", meaning: "A watchdog timer expired — something ran too long or a component hung. (Often driver/firmware.)" },
    StopCode { code: 0x00000150, name: "TCPIP_AOAC_NIC_ACTIVE_REFERENCE_LEAK", meaning: "Internal Windows fault: tcpip aoac nic active reference leak. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000151, name: "UNSUPPORTED_INSTRUCTION_MODE", meaning: "Internal Windows fault: unsupported instruction mode. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000152, name: "INVALID_PUSH_LOCK_FLAGS", meaning: "Internal Windows fault: invalid push lock flags. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000153, name: "KERNEL_LOCK_ENTRY_LEAKED_ON_THREAD_TERMINATION", meaning: "Internal Windows fault: kernel lock entry leaked on thread termination. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000154, name: "UNEXPECTED_STORE_EXCEPTION", meaning: "An unexpected error while accessing the compressed-memory store — often failing RAM." },
    StopCode { code: 0x00000155, name: "OS_DATA_TAMPERING", meaning: "Internal Windows fault: os data tampering. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000157, name: "KERNEL_THREAD_PRIORITY_FLOOR_VIOLATION", meaning: "Internal Windows fault: kernel thread priority floor violation. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000158, name: "ILLEGAL_IOMMU_PAGE_FAULT", meaning: "Internal Windows fault: illegal iommu page fault. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000159, name: "HAL_ILLEGAL_IOMMU_PAGE_FAULT", meaning: "A low-level hardware-abstraction/processor init fault. (Rare/internal.)" },
    StopCode { code: 0x0000015a, name: "SDBUS_INTERNAL_ERROR", meaning: "Internal Windows fault: sdbus internal error. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x0000015b, name: "WORKER_THREAD_RETURNED_WITH_SYSTEM_PAGE_PRIORITY_ACTIVE", meaning: "A kernel threading/synchronization fault — almost always a driver bug. (Internal.)" },
    StopCode { code: 0x00000160, name: "WIN32K_ATOMIC_CHECK_FAILURE", meaning: "Internal Windows fault: win32k atomic check failure. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000162, name: "KERNEL_AUTO_BOOST_INVALID_LOCK_RELEASE", meaning: "Internal Windows fault: kernel auto boost invalid lock release. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000163, name: "WORKER_THREAD_TEST_CONDITION", meaning: "A kernel threading/synchronization fault — almost always a driver bug. (Internal.)" },
    StopCode { code: 0x00000164, name: "WIN32K_CRITICAL_FAILURE", meaning: "Internal Windows fault: win32k critical failure. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x0000016c, name: "INVALID_RUNDOWN_PROTECTION_FLAGS", meaning: "Internal Windows fault: invalid rundown protection flags. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x0000016d, name: "INVALID_SLOT_ALLOCATOR_FLAGS", meaning: "Internal Windows fault: invalid slot allocator flags. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x0000016e, name: "ERESOURCE_INVALID_RELEASE", meaning: "Internal Windows fault: eresource invalid release. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000170, name: "CLUSTER_CSV_CLUSSVC_DISCONNECT_WATCHDOG", meaning: "Internal Windows fault: cluster csv clussvc disconnect watchdog. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000171, name: "CRYPTO_LIBRARY_INTERNAL_ERROR", meaning: "A kernel security/integrity check failed. (Security/internal.)" },
    StopCode { code: 0x00000173, name: "COREMSGCALL_INTERNAL_ERROR", meaning: "Internal Windows fault: coremsgcall internal error. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000174, name: "COREMSG_INTERNAL_ERROR", meaning: "Internal Windows fault: coremsg internal error. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000178, name: "ELAM_DRIVER_DETECTED_FATAL_ERROR", meaning: "A device-driver fault — the name points at the driver/component involved. (Driver-related.)" },
    StopCode { code: 0x0000017b, name: "PROFILER_CONFIGURATION_ILLEGAL", meaning: "Internal Windows fault: profiler configuration illegal. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x0000017e, name: "MICROCODE_REVISION_MISMATCH", meaning: "The CPU microcode revision doesn't match — firmware/OS mismatch." },
    StopCode { code: 0x00000187, name: "VIDEO_DWMINIT_TIMEOUT_FALLBACK_BDD", meaning: "A graphics/display subsystem fault — usually the GPU driver. (Video-related.)" },
    StopCode { code: 0x00000189, name: "BAD_OBJECT_HEADER", meaning: "Internal Windows fault: bad object header. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x0000018b, name: "SECURE_KERNEL_ERROR", meaning: "A kernel security/integrity check failed. (Security/internal.)" },
    StopCode { code: 0x0000018c, name: "HYPERGUARD_VIOLATION", meaning: "A kernel security/integrity check failed. (Security/internal.)" },
    StopCode { code: 0x0000018d, name: "SECURE_FAULT_UNHANDLED", meaning: "A kernel security/integrity check failed. (Security/internal.)" },
    StopCode { code: 0x0000018e, name: "KERNEL_PARTITION_REFERENCE_VIOLATION", meaning: "Internal Windows fault: kernel partition reference violation. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000191, name: "PF_DETECTED_CORRUPTION", meaning: "Internal Windows fault: pf detected corruption. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000192, name: "KERNEL_AUTO_BOOST_LOCK_ACQUISITION_WITH_RAISED_IRQL", meaning: "Internal Windows fault: kernel auto boost lock acquisition with raised irql. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000196, name: "LOADER_ROLLBACK_DETECTED", meaning: "Internal Windows fault: loader rollback detected. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000197, name: "WIN32K_SECURITY_FAILURE", meaning: "Internal Windows fault: win32k security failure. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000199, name: "KERNEL_STORAGE_SLOT_IN_USE", meaning: "Internal Windows fault: kernel storage slot in use. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x0000019a, name: "WORKER_THREAD_RETURNED_WHILE_ATTACHED_TO_SILO", meaning: "A kernel threading/synchronization fault — almost always a driver bug. (Internal.)" },
    StopCode { code: 0x0000019b, name: "TTM_FATAL_ERROR", meaning: "Internal Windows fault: ttm fatal error. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x0000019c, name: "WIN32K_POWER_WATCHDOG_TIMEOUT", meaning: "A watchdog timer expired — something ran too long or a component hung. (Often driver/firmware.)" },
    StopCode { code: 0x000001a0, name: "TTM_WATCHDOG_TIMEOUT", meaning: "A watchdog timer expired — something ran too long or a component hung. (Often driver/firmware.)" },
    StopCode { code: 0x000001a2, name: "WIN32K_CALLOUT_WATCHDOG_BUGCHECK", meaning: "A watchdog timer expired — something ran too long or a component hung. (Often driver/firmware.)" },
    StopCode { code: 0x000001aa, name: "EXCEPTION_ON_INVALID_STACK", meaning: "Internal Windows fault: exception on invalid stack. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x000001ab, name: "UNWIND_ON_INVALID_STACK", meaning: "Internal Windows fault: unwind on invalid stack. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x000001c6, name: "FAST_ERESOURCE_PRECONDITION_VIOLATION", meaning: "Internal Windows fault: fast eresource precondition violation. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x000001c7, name: "STORE_DATA_STRUCTURE_CORRUPTION", meaning: "Internal Windows fault: store data structure corruption. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x000001c8, name: "MANUALLY_INITIATED_POWER_BUTTON_HOLD", meaning: "Crash captured from a forced power-button hold — not a software fault." },
    StopCode { code: 0x000001ca, name: "SYNTHETIC_WATCHDOG_TIMEOUT", meaning: "A watchdog timer expired — something ran too long or a component hung. (Often driver/firmware.)" },
    StopCode { code: 0x000001cb, name: "INVALID_SILO_DETACH", meaning: "Internal Windows fault: invalid silo detach. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x000001cd, name: "INVALID_CALLBACK_STACK_ADDRESS", meaning: "Internal Windows fault: invalid callback stack address. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x000001ce, name: "INVALID_KERNEL_STACK_ADDRESS", meaning: "Internal Windows fault: invalid kernel stack address. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x000001cf, name: "HARDWARE_WATCHDOG_TIMEOUT", meaning: "A watchdog timer expired — something ran too long or a component hung. (Often driver/firmware.)" },
    StopCode { code: 0x000001d0, name: "ACPI_FIRMWARE_WATCHDOG_TIMEOUT", meaning: "A watchdog timer expired — something ran too long or a component hung. (Often driver/firmware.)" },
    StopCode { code: 0x000001d2, name: "WORKER_THREAD_INVALID_STATE", meaning: "A kernel threading/synchronization fault — almost always a driver bug. (Internal.)" },
    StopCode { code: 0x000001d3, name: "WFP_INVALID_OPERATION", meaning: "Internal Windows fault: wfp invalid operation. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x000001d5, name: "DRIVER_PNP_WATCHDOG", meaning: "A device-driver fault — the name points at the driver/component involved. (Driver-related.)" },
    StopCode { code: 0x000001d6, name: "WORKER_THREAD_RETURNED_WITH_NON_DEFAULT_WORKLOAD_CLASS", meaning: "A kernel threading/synchronization fault — almost always a driver bug. (Internal.)" },
    StopCode { code: 0x000001d7, name: "EFS_FATAL_ERROR", meaning: "Internal Windows fault: efs fatal error. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x000001d8, name: "UCMUCSI_FAILURE", meaning: "Internal Windows fault: ucmucsi failure. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x000001d9, name: "HAL_IOMMU_INTERNAL_ERROR", meaning: "A low-level hardware-abstraction/processor init fault. (Rare/internal.)" },
    StopCode { code: 0x000001da, name: "HAL_BLOCKED_PROCESSOR_INTERNAL_ERROR", meaning: "A low-level hardware-abstraction/processor init fault. (Rare/internal.)" },
    StopCode { code: 0x000001db, name: "IPI_WATCHDOG_TIMEOUT", meaning: "A watchdog timer expired — something ran too long or a component hung. (Often driver/firmware.)" },
    StopCode { code: 0x000001dc, name: "DMA_COMMON_BUFFER_VECTOR_ERROR", meaning: "Internal Windows fault: dma common buffer vector error. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x000001dd, name: "BUGCODE_MBBADAPTER_DRIVER", meaning: "A device-driver fault — the name points at the driver/component involved. (Driver-related.)" },
    StopCode { code: 0x000001de, name: "BUGCODE_WIFIADAPTER_DRIVER", meaning: "A device-driver fault — the name points at the driver/component involved. (Driver-related.)" },
    StopCode { code: 0x000001df, name: "PROCESSOR_START_TIMEOUT", meaning: "Internal Windows fault: processor start timeout. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x000001e4, name: "VIDEO_DXGKRNL_SYSMM_FATAL_ERROR", meaning: "A graphics/display subsystem fault — usually the GPU driver. (Video-related.)" },
    StopCode { code: 0x000001e9, name: "ILLEGAL_ATS_INITIALIZATION", meaning: "Internal Windows fault: illegal ats initialization. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x000001ea, name: "SECURE_PCI_CONFIG_SPACE_ACCESS_VIOLATION", meaning: "A kernel security/integrity check failed. (Security/internal.)" },
    StopCode { code: 0x000001eb, name: "DAM_WATCHDOG_TIMEOUT", meaning: "A watchdog timer expired — something ran too long or a component hung. (Often driver/firmware.)" },
    StopCode { code: 0x000001ed, name: "HANDLE_ERROR_ON_CRITICAL_THREAD", meaning: "Internal Windows fault: handle error on critical thread. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x000001f1, name: "KASAN_ENLIGHTENMENT_VIOLATION", meaning: "A kernel security/integrity check failed. (Security/internal.)" },
    StopCode { code: 0x000001f2, name: "KASAN_ILLEGAL_ACCESS", meaning: "A kernel security/integrity check failed. (Security/internal.)" },
    StopCode { code: 0x00000356, name: "XBOX_ERACTRL_CS_TIMEOUT", meaning: "Internal Windows fault: xbox eractrl cs timeout. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000bfe, name: "BC_BLUETOOTH_VERIFIER_FAULT", meaning: "Internal Windows fault: bc bluetooth verifier fault. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00000bff, name: "BC_BTHMINI_VERIFIER_FAULT", meaning: "Internal Windows fault: bc bthmini verifier fault. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x00020001, name: "HYPERVISOR_ERROR", meaning: "The Hyper-V hypervisor hit a fatal error." },
    StopCode { code: 0x1000007e, name: "SYSTEM_THREAD_EXCEPTION_NOT_HANDLED_M", meaning: "Internal Windows fault: system thread exception not handled m. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x1000007f, name: "UNEXPECTED_KERNEL_MODE_TRAP_M", meaning: "Internal Windows fault: unexpected kernel mode trap m. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x1000008e, name: "KERNEL_MODE_EXCEPTION_NOT_HANDLED_M", meaning: "Internal Windows fault: kernel mode exception not handled m. (Rare; see the per-code Microsoft page for detail.)" },
    StopCode { code: 0x100000ea, name: "THREAD_STUCK_IN_DEVICE_DRIVER_M", meaning: "A device-driver fault — the name points at the driver/component involved. (Driver-related.)" },
    StopCode { code: 0x4000008a, name: "THREAD_TERMINATE_HELD_MUTEX", meaning: "A kernel threading/synchronization fault — almost always a driver bug. (Internal.)" },
    StopCode { code: 0xc0000218, name: "STATUS_CANNOT_LOAD_REGISTRY_FILE", meaning: "Windows couldn't load a registry hive — corruption or disk failure." },
    StopCode { code: 0xc000021a, name: "WINLOGON_FATAL_ERROR", meaning: "The Winlogon logon process failed fatally." },
    StopCode { code: 0xc0000221, name: "STATUS_IMAGE_CHECKSUM_MISMATCH", meaning: "A system file failed its checksum — a corrupted or damaged driver/DLL." },
    StopCode { code: 0xdeaddead, name: "MANUALLY_INITIATED_CRASH1", meaning: "A crash triggered on purpose for testing — not a fault." },
];

/// Look up a stop code by its numeric value (`0x0000000A` → `IRQL_NOT_LESS_OR_EQUAL`).
///
/// `O(log n)` binary search over the code-sorted [`STOP_CODES`] table.
pub fn by_code(code: u32) -> Option<&'static StopCode> {
    STOP_CODES
        .binary_search_by(|e| e.code.cmp(&code))
        .ok()
        .map(|i| &STOP_CODES[i])
}

/// Look up a stop code by its symbolic name, case-insensitively
/// (`"irql_not_less_or_equal"` and `"IRQL_NOT_LESS_OR_EQUAL"` both resolve).
pub fn by_name(name: &str) -> Option<&'static StopCode> {
    STOP_CODES
        .iter()
        .find(|e| e.name.eq_ignore_ascii_case(name))
}

/// Resolve a free-form token from a crash report to its stop code, accepting
/// any of the forms a user or log actually writes:
///
/// - a hex literal, padded or not, any case: `"0x0000000A"`, `"0xA"`, `"0xa"`
/// - a bare decimal: `"10"`
/// - a symbolic name, any case: `"IRQL_NOT_LESS_OR_EQUAL"`
///
/// Returns `None` if the token is not a recognized code or name.
pub fn describe(token: &str) -> Option<&'static StopCode> {
    let t = token.trim();
    if let Some(hex) = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")) {
        return u32::from_str_radix(hex, 16).ok().and_then(by_code);
    }
    if !t.is_empty() && t.bytes().all(|b| b.is_ascii_digit()) {
        return t.parse::<u32>().ok().and_then(by_code);
    }
    by_name(t)
}

/// The set of symbolic names in [`STOP_CODES`], for the de-id drift test in
/// [`crate::extract`] to prove its allowlist mirrors this table exactly. Kept
/// `pub(crate)` because the allowlist relationship is an internal invariant.
#[cfg(test)]
pub(crate) fn names() -> impl Iterator<Item = &'static str> {
    STOP_CODES.iter().map(|e| e.name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_is_sorted_unique_by_code() {
        for w in STOP_CODES.windows(2) {
            assert!(
                w[0].code < w[1].code,
                "STOP_CODES must be strictly ascending by code: {:#010x} then {:#010x}",
                w[0].code,
                w[1].code
            );
        }
    }

    #[test]
    fn names_are_unique_and_canonical() {
        let mut seen = std::collections::HashSet::new();
        for e in STOP_CODES {
            assert!(seen.insert(e.name), "duplicate stop-code name {:?}", e.name);
            assert!(
                !e.name.is_empty()
                    && e.name
                        .bytes()
                        .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'_'),
                "stop-code name {:?} must be ALL_CAPS [A-Z0-9_]",
                e.name
            );
            assert!(
                !e.meaning.trim().is_empty(),
                "stop code {:?} has no meaning",
                e.name
            );
        }
    }

    #[test]
    fn lookup_round_trips() {
        let e = by_code(0x0000_000A).expect("0xA present");
        assert_eq!(e.name, "IRQL_NOT_LESS_OR_EQUAL");
        assert_eq!(by_name("irql_not_less_or_equal"), Some(e));
        assert_eq!(describe("0x0000000A"), Some(e));
        assert_eq!(describe("0xA"), Some(e));
        assert_eq!(describe("0xa"), Some(e));
        assert_eq!(describe("10"), Some(e));
        assert_eq!(describe("IRQL_NOT_LESS_OR_EQUAL"), Some(e));
    }

    #[test]
    fn unknown_tokens_resolve_to_none() {
        assert_eq!(by_code(0xFFFF_0000), None);
        assert_eq!(by_name("NOT_A_REAL_BUGCHECK"), None);
        assert_eq!(describe("hello"), None);
        assert_eq!(describe(""), None);
        assert_eq!(describe("0xZZZ"), None);
    }
}
