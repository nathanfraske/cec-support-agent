<!-- Source: Microsoft Learn — Bug Check Code Reference (bug-check-code-reference2), updated 2025-07-24. -->
<!-- Code+name are verbatim from the official table (two markdown artifacts corrected). Plain-English column is a PARAPHRASE from the names + domain knowledge, NOT lifted from the official per-code pages — glance-check advised; the per-code MS Learn page is authoritative for exact cause. -->
# Windows bug-check (stop) codes — complete reference

**379 codes.** Complete code→name enumeration from Microsoft's official *Bug Check Code Reference* (the list WinDbg `!analyze` uses), plus a plain-English meaning per code. Code and name are verbatim from the source; the **Plain-English** column is a paraphrase (from the names + known causes) for readability — the per-code Microsoft Learn page (`bug-check-0x<code>--<name>`) is authoritative for exact parameters and cause. Source: <https://learn.microsoft.com/en-us/windows-hardware/drivers/debugger/bug-check-code-reference2> (updated 2025-07-24).

| Hex code | Symbolic name | Plain-English meaning |
|---|---|---|
| `0x00000001` | APC_INDEX_MISMATCH | Internal Windows fault: apc index mismatch. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000002` | DEVICE_QUEUE_NOT_BUSY | Internal Windows fault: device queue not busy. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000003` | INVALID_AFFINITY_SET | Internal Windows fault: invalid affinity set. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000004` | INVALID_DATA_ACCESS_TRAP | Internal Windows fault: invalid data access trap. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000005` | INVALID_PROCESS_ATTACH_ATTEMPT | Internal Windows fault: invalid process attach attempt. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000006` | INVALID_PROCESS_DETACH_ATTEMPT | Internal Windows fault: invalid process detach attempt. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000007` | INVALID_SOFTWARE_INTERRUPT | Internal Windows fault: invalid software interrupt. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000008` | IRQL_NOT_DISPATCH_LEVEL | A driver accessed memory at the wrong interrupt level — a driver bug. |
| `0x00000009` | IRQL_NOT_GREATER_OR_EQUAL | A driver accessed memory at the wrong interrupt level — a driver bug. |
| `0x0000000A` | IRQL_NOT_LESS_OR_EQUAL | A driver tried to access memory it shouldn't (bad driver or faulty RAM). |
| `0x0000000B` | NO_EXCEPTION_HANDLING_SUPPORT | Internal Windows fault: no exception handling support. (Rare; see the per-code Microsoft page for detail.) |
| `0x0000000C` | MAXIMUM_WAIT_OBJECTS_EXCEEDED | Internal Windows fault: maximum wait objects exceeded. (Rare; see the per-code Microsoft page for detail.) |
| `0x0000000D` | MUTEX_LEVEL_NUMBER_VIOLATION | A kernel threading/synchronization fault — almost always a driver bug. (Internal.) |
| `0x0000000E` | NO_USER_MODE_CONTEXT | Internal Windows fault: no user mode context. (Rare; see the per-code Microsoft page for detail.) |
| `0x0000000F` | SPIN_LOCK_ALREADY_OWNED | A kernel threading/synchronization fault — almost always a driver bug. (Internal.) |
| `0x00000010` | SPIN_LOCK_NOT_OWNED | A kernel threading/synchronization fault — almost always a driver bug. (Internal.) |
| `0x00000011` | THREAD_NOT_MUTEX_OWNER | A kernel threading/synchronization fault — almost always a driver bug. (Internal.) |
| `0x00000012` | TRAP_CAUSE_UNKNOWN | Internal Windows fault: trap cause unknown. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000013` | EMPTY_THREAD_REAPER_LIST | Internal Windows fault: empty thread reaper list. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000014` | CREATE_DELETE_LOCK_NOT_LOCKED | Internal Windows fault: create delete lock not locked. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000015` | LAST_CHANCE_CALLED_FROM_KMODE | Internal Windows fault: last chance called from kmode. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000016` | CID_HANDLE_CREATION | Internal Windows fault: cid handle creation. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000017` | CID_HANDLE_DELETION | Internal Windows fault: cid handle deletion. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000018` | REFERENCE_BY_POINTER | Internal Windows fault: reference by pointer. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000019` | BAD_POOL_HEADER | Kernel memory pool is corrupted — usually a driver, sometimes bad RAM. |
| `0x0000001A` | MEMORY_MANAGEMENT | The memory manager found an inconsistency — frequently failing RAM or a driver. |
| `0x0000001B` | PFN_SHARE_COUNT | A memory-management fault — often a driver bug or failing RAM. |
| `0x0000001C` | PFN_REFERENCE_COUNT | A memory-management fault — often a driver bug or failing RAM. |
| `0x0000001D` | NO_SPIN_LOCK_AVAILABLE | A kernel threading/synchronization fault — almost always a driver bug. (Internal.) |
| `0x0000001E` | KMODE_EXCEPTION_NOT_HANDLED | A kernel component threw an error nothing handled — driver, hardware, or BIOS. |
| `0x0000001F` | SHARED_RESOURCE_CONV_ERROR | Internal Windows fault: shared resource conv error. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000020` | KERNEL_APC_PENDING_DURING_EXIT | Internal Windows fault: kernel apc pending during exit. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000021` | QUOTA_UNDERFLOW | Internal Windows fault: quota underflow. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000022` | FILE_SYSTEM | Internal Windows fault: file system. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000023` | FAT_FILE_SYSTEM | The FAT file-system driver hit a fatal error — often disk corruption. |
| `0x00000024` | NTFS_FILE_SYSTEM | The NTFS file-system driver hit a fatal error — often disk corruption or a failing drive. |
| `0x00000025` | NPFS_FILE_SYSTEM | The NPFS file-system driver hit a fatal error. |
| `0x00000026` | CDFS_FILE_SYSTEM | The CDFS file-system driver hit a fatal error. |
| `0x00000027` | RDR_FILE_SYSTEM | The RDR file-system driver hit a fatal error. |
| `0x00000028` | CORRUPT_ACCESS_TOKEN | Internal Windows fault: corrupt access token. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000029` | SECURITY_SYSTEM | Internal Windows fault: security system. (Rare; see the per-code Microsoft page for detail.) |
| `0x0000002A` | INCONSISTENT_IRP | A kernel threading/synchronization fault — almost always a driver bug. (Internal.) |
| `0x0000002B` | PANIC_STACK_SWITCH | Internal Windows fault: panic stack switch. (Rare; see the per-code Microsoft page for detail.) |
| `0x0000002C` | PORT_DRIVER_INTERNAL | A device-driver fault — the name points at the driver/component involved. (Driver-related.) |
| `0x0000002D` | SCSI_DISK_DRIVER_INTERNAL | A device-driver fault — the name points at the driver/component involved. (Driver-related.) |
| `0x0000002E` | DATA_BUS_ERROR | Internal Windows fault: data bus error. (Rare; see the per-code Microsoft page for detail.) |
| `0x0000002F` | INSTRUCTION_BUS_ERROR | Internal Windows fault: instruction bus error. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000030` | SET_OF_INVALID_CONTEXT | Internal Windows fault: set of invalid context. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000031` | PHASE0_INITIALIZATION_FAILED | A Windows internal component (Phase0) failed to initialize during startup. (Rare/internal.) |
| `0x00000032` | PHASE1_INITIALIZATION_FAILED | A Windows internal component (Phase1) failed to initialize during startup. (Rare/internal.) |
| `0x00000033` | UNEXPECTED_INITIALIZATION_CALL | Internal Windows fault: unexpected initialization call. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000034` | CACHE_MANAGER | Internal Windows fault: cache manager. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000035` | NO_MORE_IRP_STACK_LOCATIONS | A kernel threading/synchronization fault — almost always a driver bug. (Internal.) |
| `0x00000036` | DEVICE_REFERENCE_COUNT_NOT_ZERO | Internal Windows fault: device reference count not zero. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000037` | FLOPPY_INTERNAL_ERROR | Internal Windows fault: floppy internal error. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000038` | SERIAL_DRIVER_INTERNAL | A device-driver fault — the name points at the driver/component involved. (Driver-related.) |
| `0x00000039` | SYSTEM_EXIT_OWNED_MUTEX | A kernel threading/synchronization fault — almost always a driver bug. (Internal.) |
| `0x0000003A` | SYSTEM_UNWIND_PREVIOUS_USER | Internal Windows fault: system unwind previous user. (Rare; see the per-code Microsoft page for detail.) |
| `0x0000003B` | SYSTEM_SERVICE_EXCEPTION | A crash during a system call — often a driver, anti-virus, or corrupted system file. |
| `0x0000003C` | INTERRUPT_UNWIND_ATTEMPTED | A memory-management fault — often a driver bug or failing RAM. |
| `0x0000003D` | INTERRUPT_EXCEPTION_NOT_HANDLED | Internal Windows fault: interrupt exception not handled. (Rare; see the per-code Microsoft page for detail.) |
| `0x0000003E` | MULTIPROCESSOR_CONFIGURATION_NOT_SUPPORTED | Internal Windows fault: multiprocessor configuration not supported. (Rare; see the per-code Microsoft page for detail.) |
| `0x0000003F` | NO_MORE_SYSTEM_PTES | A memory-management fault — often a driver bug or failing RAM. |
| `0x00000040` | TARGET_MDL_TOO_SMALL | Internal Windows fault: target mdl too small. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000041` | MUST_SUCCEED_POOL_EMPTY | A memory-management fault — often a driver bug or failing RAM. |
| `0x00000042` | ATDISK_DRIVER_INTERNAL | A device-driver fault — the name points at the driver/component involved. (Driver-related.) |
| `0x00000043` | NO_SUCH_PARTITION | Internal Windows fault: no such partition. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000044` | MULTIPLE_IRP_COMPLETE_REQUESTS | A kernel threading/synchronization fault — almost always a driver bug. (Internal.) |
| `0x00000045` | INSUFFICIENT_SYSTEM_MAP_REGS | Internal Windows fault: insufficient system map regs. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000046` | DEREF_UNKNOWN_LOGON_SESSION | Internal Windows fault: deref unknown logon session. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000047` | REF_UNKNOWN_LOGON_SESSION | Internal Windows fault: ref unknown logon session. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000048` | CANCEL_STATE_IN_COMPLETED_IRP | A kernel threading/synchronization fault — almost always a driver bug. (Internal.) |
| `0x00000049` | PAGE_FAULT_WITH_INTERRUPTS_OFF | A memory-management fault — often a driver bug or failing RAM. |
| `0x0000004A` | IRQL_GT_ZERO_AT_SYSTEM_SERVICE | A driver accessed memory at the wrong interrupt level — a driver bug. |
| `0x0000004B` | STREAMS_INTERNAL_ERROR | Internal Windows fault: streams internal error. (Rare; see the per-code Microsoft page for detail.) |
| `0x0000004C` | FATAL_UNHANDLED_HARD_ERROR | Internal Windows fault: fatal unhandled hard error. (Rare; see the per-code Microsoft page for detail.) |
| `0x0000004D` | NO_PAGES_AVAILABLE | Internal Windows fault: no pages available. (Rare; see the per-code Microsoft page for detail.) |
| `0x0000004E` | PFN_LIST_CORRUPT | The memory page tracking list is corrupted — typically failing RAM or a driver. |
| `0x0000004F` | NDIS_INTERNAL_ERROR | Internal Windows fault: ndis internal error. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000050` | PAGE_FAULT_IN_NONPAGED_AREA | Windows referenced memory that isn't there — bad driver, anti-virus, or failing RAM. |
| `0x00000051` | REGISTRY_ERROR | The registry hit a fatal error — often disk corruption or a damaged hive. |
| `0x00000052` | MAILSLOT_FILE_SYSTEM | The MAILSLOT file-system driver hit a fatal error. |
| `0x00000053` | NO_BOOT_DEVICE | No bootable device was found. |
| `0x00000054` | LM_SERVER_INTERNAL_ERROR | Internal Windows fault: lm server internal error. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000055` | DATA_COHERENCY_EXCEPTION | Internal Windows fault: data coherency exception. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000056` | INSTRUCTION_COHERENCY_EXCEPTION | Internal Windows fault: instruction coherency exception. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000057` | XNS_INTERNAL_ERROR | Internal Windows fault: xns internal error. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000058` | FTDISK_INTERNAL_ERROR | Internal Windows fault: ftdisk internal error. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000059` | PINBALL_FILE_SYSTEM | The PINBALL file-system driver hit a fatal error. |
| `0x0000005A` | CRITICAL_SERVICE_FAILED | Internal Windows fault: critical service failed. (Rare; see the per-code Microsoft page for detail.) |
| `0x0000005B` | SET_ENV_VAR_FAILED | Internal Windows fault: set env var failed. (Rare; see the per-code Microsoft page for detail.) |
| `0x0000005C` | HAL_INITIALIZATION_FAILED | A Windows internal component (Hal) failed to initialize during startup. (Rare/internal.) |
| `0x0000005D` | UNSUPPORTED_PROCESSOR | Internal Windows fault: unsupported processor. (Rare; see the per-code Microsoft page for detail.) |
| `0x0000005E` | OBJECT_INITIALIZATION_FAILED | A Windows internal component (Object) failed to initialize during startup. (Rare/internal.) |
| `0x0000005F` | SECURITY_INITIALIZATION_FAILED | A Windows internal component (Security) failed to initialize during startup. (Rare/internal.) |
| `0x00000060` | PROCESS_INITIALIZATION_FAILED | A Windows internal component (Process) failed to initialize during startup. (Rare/internal.) |
| `0x00000061` | HAL1_INITIALIZATION_FAILED | A Windows internal component (Hal1) failed to initialize during startup. (Rare/internal.) |
| `0x00000062` | OBJECT1_INITIALIZATION_FAILED | A Windows internal component (Object1) failed to initialize during startup. (Rare/internal.) |
| `0x00000063` | SECURITY1_INITIALIZATION_FAILED | A Windows internal component (Security1) failed to initialize during startup. (Rare/internal.) |
| `0x00000064` | SYMBOLIC_INITIALIZATION_FAILED | A Windows internal component (Symbolic) failed to initialize during startup. (Rare/internal.) |
| `0x00000065` | MEMORY1_INITIALIZATION_FAILED | A Windows internal component (Memory1) failed to initialize during startup. (Rare/internal.) |
| `0x00000066` | CACHE_INITIALIZATION_FAILED | A Windows internal component (Cache) failed to initialize during startup. (Rare/internal.) |
| `0x00000067` | CONFIG_INITIALIZATION_FAILED | A Windows internal component (Config) failed to initialize during startup. (Rare/internal.) |
| `0x00000068` | FILE_INITIALIZATION_FAILED | A Windows internal component (File) failed to initialize during startup. (Rare/internal.) |
| `0x00000069` | IO1_INITIALIZATION_FAILED | A Windows internal component (Io1) failed to initialize during startup. (Rare/internal.) |
| `0x0000006A` | LPC_INITIALIZATION_FAILED | A Windows internal component (Lpc) failed to initialize during startup. (Rare/internal.) |
| `0x0000006B` | PROCESS1_INITIALIZATION_FAILED | A Windows internal component (Process1) failed to initialize during startup. (Rare/internal.) |
| `0x0000006C` | REFMON_INITIALIZATION_FAILED | A Windows internal component (Refmon) failed to initialize during startup. (Rare/internal.) |
| `0x0000006D` | SESSION1_INITIALIZATION_FAILED | A Windows internal component (Session1) failed to initialize during startup. (Rare/internal.) |
| `0x0000006E` | SESSION2_INITIALIZATION_FAILED | A Windows internal component (Session2) failed to initialize during startup. (Rare/internal.) |
| `0x0000006F` | SESSION3_INITIALIZATION_FAILED | A Windows internal component (Session3) failed to initialize during startup. (Rare/internal.) |
| `0x00000070` | SESSION4_INITIALIZATION_FAILED | A Windows internal component (Session4) failed to initialize during startup. (Rare/internal.) |
| `0x00000071` | SESSION5_INITIALIZATION_FAILED | A Windows internal component (Session5) failed to initialize during startup. (Rare/internal.) |
| `0x00000072` | ASSIGN_DRIVE_LETTERS_FAILED | Internal Windows fault: assign drive letters failed. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000073` | CONFIG_LIST_FAILED | Internal Windows fault: config list failed. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000074` | BAD_SYSTEM_CONFIG_INFO | Internal Windows fault: bad system config info. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000075` | CANNOT_WRITE_CONFIGURATION | Internal Windows fault: cannot write configuration. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000076` | PROCESS_HAS_LOCKED_PAGES | Internal Windows fault: process has locked pages. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000077` | KERNEL_STACK_INPAGE_ERROR | Windows couldn't read the kernel stack from disk — failing drive or controller. |
| `0x00000078` | PHASE0_EXCEPTION | Internal Windows fault: phase0 exception. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000079` | MISMATCHED_HAL | Internal Windows fault: mismatched hal. (Rare; see the per-code Microsoft page for detail.) |
| `0x0000007A` | KERNEL_DATA_INPAGE_ERROR | Windows couldn't read a page from disk — failing drive, cable, or RAM. |
| `0x0000007B` | INACCESSIBLE_BOOT_DEVICE | Windows can't reach the boot drive — controller mode change, driver, or dying disk. |
| `0x0000007C` | BUGCODE_NDIS_DRIVER | A device-driver fault — the name points at the driver/component involved. (Driver-related.) |
| `0x0000007D` | INSTALL_MORE_MEMORY | Internal Windows fault: install more memory. (Rare; see the per-code Microsoft page for detail.) |
| `0x0000007E` | SYSTEM_THREAD_EXCEPTION_NOT_HANDLED | A system thread crashed unhandled — usually a driver (often names it). |
| `0x0000007F` | UNEXPECTED_KERNEL_MODE_TRAP | The CPU hit an unexpected trap — hardware fault, RAM, or a driver. |
| `0x00000080` | NMI_HARDWARE_FAILURE | A hardware failure raised a non-maskable interrupt. |
| `0x00000081` | SPIN_LOCK_INIT_FAILURE | A kernel threading/synchronization fault — almost always a driver bug. (Internal.) |
| `0x00000082` | DFS_FILE_SYSTEM | The DFS file-system driver hit a fatal error. |
| `0x00000085` | SETUP_FAILURE | Internal Windows fault: setup failure. (Rare; see the per-code Microsoft page for detail.) |
| `0x0000008B` | MBR_CHECKSUM_MISMATCH | Internal Windows fault: mbr checksum mismatch. (Rare; see the per-code Microsoft page for detail.) |
| `0x0000008E` | KERNEL_MODE_EXCEPTION_NOT_HANDLED | An unhandled kernel error — commonly a bad driver or hardware. |
| `0x0000008F` | PP0_INITIALIZATION_FAILED | A Windows internal component (Pp0) failed to initialize during startup. (Rare/internal.) |
| `0x00000090` | PP1_INITIALIZATION_FAILED | A Windows internal component (Pp1) failed to initialize during startup. (Rare/internal.) |
| `0x00000092` | UP_DRIVER_ON_MP_SYSTEM | A device-driver fault — the name points at the driver/component involved. (Driver-related.) |
| `0x00000093` | INVALID_KERNEL_HANDLE | Internal Windows fault: invalid kernel handle. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000094` | KERNEL_STACK_LOCKED_AT_EXIT | Internal Windows fault: kernel stack locked at exit. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000096` | INVALID_WORK_QUEUE_ITEM | Internal Windows fault: invalid work queue item. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000097` | BOUND_IMAGE_UNSUPPORTED | Internal Windows fault: bound image unsupported. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000098` | END_OF_NT_EVALUATION_PERIOD | Internal Windows fault: end of nt evaluation period. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000099` | INVALID_REGION_OR_SEGMENT | Internal Windows fault: invalid region or segment. (Rare; see the per-code Microsoft page for detail.) |
| `0x0000009A` | SYSTEM_LICENSE_VIOLATION | Internal Windows fault: system license violation. (Rare; see the per-code Microsoft page for detail.) |
| `0x0000009B` | UDFS_FILE_SYSTEM | The UDFS file-system driver hit a fatal error. |
| `0x0000009C` | MACHINE_CHECK_EXCEPTION | The CPU reported an unrecoverable hardware fault (heat, power, or failing part). |
| `0x0000009E` | USER_MODE_HEALTH_MONITOR | Internal Windows fault: user mode health monitor. (Rare; see the per-code Microsoft page for detail.) |
| `0x0000009F` | DRIVER_POWER_STATE_FAILURE | A driver didn't complete a power (sleep/wake) transition in time. |
| `0x000000A0` | INTERNAL_POWER_ERROR | The power manager hit a fatal error — often during sleep/hibernate. |
| `0x000000A1` | PCI_BUS_DRIVER_INTERNAL | A device-driver fault — the name points at the driver/component involved. (Driver-related.) |
| `0x000000A2` | MEMORY_IMAGE_CORRUPT | Internal Windows fault: memory image corrupt. (Rare; see the per-code Microsoft page for detail.) |
| `0x000000A3` | ACPI_DRIVER_INTERNAL | A device-driver fault — the name points at the driver/component involved. (Driver-related.) |
| `0x000000A4` | CNSS_FILE_SYSTEM_FILTER | Internal Windows fault: cnss file system filter. (Rare; see the per-code Microsoft page for detail.) |
| `0x000000A5` | ACPI_BIOS_ERROR | The motherboard firmware (ACPI/BIOS) is faulty or incompatible — update the BIOS. |
| `0x000000A7` | BAD_EXHANDLE | Internal Windows fault: bad exhandle. (Rare; see the per-code Microsoft page for detail.) |
| `0x000000AC` | HAL_MEMORY_ALLOCATION | A low-level hardware-abstraction/processor init fault. (Rare/internal.) |
| `0x000000AD` | VIDEO_DRIVER_DEBUG_REPORT_REQUEST | A device-driver fault — the name points at the driver/component involved. (Driver-related.) |
| `0x000000B1` | BGI_DETECTED_VIOLATION | Internal Windows fault: bgi detected violation. (Rare; see the per-code Microsoft page for detail.) |
| `0x000000B4` | VIDEO_DRIVER_INIT_FAILURE | A device-driver fault — the name points at the driver/component involved. (Driver-related.) |
| `0x000000B8` | ATTEMPTED_SWITCH_FROM_DPC | A memory-management fault — often a driver bug or failing RAM. |
| `0x000000B9` | CHIPSET_DETECTED_ERROR | Internal Windows fault: chipset detected error. (Rare; see the per-code Microsoft page for detail.) |
| `0x000000BA` | SESSION_HAS_VALID_VIEWS_ON_EXIT | Internal Windows fault: session has valid views on exit. (Rare; see the per-code Microsoft page for detail.) |
| `0x000000BB` | NETWORK_BOOT_INITIALIZATION_FAILED | A Windows internal component (Network Boot) failed to initialize during startup. (Rare/internal.) |
| `0x000000BC` | NETWORK_BOOT_DUPLICATE_ADDRESS | Internal Windows fault: network boot duplicate address. (Rare; see the per-code Microsoft page for detail.) |
| `0x000000BD` | INVALID_HIBERNATED_STATE | Internal Windows fault: invalid hibernated state. (Rare; see the per-code Microsoft page for detail.) |
| `0x000000BE` | ATTEMPTED_WRITE_TO_READONLY_MEMORY | A driver tried to write to read-only memory — a driver bug. |
| `0x000000BF` | MUTEX_ALREADY_OWNED | A kernel threading/synchronization fault — almost always a driver bug. (Internal.) |
| `0x000000C1` | SPECIAL_POOL_DETECTED_MEMORY_CORRUPTION | A memory-management fault — often a driver bug or failing RAM. |
| `0x000000C2` | BAD_POOL_CALLER | A driver made an illegal memory-pool request. |
| `0x000000C4` | DRIVER_VERIFIER_DETECTED_VIOLATION | Driver Verifier caught a misbehaving driver (names it). |
| `0x000000C5` | DRIVER_CORRUPTED_EXPOOL | A driver corrupted the kernel memory pool. |
| `0x000000C6` | DRIVER_CAUGHT_MODIFYING_FREED_POOL | A device-driver fault — the name points at the driver/component involved. (Driver-related.) |
| `0x000000C7` | TIMER_OR_DPC_INVALID | Internal Windows fault: timer or dpc invalid. (Rare; see the per-code Microsoft page for detail.) |
| `0x000000C8` | IRQL_UNEXPECTED_VALUE | A driver accessed memory at the wrong interrupt level — a driver bug. |
| `0x000000C9` | DRIVER_VERIFIER_IOMANAGER_VIOLATION | Driver Verifier caught a driver misusing the I/O manager. |
| `0x000000CA` | PNP_DETECTED_FATAL_ERROR | Internal Windows fault: pnp detected fatal error. (Rare; see the per-code Microsoft page for detail.) |
| `0x000000CB` | DRIVER_LEFT_LOCKED_PAGES_IN_PROCESS | A device-driver fault — the name points at the driver/component involved. (Driver-related.) |
| `0x000000CC` | PAGE_FAULT_IN_FREED_SPECIAL_POOL | A memory-management fault — often a driver bug or failing RAM. |
| `0x000000CD` | PAGE_FAULT_BEYOND_END_OF_ALLOCATION | A memory-management fault — often a driver bug or failing RAM. |
| `0x000000CE` | DRIVER_UNLOADED_WITHOUT_CANCELLING_PENDING_OPERATIONS | A device-driver fault — the name points at the driver/component involved. (Driver-related.) |
| `0x000000CF` | TERMINAL_SERVER_DRIVER_MADE_INCORRECT_MEMORY_REFERENCE | A device-driver fault — the name points at the driver/component involved. (Driver-related.) |
| `0x000000D0` | DRIVER_CORRUPTED_MMPOOL | A device-driver fault — the name points at the driver/component involved. (Driver-related.) |
| `0x000000D1` | DRIVER_IRQL_NOT_LESS_OR_EQUAL | A specific driver accessed invalid memory — usually names the faulting driver. |
| `0x000000D2` | BUGCODE_ID_DRIVER | A device-driver fault — the name points at the driver/component involved. (Driver-related.) |
| `0x000000D3` | DRIVER_PORTION_MUST_BE_NONPAGED | A device-driver fault — the name points at the driver/component involved. (Driver-related.) |
| `0x000000D4` | SYSTEM_SCAN_AT_RAISED_IRQL_CAUGHT_IMPROPER_DRIVER_UNLOAD | A device-driver fault — the name points at the driver/component involved. (Driver-related.) |
| `0x000000D5` | DRIVER_PAGE_FAULT_IN_FREED_SPECIAL_POOL | A device-driver fault — the name points at the driver/component involved. (Driver-related.) |
| `0x000000D6` | DRIVER_PAGE_FAULT_BEYOND_END_OF_ALLOCATION | A device-driver fault — the name points at the driver/component involved. (Driver-related.) |
| `0x000000D7` | DRIVER_UNMAPPING_INVALID_VIEW | A device-driver fault — the name points at the driver/component involved. (Driver-related.) |
| `0x000000D8` | DRIVER_USED_EXCESSIVE_PTES | A device-driver fault — the name points at the driver/component involved. (Driver-related.) |
| `0x000000D9` | LOCKED_PAGES_TRACKER_CORRUPTION | Internal Windows fault: locked pages tracker corruption. (Rare; see the per-code Microsoft page for detail.) |
| `0x000000DA` | SYSTEM_PTE_MISUSE | A memory-management fault — often a driver bug or failing RAM. |
| `0x000000DB` | DRIVER_CORRUPTED_SYSPTES | A device-driver fault — the name points at the driver/component involved. (Driver-related.) |
| `0x000000DC` | DRIVER_INVALID_STACK_ACCESS | A device-driver fault — the name points at the driver/component involved. (Driver-related.) |
| `0x000000DE` | POOL_CORRUPTION_IN_FILE_AREA | A memory-management fault — often a driver bug or failing RAM. |
| `0x000000DF` | IMPERSONATING_WORKER_THREAD | A kernel threading/synchronization fault — almost always a driver bug. (Internal.) |
| `0x000000E0` | ACPI_BIOS_FATAL_ERROR | A fatal motherboard-firmware (ACPI) error — update the BIOS. |
| `0x000000E1` | WORKER_THREAD_RETURNED_AT_BAD_IRQL | A kernel threading/synchronization fault — almost always a driver bug. (Internal.) |
| `0x000000E2` | MANUALLY_INITIATED_CRASH | A crash triggered on purpose (keyboard/debugger) — not a fault. |
| `0x000000E3` | RESOURCE_NOT_OWNED | Internal Windows fault: resource not owned. (Rare; see the per-code Microsoft page for detail.) |
| `0x000000E4` | WORKER_INVALID | Internal Windows fault: worker invalid. (Rare; see the per-code Microsoft page for detail.) |
| `0x000000E6` | DRIVER_VERIFIER_DMA_VIOLATION | A device-driver fault — the name points at the driver/component involved. (Driver-related.) |
| `0x000000E7` | INVALID_FLOATING_POINT_STATE | Internal Windows fault: invalid floating point state. (Rare; see the per-code Microsoft page for detail.) |
| `0x000000E8` | INVALID_CANCEL_OF_FILE_OPEN | Internal Windows fault: invalid cancel of file open. (Rare; see the per-code Microsoft page for detail.) |
| `0x000000E9` | ACTIVE_EX_WORKER_THREAD_TERMINATION | A kernel threading/synchronization fault — almost always a driver bug. (Internal.) |
| `0x000000EA` | THREAD_STUCK_IN_DEVICE_DRIVER | A driver looped and hung the system — often the display driver. |
| `0x000000EB` | DIRTY_MAPPED_PAGES_CONGESTION | Internal Windows fault: dirty mapped pages congestion. (Rare; see the per-code Microsoft page for detail.) |
| `0x000000EC` | SESSION_HAS_VALID_SPECIAL_POOL_ON_EXIT | A memory-management fault — often a driver bug or failing RAM. |
| `0x000000ED` | UNMOUNTABLE_BOOT_VOLUME | The boot volume can't be mounted — file-system corruption on the system drive. |
| `0x000000EF` | CRITICAL_PROCESS_DIED | A process Windows can't run without died (e.g. csrss, wininit) — often corruption. |
| `0x000000F0` | STORAGE_MINIPORT_ERROR | A storage controller (miniport) driver failed. |
| `0x000000F1` | SCSI_VERIFIER_DETECTED_VIOLATION | Internal Windows fault: scsi verifier detected violation. (Rare; see the per-code Microsoft page for detail.) |
| `0x000000F2` | HARDWARE_INTERRUPT_STORM | A device flooded the CPU with interrupts — a faulty device or driver. |
| `0x000000F3` | DISORDERLY_SHUTDOWN | Internal Windows fault: disorderly shutdown. (Rare; see the per-code Microsoft page for detail.) |
| `0x000000F4` | CRITICAL_OBJECT_TERMINATION | A critical system object was terminated — similar to CRITICAL_PROCESS_DIED. |
| `0x000000F5` | FLTMGR_FILE_SYSTEM | The file-system filter manager failed — often a filter driver (AV/backup). |
| `0x000000F6` | PCI_VERIFIER_DETECTED_VIOLATION | Internal Windows fault: pci verifier detected violation. (Rare; see the per-code Microsoft page for detail.) |
| `0x000000F7` | DRIVER_OVERRAN_STACK_BUFFER | A driver overran a stack buffer — a driver bug (or attack). |
| `0x000000F8` | RAMDISK_BOOT_INITIALIZATION_FAILED | A Windows internal component (Ramdisk Boot) failed to initialize during startup. (Rare/internal.) |
| `0x000000F9` | DRIVER_RETURNED_STATUS_REPARSE_FOR_VOLUME_OPEN | A device-driver fault — the name points at the driver/component involved. (Driver-related.) |
| `0x000000FA` | HTTP_DRIVER_CORRUPTED | A device-driver fault — the name points at the driver/component involved. (Driver-related.) |
| `0x000000FC` | ATTEMPTED_EXECUTE_OF_NOEXECUTE_MEMORY | Code tried to run from non-executable memory — driver bug or malware. |
| `0x000000FD` | DIRTY_NOWRITE_PAGES_CONGESTION | Internal Windows fault: dirty nowrite pages congestion. (Rare; see the per-code Microsoft page for detail.) |
| `0x000000FE` | BUGCODE_USB_DRIVER | A device-driver fault — the name points at the driver/component involved. (Driver-related.) |
| `0x000000FF` | RESERVE_QUEUE_OVERFLOW | Internal Windows fault: reserve queue overflow. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000100` | LOADER_BLOCK_MISMATCH | Internal Windows fault: loader block mismatch. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000101` | CLOCK_WATCHDOG_TIMEOUT | A CPU core stopped responding — often a CPU/hardware or firmware problem. |
| `0x00000102` | DPC_WATCHDOG_TIMEOUT | A deferred routine ran too long — driver or firmware issue. |
| `0x00000103` | MUP_FILE_SYSTEM | The MUP file-system driver hit a fatal error. |
| `0x00000104` | AGP_INVALID_ACCESS | A graphics/display subsystem fault — usually the GPU driver. (Video-related.) |
| `0x00000105` | AGP_GART_CORRUPTION | A graphics/display subsystem fault — usually the GPU driver. (Video-related.) |
| `0x00000106` | AGP_ILLEGALLY_REPROGRAMMED | A graphics/display subsystem fault — usually the GPU driver. (Video-related.) |
| `0x00000108` | THIRD_PARTY_FILE_SYSTEM_FAILURE | Internal Windows fault: third party file system failure. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000109` | CRITICAL_STRUCTURE_CORRUPTION | Internal Windows fault: critical structure corruption. (Rare; see the per-code Microsoft page for detail.) |
| `0x0000010A` | APP_TAGGING_INITIALIZATION_FAILED | A Windows internal component (App Tagging) failed to initialize during startup. (Rare/internal.) |
| `0x0000010C` | FSRTL_EXTRA_CREATE_PARAMETER_VIOLATION | Internal Windows fault: fsrtl extra create parameter violation. (Rare; see the per-code Microsoft page for detail.) |
| `0x0000010D` | WDF_VIOLATION | Internal Windows fault: wdf violation. (Rare; see the per-code Microsoft page for detail.) |
| `0x0000010E` | VIDEO_MEMORY_MANAGEMENT_INTERNAL | The GPU memory manager failed — display driver or GPU fault. |
| `0x0000010F` | RESOURCE_MANAGER_EXCEPTION_NOT_HANDLED | Internal Windows fault: resource manager exception not handled. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000111` | RECURSIVE_NMI | Internal Windows fault: recursive nmi. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000112` | MSRPC_STATE_VIOLATION | Internal Windows fault: msrpc state violation. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000113` | VIDEO_DXGKRNL_FATAL_ERROR | The DirectX graphics kernel failed — display driver issue. |
| `0x00000114` | VIDEO_SHADOW_DRIVER_FATAL_ERROR | A device-driver fault — the name points at the driver/component involved. (Driver-related.) |
| `0x00000115` | AGP_INTERNAL | A graphics/display subsystem fault — usually the GPU driver. (Video-related.) |
| `0x00000116` | VIDEO_TDR_FAILURE | The graphics driver stopped responding and was reset — GPU driver or overheating GPU. |
| `0x00000117` | VIDEO_TDR_TIMEOUT_DETECTED | The GPU didn't respond in time and was reset. |
| `0x00000119` | VIDEO_SCHEDULER_INTERNAL_ERROR | The GPU scheduler hit a fatal error — usually the display driver. |
| `0x0000011A` | EM_INITIALIZATION_FAILURE | A Windows internal component (Em) failed to initialize during startup. (Rare/internal.) |
| `0x0000011B` | DRIVER_RETURNED_HOLDING_CANCEL_LOCK | A device-driver fault — the name points at the driver/component involved. (Driver-related.) |
| `0x0000011C` | ATTEMPTED_WRITE_TO_CM_PROTECTED_STORAGE | A memory-management fault — often a driver bug or failing RAM. |
| `0x0000011D` | EVENT_TRACING_FATAL_ERROR | Internal Windows fault: event tracing fatal error. (Rare; see the per-code Microsoft page for detail.) |
| `0x0000011E` | TOO_MANY_RECURSIVE_FAULTS | Internal Windows fault: too many recursive faults. (Rare; see the per-code Microsoft page for detail.) |
| `0x0000011F` | INVALID_DRIVER_HANDLE | A device-driver fault — the name points at the driver/component involved. (Driver-related.) |
| `0x00000120` | BITLOCKER_FATAL_ERROR | A fatal BitLocker drive-encryption error. |
| `0x00000121` | DRIVER_VIOLATION | A device-driver fault — the name points at the driver/component involved. (Driver-related.) |
| `0x00000122` | WHEA_INTERNAL_ERROR | The hardware-error reporting subsystem itself failed. |
| `0x00000123` | CRYPTO_SELF_TEST_FAILURE | A kernel security/integrity check failed. (Security/internal.) |
| `0x00000124` | WHEA_UNCORRECTABLE_ERROR | A fatal hardware error — heat, failing CPU/RAM, or unstable overclock. |
| `0x00000125` | NMR_INVALID_STATE | Internal Windows fault: nmr invalid state. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000126` | NETIO_INVALID_POOL_CALLER | A memory-management fault — often a driver bug or failing RAM. |
| `0x00000127` | PAGE_NOT_ZERO | Internal Windows fault: page not zero. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000128` | WORKER_THREAD_RETURNED_WITH_BAD_IO_PRIORITY | A kernel threading/synchronization fault — almost always a driver bug. (Internal.) |
| `0x00000129` | WORKER_THREAD_RETURNED_WITH_BAD_PAGING_IO_PRIORITY | A kernel threading/synchronization fault — almost always a driver bug. (Internal.) |
| `0x0000012A` | MUI_NO_VALID_SYSTEM_LANGUAGE | Internal Windows fault: mui no valid system language. (Rare; see the per-code Microsoft page for detail.) |
| `0x0000012B` | FAULTY_HARDWARE_CORRUPTED_PAGE | Hardware (usually RAM) corrupted a memory page. |
| `0x0000012C` | EXFAT_FILE_SYSTEM | The exFAT file-system driver hit a fatal error. |
| `0x0000012D` | VOLSNAP_OVERLAPPED_TABLE_ACCESS | Internal Windows fault: volsnap overlapped table access. (Rare; see the per-code Microsoft page for detail.) |
| `0x0000012E` | INVALID_MDL_RANGE | Internal Windows fault: invalid mdl range. (Rare; see the per-code Microsoft page for detail.) |
| `0x0000012F` | VHD_BOOT_INITIALIZATION_FAILED | A Windows internal component (Vhd Boot) failed to initialize during startup. (Rare/internal.) |
| `0x00000130` | DYNAMIC_ADD_PROCESSOR_MISMATCH | Internal Windows fault: dynamic add processor mismatch. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000131` | INVALID_EXTENDED_PROCESSOR_STATE | Internal Windows fault: invalid extended processor state. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000132` | RESOURCE_OWNER_POINTER_INVALID | Internal Windows fault: resource owner pointer invalid. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000133` | DPC_WATCHDOG_VIOLATION | A driver (often storage/SSD firmware) hung the CPU too long — update drivers/firmware. |
| `0x00000134` | DRIVE_EXTENDER | Internal Windows fault: drive extender. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000135` | REGISTRY_FILTER_DRIVER_EXCEPTION | A device-driver fault — the name points at the driver/component involved. (Driver-related.) |
| `0x00000136` | VHD_BOOT_HOST_VOLUME_NOT_ENOUGH_SPACE | Internal Windows fault: vhd boot host volume not enough space. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000137` | WIN32K_HANDLE_MANAGER | Internal Windows fault: win32k handle manager. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000138` | GPIO_CONTROLLER_DRIVER_ERROR | A device-driver fault — the name points at the driver/component involved. (Driver-related.) |
| `0x00000139` | KERNEL_SECURITY_CHECK_FAILURE | Windows detected a corrupted data structure — driver bug or memory corruption. |
| `0x0000013A` | KERNEL_MODE_HEAP_CORRUPTION | The kernel heap is corrupted — almost always a driver bug. |
| `0x0000013B` | PASSIVE_INTERRUPT_ERROR | Internal Windows fault: passive interrupt error. (Rare; see the per-code Microsoft page for detail.) |
| `0x0000013C` | INVALID_IO_BOOST_STATE | Internal Windows fault: invalid io boost state. (Rare; see the per-code Microsoft page for detail.) |
| `0x0000013D` | CRITICAL_INITIALIZATION_FAILURE | A Windows internal component (Critical) failed to initialize during startup. (Rare/internal.) |
| `0x00000140` | STORAGE_DEVICE_ABNORMALITY_DETECTED | The storage stack detected an abnormal drive condition. |
| `0x00000143` | PROCESSOR_DRIVER_INTERNAL | A device-driver fault — the name points at the driver/component involved. (Driver-related.) |
| `0x00000144` | BUGCODE_USB3_DRIVER | A device-driver fault — the name points at the driver/component involved. (Driver-related.) |
| `0x00000145` | SECURE_BOOT_VIOLATION | A Secure Boot policy check failed — boot integrity problem. |
| `0x00000147` | ABNORMAL_RESET_DETECTED | Internal Windows fault: abnormal reset detected. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000149` | REFS_FILE_SYSTEM | The ReFS file-system driver hit a fatal error. |
| `0x0000014A` | KERNEL_WMI_INTERNAL | Internal Windows fault: kernel wmi internal. (Rare; see the per-code Microsoft page for detail.) |
| `0x0000014B` | SOC_SUBSYSTEM_FAILURE | Internal Windows fault: soc subsystem failure. (Rare; see the per-code Microsoft page for detail.) |
| `0x0000014C` | FATAL_ABNORMAL_RESET_ERROR | Internal Windows fault: fatal abnormal reset error. (Rare; see the per-code Microsoft page for detail.) |
| `0x0000014D` | EXCEPTION_SCOPE_INVALID | Internal Windows fault: exception scope invalid. (Rare; see the per-code Microsoft page for detail.) |
| `0x0000014E` | SOC_CRITICAL_DEVICE_REMOVED | Internal Windows fault: soc critical device removed. (Rare; see the per-code Microsoft page for detail.) |
| `0x0000014F` | PDC_WATCHDOG_TIMEOUT | A watchdog timer expired — something ran too long or a component hung. (Often driver/firmware.) |
| `0x00000150` | TCPIP_AOAC_NIC_ACTIVE_REFERENCE_LEAK | Internal Windows fault: tcpip aoac nic active reference leak. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000151` | UNSUPPORTED_INSTRUCTION_MODE | Internal Windows fault: unsupported instruction mode. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000152` | INVALID_PUSH_LOCK_FLAGS | Internal Windows fault: invalid push lock flags. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000153` | KERNEL_LOCK_ENTRY_LEAKED_ON_THREAD_TERMINATION | Internal Windows fault: kernel lock entry leaked on thread termination. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000154` | UNEXPECTED_STORE_EXCEPTION | An unexpected error while accessing the compressed-memory store — often failing RAM. |
| `0x00000155` | OS_DATA_TAMPERING | Internal Windows fault: os data tampering. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000157` | KERNEL_THREAD_PRIORITY_FLOOR_VIOLATION | Internal Windows fault: kernel thread priority floor violation. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000158` | ILLEGAL_IOMMU_PAGE_FAULT | Internal Windows fault: illegal iommu page fault. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000159` | HAL_ILLEGAL_IOMMU_PAGE_FAULT | A low-level hardware-abstraction/processor init fault. (Rare/internal.) |
| `0x0000015A` | SDBUS_INTERNAL_ERROR | Internal Windows fault: sdbus internal error. (Rare; see the per-code Microsoft page for detail.) |
| `0x0000015B` | WORKER_THREAD_RETURNED_WITH_SYSTEM_PAGE_PRIORITY_ACTIVE | A kernel threading/synchronization fault — almost always a driver bug. (Internal.) |
| `0x00000160` | WIN32K_ATOMIC_CHECK_FAILURE | Internal Windows fault: win32k atomic check failure. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000162` | KERNEL_AUTO_BOOST_INVALID_LOCK_RELEASE | Internal Windows fault: kernel auto boost invalid lock release. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000163` | WORKER_THREAD_TEST_CONDITION | A kernel threading/synchronization fault — almost always a driver bug. (Internal.) |
| `0x00000164` | WIN32K_CRITICAL_FAILURE | Internal Windows fault: win32k critical failure. (Rare; see the per-code Microsoft page for detail.) |
| `0x0000016C` | INVALID_RUNDOWN_PROTECTION_FLAGS | Internal Windows fault: invalid rundown protection flags. (Rare; see the per-code Microsoft page for detail.) |
| `0x0000016D` | INVALID_SLOT_ALLOCATOR_FLAGS | Internal Windows fault: invalid slot allocator flags. (Rare; see the per-code Microsoft page for detail.) |
| `0x0000016E` | ERESOURCE_INVALID_RELEASE | Internal Windows fault: eresource invalid release. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000170` | CLUSTER_CSV_CLUSSVC_DISCONNECT_WATCHDOG | Internal Windows fault: cluster csv clussvc disconnect watchdog. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000171` | CRYPTO_LIBRARY_INTERNAL_ERROR | A kernel security/integrity check failed. (Security/internal.) |
| `0x00000173` | COREMSGCALL_INTERNAL_ERROR | Internal Windows fault: coremsgcall internal error. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000174` | COREMSG_INTERNAL_ERROR | Internal Windows fault: coremsg internal error. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000178` | ELAM_DRIVER_DETECTED_FATAL_ERROR | A device-driver fault — the name points at the driver/component involved. (Driver-related.) |
| `0x0000017B` | PROFILER_CONFIGURATION_ILLEGAL | Internal Windows fault: profiler configuration illegal. (Rare; see the per-code Microsoft page for detail.) |
| `0x0000017E` | MICROCODE_REVISION_MISMATCH | The CPU microcode revision doesn't match — firmware/OS mismatch. |
| `0x00000187` | VIDEO_DWMINIT_TIMEOUT_FALLBACK_BDD | A graphics/display subsystem fault — usually the GPU driver. (Video-related.) |
| `0x00000189` | BAD_OBJECT_HEADER | Internal Windows fault: bad object header. (Rare; see the per-code Microsoft page for detail.) |
| `0x0000018B` | SECURE_KERNEL_ERROR | A kernel security/integrity check failed. (Security/internal.) |
| `0x0000018C` | HYPERGUARD_VIOLATION | A kernel security/integrity check failed. (Security/internal.) |
| `0x0000018D` | SECURE_FAULT_UNHANDLED | A kernel security/integrity check failed. (Security/internal.) |
| `0x0000018E` | KERNEL_PARTITION_REFERENCE_VIOLATION | Internal Windows fault: kernel partition reference violation. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000191` | PF_DETECTED_CORRUPTION | Internal Windows fault: pf detected corruption. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000192` | KERNEL_AUTO_BOOST_LOCK_ACQUISITION_WITH_RAISED_IRQL | Internal Windows fault: kernel auto boost lock acquisition with raised irql. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000196` | LOADER_ROLLBACK_DETECTED | Internal Windows fault: loader rollback detected. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000197` | WIN32K_SECURITY_FAILURE | Internal Windows fault: win32k security failure. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000199` | KERNEL_STORAGE_SLOT_IN_USE | Internal Windows fault: kernel storage slot in use. (Rare; see the per-code Microsoft page for detail.) |
| `0x0000019A` | WORKER_THREAD_RETURNED_WHILE_ATTACHED_TO_SILO | A kernel threading/synchronization fault — almost always a driver bug. (Internal.) |
| `0x0000019B` | TTM_FATAL_ERROR | Internal Windows fault: ttm fatal error. (Rare; see the per-code Microsoft page for detail.) |
| `0x0000019C` | WIN32K_POWER_WATCHDOG_TIMEOUT | A watchdog timer expired — something ran too long or a component hung. (Often driver/firmware.) |
| `0x000001A0` | TTM_WATCHDOG_TIMEOUT | A watchdog timer expired — something ran too long or a component hung. (Often driver/firmware.) |
| `0x000001A2` | WIN32K_CALLOUT_WATCHDOG_BUGCHECK | A watchdog timer expired — something ran too long or a component hung. (Often driver/firmware.) |
| `0x000001AA` | EXCEPTION_ON_INVALID_STACK | Internal Windows fault: exception on invalid stack. (Rare; see the per-code Microsoft page for detail.) |
| `0x000001AB` | UNWIND_ON_INVALID_STACK | Internal Windows fault: unwind on invalid stack. (Rare; see the per-code Microsoft page for detail.) |
| `0x000001C6` | FAST_ERESOURCE_PRECONDITION_VIOLATION | Internal Windows fault: fast eresource precondition violation. (Rare; see the per-code Microsoft page for detail.) |
| `0x000001C7` | STORE_DATA_STRUCTURE_CORRUPTION | Internal Windows fault: store data structure corruption. (Rare; see the per-code Microsoft page for detail.) |
| `0x000001C8` | MANUALLY_INITIATED_POWER_BUTTON_HOLD | Crash captured from a forced power-button hold — not a software fault. |
| `0x000001CA` | SYNTHETIC_WATCHDOG_TIMEOUT | A watchdog timer expired — something ran too long or a component hung. (Often driver/firmware.) |
| `0x000001CB` | INVALID_SILO_DETACH | Internal Windows fault: invalid silo detach. (Rare; see the per-code Microsoft page for detail.) |
| `0x000001CD` | INVALID_CALLBACK_STACK_ADDRESS | Internal Windows fault: invalid callback stack address. (Rare; see the per-code Microsoft page for detail.) |
| `0x000001CE` | INVALID_KERNEL_STACK_ADDRESS | Internal Windows fault: invalid kernel stack address. (Rare; see the per-code Microsoft page for detail.) |
| `0x000001CF` | HARDWARE_WATCHDOG_TIMEOUT | A watchdog timer expired — something ran too long or a component hung. (Often driver/firmware.) |
| `0x000001D0` | ACPI_FIRMWARE_WATCHDOG_TIMEOUT | A watchdog timer expired — something ran too long or a component hung. (Often driver/firmware.) |
| `0x000001D2` | WORKER_THREAD_INVALID_STATE | A kernel threading/synchronization fault — almost always a driver bug. (Internal.) |
| `0x000001D3` | WFP_INVALID_OPERATION | Internal Windows fault: wfp invalid operation. (Rare; see the per-code Microsoft page for detail.) |
| `0x000001D5` | DRIVER_PNP_WATCHDOG | A device-driver fault — the name points at the driver/component involved. (Driver-related.) |
| `0x000001D6` | WORKER_THREAD_RETURNED_WITH_NON_DEFAULT_WORKLOAD_CLASS | A kernel threading/synchronization fault — almost always a driver bug. (Internal.) |
| `0x000001D7` | EFS_FATAL_ERROR | Internal Windows fault: efs fatal error. (Rare; see the per-code Microsoft page for detail.) |
| `0x000001D8` | UCMUCSI_FAILURE | Internal Windows fault: ucmucsi failure. (Rare; see the per-code Microsoft page for detail.) |
| `0x000001D9` | HAL_IOMMU_INTERNAL_ERROR | A low-level hardware-abstraction/processor init fault. (Rare/internal.) |
| `0x000001DA` | HAL_BLOCKED_PROCESSOR_INTERNAL_ERROR | A low-level hardware-abstraction/processor init fault. (Rare/internal.) |
| `0x000001DB` | IPI_WATCHDOG_TIMEOUT | A watchdog timer expired — something ran too long or a component hung. (Often driver/firmware.) |
| `0x000001DC` | DMA_COMMON_BUFFER_VECTOR_ERROR | Internal Windows fault: dma common buffer vector error. (Rare; see the per-code Microsoft page for detail.) |
| `0x000001DD` | BUGCODE_MBBADAPTER_DRIVER | A device-driver fault — the name points at the driver/component involved. (Driver-related.) |
| `0x000001DE` | BUGCODE_WIFIADAPTER_DRIVER | A device-driver fault — the name points at the driver/component involved. (Driver-related.) |
| `0x000001DF` | PROCESSOR_START_TIMEOUT | Internal Windows fault: processor start timeout. (Rare; see the per-code Microsoft page for detail.) |
| `0x000001E4` | VIDEO_DXGKRNL_SYSMM_FATAL_ERROR | A graphics/display subsystem fault — usually the GPU driver. (Video-related.) |
| `0x000001E9` | ILLEGAL_ATS_INITIALIZATION | Internal Windows fault: illegal ats initialization. (Rare; see the per-code Microsoft page for detail.) |
| `0x000001EA` | SECURE_PCI_CONFIG_SPACE_ACCESS_VIOLATION | A kernel security/integrity check failed. (Security/internal.) |
| `0x000001EB` | DAM_WATCHDOG_TIMEOUT | A watchdog timer expired — something ran too long or a component hung. (Often driver/firmware.) |
| `0x000001ED` | HANDLE_ERROR_ON_CRITICAL_THREAD | Internal Windows fault: handle error on critical thread. (Rare; see the per-code Microsoft page for detail.) |
| `0x000001F1` | KASAN_ENLIGHTENMENT_VIOLATION | A kernel security/integrity check failed. (Security/internal.) |
| `0x000001F2` | KASAN_ILLEGAL_ACCESS | A kernel security/integrity check failed. (Security/internal.) |
| `0x00000356` | XBOX_ERACTRL_CS_TIMEOUT | Internal Windows fault: xbox eractrl cs timeout. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000BFE` | BC_BLUETOOTH_VERIFIER_FAULT | Internal Windows fault: bc bluetooth verifier fault. (Rare; see the per-code Microsoft page for detail.) |
| `0x00000BFF` | BC_BTHMINI_VERIFIER_FAULT | Internal Windows fault: bc bthmini verifier fault. (Rare; see the per-code Microsoft page for detail.) |
| `0x00020001` | HYPERVISOR_ERROR | The Hyper-V hypervisor hit a fatal error. |
| `0x1000007E` | SYSTEM_THREAD_EXCEPTION_NOT_HANDLED_M | Internal Windows fault: system thread exception not handled m. (Rare; see the per-code Microsoft page for detail.) |
| `0x1000007F` | UNEXPECTED_KERNEL_MODE_TRAP_M | Internal Windows fault: unexpected kernel mode trap m. (Rare; see the per-code Microsoft page for detail.) |
| `0x1000008E` | KERNEL_MODE_EXCEPTION_NOT_HANDLED_M | Internal Windows fault: kernel mode exception not handled m. (Rare; see the per-code Microsoft page for detail.) |
| `0x100000EA` | THREAD_STUCK_IN_DEVICE_DRIVER_M | A device-driver fault — the name points at the driver/component involved. (Driver-related.) |
| `0x4000008A` | THREAD_TERMINATE_HELD_MUTEX | A kernel threading/synchronization fault — almost always a driver bug. (Internal.) |
| `0xC0000218` | STATUS_CANNOT_LOAD_REGISTRY_FILE | Windows couldn't load a registry hive — corruption or disk failure. |
| `0xC000021A` | WINLOGON_FATAL_ERROR | The Winlogon logon process failed fatally. |
| `0xC0000221` | STATUS_IMAGE_CHECKSUM_MISMATCH | A system file failed its checksum — a corrupted or damaged driver/DLL. |
| `0xDEADDEAD` | MANUALLY_INITIATED_CRASH1 | A crash triggered on purpose for testing — not a fault. |
