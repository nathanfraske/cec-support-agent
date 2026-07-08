<!-- GENERATED from crates/tools-windows/src/catalog.rs — edit the catalog, then regenerate. -->
# Tool catalog — plain-language reference

The engine's built-in tool inventory in plain language. **50 safe-core tools** (read-only or reversible) are registered and runnable; the **destructive ops** below are a separate, differentiated list — tracked but NOT auto-runnable, and gated behind human sign-off.

## Safe core (registered, runnable)

### Storage & disk
| Tool | What it does | Risk | Runs on Windows now? |
|---|---|---|---|
| **Check a disk for errors (read-only)** | Scans a drive's file system for logical and physical errors without changing anything. | read-only (safe to run) | needs input (drop-in) |
| **Report drive health** | Lists each physical drive with its health status (Healthy / Warning / Unhealthy). | read-only (safe to run) | yes |
| **Read drive SMART data** | Reads a drive's reliability counters: temperature, power-on hours, wear, and error counts. | read-only (safe to run) | yes |
| **Show file-system details** | Reports drive type, volume properties, NTFS metadata locations, and sector sizes. | read-only (safe to run) | needs input (drop-in) |
| **Check a drive's 'needs-check' flag** | Reports whether a volume is marked dirty (which schedules a disk check at next restart). | read-only (safe to run) | needs input (drop-in) |
| **List disks and partitions** | Enumerates disks, partitions, and volumes with their sizes and layout. | read-only (safe to run) | needs input (drop-in) |
| **Show a disk's details** | Displays the detailed properties of a selected disk, partition, or volume. | read-only (safe to run) | needs input (drop-in) |
| **Optimize a drive (defrag / TRIM)** | Defragments an HDD or issues TRIM to an SSD; a maintenance operation that does not erase data. | reversible (undoable) | needs input (drop-in) |
| **Schedule a startup disk check** | Displays or changes which drives are checked automatically at startup. | reversible (undoable) | needs input (drop-in) |

### Windows health
| Tool | What it does | Risk | Runs on Windows now? |
|---|---|---|---|
| **Verify system files (no repair)** | Scans protected Windows system files for corruption without changing anything. | read-only (safe to run) | yes |
| **Repair system files** | Scans and repairs corrupted protected Windows system files from a known-good source. | reversible (undoable) | yes |
| **Check Windows image health** | Reports whether the Windows component store is healthy, repairable, or not — no repair. | read-only (safe to run) | yes |
| **Scan Windows image for corruption** | Scans the Windows component store for corruption and logs what it finds — no repair. | read-only (safe to run) | yes |
| **Repair the Windows image** | Repairs corrupted or missing Windows components using Windows Update or a given source. | reversible (undoable) | yes |

### Drivers
| Tool | What it does | Risk | Runs on Windows now? |
|---|---|---|---|
| **List devices** | Enumerates devices with their status, drivers, and any problem codes. | read-only (safe to run) | yes |
| **List installed driver packages** | Lists third-party driver packages in the Windows driver store. | read-only (safe to run) | yes |
| **Disable a device** | Disables a specific device to isolate or work around problematic hardware. | reversible (undoable) | needs input (drop-in) |

### Memory & hardware
| Tool | What it does | Risk | Runs on Windows now? |
|---|---|---|---|
| **Test system memory** | Schedules the built-in Windows Memory Diagnostic to test RAM at the next restart. | read-only (safe to run) | needs input (drop-in) |

### Boot & recovery
| Tool | What it does | Risk | Runs on Windows now? |
|---|---|---|---|
| **Back up boot configuration** | Exports the Boot Configuration Data (BCD) store to a file so it can be restored. | read-only (safe to run) | needs input (drop-in) |
| **Rebuild the boot menu** | Scans the disks for Windows installations and rebuilds the boot configuration store. | reversible (undoable) | yes |
| **Rebuild boot files (modern UEFI)** | Recreates the EFI System Partition boot files and BCD store — the modern boot-repair path. | reversible (undoable) | needs input (drop-in) |

### Networking
| Tool | What it does | Risk | Runs on Windows now? |
|---|---|---|---|
| **Clear the DNS cache** | Clears the local DNS resolver cache to fix stale name-resolution problems. | reversible (undoable) | yes |
| **Release the network address** | Releases the current DHCP-assigned IP address. | reversible (undoable) | yes |
| **Renew the network address** | Requests a fresh IP address from the DHCP server. | reversible (undoable) | yes |
| **Trace the network route** | Traces the network path to a destination, hop by hop, to locate where traffic stops. | read-only (safe to run) | needs input (drop-in) |
| **Measure path latency and loss** | Reports latency and packet loss at each hop between here and a destination. | read-only (safe to run) | needs input (drop-in) |
| **Ping a host** | Sends echo requests to confirm basic network reachability to a host. | read-only (safe to run) | needs input (drop-in) |
| **Look up a DNS name** | Queries DNS servers to diagnose name-resolution problems. | read-only (safe to run) | needs input (drop-in) |
| **Show network connections** | Displays active connections, listening ports, and per-connection owning process. | read-only (safe to run) | yes |
| **List network adapters** | Lists network adapters and their status and properties. | read-only (safe to run) | yes |
| **Restart a network adapter** | Disables and re-enables a network adapter to clear a stuck connection. | reversible (undoable) | needs input (drop-in) |
| **Reset the firewall to defaults** | Resets Windows Firewall policy to defaults; the built-in export makes it restorable. | reversible (undoable) | yes |

### Security & malware
| Tool | What it does | Risk | Runs on Windows now? |
|---|---|---|---|
| **Run a malware scan** | Runs a Microsoft Defender scan of the system or a chosen path. | read-only (safe to run) | yes |
| **Run an offline malware scan** | Reboots into a trusted environment and scans for rootkits and boot-record malware. | reversible (undoable) | yes |
| **Show antivirus status** | Reports Defender's protection state, real-time protection, and signature freshness. | read-only (safe to run) | yes |
| **List detected threats** | Lists malware threats Defender has detected, past and active. | read-only (safe to run) | yes |
| **Suspend drive encryption** | Temporarily suspends BitLocker (the required safe step before clearing the TPM). | reversible (undoable) | needs input (drop-in) |

### Performance
| Tool | What it does | Risk | Runs on Windows now? |
|---|---|---|---|
| **Generate a power/energy report** | Analyzes power settings and battery health and writes a report. | read-only (safe to run) | yes |
| **Generate a performance report** | Collects performance counters and produces a system diagnostics report. | read-only (safe to run) | yes |

### Firmware / Secure Boot
| Tool | What it does | Risk | Runs on Windows now? |
|---|---|---|---|
| **Check Secure Boot** | Reports whether Secure Boot is enabled on a UEFI system. | read-only (safe to run) | yes |

### Registry
| Tool | What it does | Risk | Runs on Windows now? |
|---|---|---|---|
| **Read a registry key** | Reads registry subkeys and values, with optional recursive search. | read-only (safe to run) | needs input (drop-in) |
| **Back up a registry key** | Exports a registry key to a file so it can be restored after edits. | read-only (safe to run) | needs input (drop-in) |
| **Restore a registry key** | Restores registry keys/values from a previously exported file. | reversible (undoable) | needs input (drop-in) |
| **Compare registry keys** | Compares two registry locations to find differences. | read-only (safe to run) | needs input (drop-in) |

### Backup & restore
| Tool | What it does | Risk | Runs on Windows now? |
|---|---|---|---|
| **Back up the system** | Creates an image, volume, file, or system-state backup. | reversible (undoable) | needs input (drop-in) |
| **List restore snapshots** | Lists Volume Shadow Copy snapshots available for restore. | read-only (safe to run) | yes |

### Processes & files
| Tool | What it does | Risk | Runs on Windows now? |
|---|---|---|---|
| **List running processes (detailed)** | Lists running processes with handle, thread, memory, and module detail, as JSON. | read-only (safe to run) | yes |
| **List running processes (basic)** | Lists running processes in CSV form (built-in, no dependencies). | read-only (safe to run) | yes |
| **List open files** | Lists files currently held open, locally or from the network. | read-only (safe to run) | yes |
| **Search text in files** | Searches files for a literal or regex pattern (built-in log/text triage). | read-only (safe to run) | needs input (drop-in) |

## Destructive ops — DIFFERENTIATED (tracked, human-sign-off gated, not auto-run)

These are catalogued so they are visible and the sign-off gate knows them, but they are **not** in the runnable set. Wiring any of them is a deliberate, human-gated step.

| Operation | What it does | Reversal / caution |
|---|---|---|
| ⚠ **Wipe a disk** | Erases the partition table (and with 'all', zero-fills every sector) of a disk. | IRREVERSIBLE — all data is destroyed. Confirm the correct disk and that data is backed up. |
| ⚠ **Repair a disk (chkdsk /r)** | Repairs file-system errors and recovers bad sectors — repair mode can orphan or truncate files. | Can lose data while repairing. Back up first; run read-only 'disk_check' before deciding. |
| ⚠ **Clean up the Windows image** | Deletes superseded Windows components; those specific updates can no longer be uninstalled. | Removes update rollback capability. Not reversible without reinstalling the affected updates. |
| ⚠ **Remove a driver package** | Permanently removes a driver package from the store, optionally ripping it off live devices. | Keep the original driver package to reinstall. /force can leave hardware without a driver. |
| ⚠ **Clean-remove GPU drivers (DDU)** | Fully strips graphics drivers and leftovers (the GPU-swap / driver-corruption case). | High-effort recovery: stage the GPU driver installer BEFORE running; no display output until reinstalled. |
| ⚠ **Change Secure Boot keys** | Writes Secure Boot key material (PK/KEK/DB/DBX). | A bad write can make the machine unbootable with no simple recovery. Expert-only. |
| ⚠ **Clear the security chip (TPM)** | Invalidates all TPM-sealed keys. | SUSPEND BitLocker first ('suspend_bitlocker') or you cause a recovery-key lockout = permanent data loss. |
| ⚠ **Reset the TCP/IP stack** | Rewrites the TCP/IP registry subsystem to defaults. | Windows makes NO automatic backup — export the config first; needs a reboot. |
| ⚠ **Reset the network sockets catalog** | Resets the Winsock catalog and strips custom network layers (LSPs). | Only reversible if 'netsh winsock dump' was captured beforehand; needs a reboot. |
| ⚠ **Rewrite the master boot record (legacy)** | Writes the MBR/boot sector — legacy BIOS only; a no-op or error on modern UEFI systems. | Writes boot structures; back up the BCD first. Prefer 'rebuild_boot_files' (bcdboot) on UEFI. |
| ⚠ **Delete a registry key** | Permanently removes registry subkeys or values. | Back up first with 'backup_registry_key' (reg export); deletion has no undo. |
| ⚠ **Reset this PC** | Reinstalls Windows; can keep files or remove everything depending on the option chosen. | 'Remove everything' wipes the device. Confirm the option and back up first. |
| ⚠ **Update firmware / BIOS** | Flashes system or device firmware (via vendor tool or Windows UpdateCapsule). | Power loss mid-flash can BRICK the board. Ensure stable power; have the recovery procedure ready. |
