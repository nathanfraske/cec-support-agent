# Consolidated tool inventory — both panels, merged & classified (2026-07-08)

Everything the two research panels vetted, merged into one view and split by **how the engine
consumes it**:

- **Built-in CLI** — ships with Windows / is a built-in command or PowerShell cmdlet. The engine
  invokes it directly. No download, no supply-chain risk. These map cleanly to registered
  `ACTION_VOCABULARY` tools.
- **Download** — third-party (or separately-downloaded Microsoft) binary. Must be **fetched +
  signature/hash-verified** before running → hits the `download_file` action + the F4 binary-provenance
  concern + a **commercial-use licensing** check (you run a shop).
- **GUI-only** — built-in but not headless-automatable; needs a person or UI automation (many have a
  CLI/PowerShell equivalent noted).

Risks below reflect the **corrected** panel #1 v2 classification (destructive ops that were mislabeled
are fixed). ⚠ = destructive / human-sign-off required.

Counts: ~**60 built-in CLI**, ~**30 download**, ~**13 GUI-only**.

---

## 1. BUILT-IN CLI (invoke directly — no download)

### Storage / disk
| Tool | What | Invocation | Risk |
|---|---|---|---|
| chkdsk (read) | FS check, no repair | `chkdsk d:` | read-only |
| chkdsk /r, /f, /f /r | FS **repair** | `chkdsk d: /r` | ⚠ destructive (can orphan/truncate) |
| chkntfs | schedule/exclude startup disk checks | `chkntfs /c d:` | reversible |
| fsutil fsinfo | drive/volume/NTFS/sector info | `fsutil fsinfo sectorinfo d:` | read-only |
| fsutil dirty | query/set dirty bit | `fsutil dirty query c:` | read-only (query) |
| Get-PhysicalDisk | disk HealthStatus | `Get-PhysicalDisk` | read-only |
| Get-StorageReliabilityCounter | SMART (temp, hours, wear, errors) | `… \| Get-StorageReliabilityCounter` | read-only |
| diskpart list / detail | enumerate disks/partitions/volumes | `diskpart > list disk` | read-only |
| diskpart clean / clean all | erase partition table / zero-fill | `diskpart > clean [all]` | ⚠ destructive — **irreversible** |
| Optimize-Volume | defrag / TRIM / slab / tier | `Optimize-Volume -DriveLetter X -Defrag` | reversible |

### OS integrity
| sfc /scannow, /scanfile, offline | scan+repair protected system files | `sfc /scannow` | reversible |
| sfc /verifyonly | scan only | `sfc /verifyonly` | read-only |
| DISM /CheckHealth, /ScanHealth | image corruption report | `DISM /Online /Cleanup-Image /ScanHealth` | read-only |
| DISM /RestoreHealth | repair component store | `DISM /Online /Cleanup-Image /RestoreHealth` | reversible |
| DISM /StartComponentCleanup | delete superseded components | `… /StartComponentCleanup` | ⚠ destructive (blocks update rollback) |
| DISM /StartComponentCleanup /ResetBase | remove ALL superseded | `… /ResetBase` | ⚠ destructive |

### Drivers
| pnputil /enum-devices, /enum-drivers | list devices/drivers | `pnputil /enum-devices` | read-only |
| pnputil /disable-device | disable a device | `pnputil /disable-device …` | reversible |
| pnputil /delete-driver | remove driver package | `pnputil /delete-driver <oem#.inf> [/uninstall /force]` | ⚠ destructive |

### Memory / hardware / crash
| mdsched.exe | Windows Memory Diagnostic (RAM test at boot) | `mdsched.exe` | read-only |
| Get-WinEvent | query event logs incl. WHEA | `Get-WinEvent -FilterHashtable @{LogName='System'}` | read-only |
| WER / LocalDumps config | crash-dump collection (registry) | `HKLM\…\Windows Error Reporting\LocalDumps` | reversible |

### Boot / recovery
| bootrec /rebuildbcd | rebuild BCD | `bootrec /rebuildbcd` | reversible (export BCD first) |
| bootrec /fixmbr, /fixboot | write MBR/boot sector | `bootrec /fixmbr` | ⚠ destructive; **legacy BIOS** (no-op on UEFI) |
| bcdedit /export, /import, /set | manage BCD store | `bcdedit /export c:\bcdbackup` | reversible |
| bcdboot | rebuild EFI System Partition / boot files (**modern UEFI path**) | `bcdboot C:\Windows` | reversible* (inference-derived label) |

### Networking
| ipconfig /flushdns, /release, /renew | DNS/DHCP | `ipconfig /flushdns` | reversible |
| tracert, pathping, ping, nslookup, netstat | path/connectivity/DNS/socket diagnostics | `tracert <target>` | read-only |
| Get-NetAdapter | adapter properties | `Get-NetAdapter` | read-only |
| Restart-NetAdapter | disable+re-enable adapter | `Restart-NetAdapter -Name <a>` | reversible |
| netsh advfirewall reset | reset firewall (has export/import) | `netsh advfirewall reset` | reversible |
| netsh int ip reset / winsock reset | reset TCP-IP / Winsock | `netsh winsock reset` | ⚠ destructive — no auto-backup (export FIRST) |

### Security / malware
| Start-MpScan | Defender scan | `Start-MpScan -ScanType 2` | read-only |
| Start-MpWDOScan | Defender **Offline** scan (forces reboot) | `Start-MpWDOScan` | reversible (⚠ unannounced reboot) |
| MpCmdRun.exe | Defender CLI (scan/update/getfiles) | `MpCmdRun.exe -Scan -ScanType 2` | reversible |
| Get-MpComputerStatus, Get-MpThreatDetection | Defender status/threats | `Get-MpComputerStatus` | read-only |
| manage-bde -protectors -disable | **suspend BitLocker** (do before TPM clear) | `manage-bde -protectors -disable <drv>` | reversible |

### Performance
| powercfg | power plans / energy & battery reports | `powercfg /energy` | reversible |
| defrag | analyze/consolidate/TRIM | `defrag c: /u /v` | reversible (writes — corrected from read-only) |
| Perfmon | perf counters / reports | `perfmon /report` | read-only |

### Firmware / Secure Boot
| Confirm-/Get-SecureBootUEFI | Secure Boot state/vars | `Confirm-SecureBootUEFI` | read-only |
| Set-SecureBootUEFI | write PK/KEK/DB/DBX | `Set-SecureBootUEFI …` | ⚠ destructive (can brick boot) |
| UEFI UpdateCapsule | firmware update via INF | (driver package) | ⚠ destructive (brick on power loss) |

### Registry
| reg query / export / save / compare | read/backup registry | `reg query <key> /s` | read-only |
| reg add / import / restore | write/restore registry | `reg add <key> …` | reversible |
| reg delete | remove keys/values | `reg delete <key> /f` | ⚠ destructive |
| Get-ItemProperty → ConvertTo-Json | live registry read as JSON | `Get-ItemProperty 'HKLM:\…' \| ConvertTo-Json` | read-only |

### Backup / restore
| Checkpoint-Computer | create restore point | `Checkpoint-Computer -Description "x"` | reversible |
| wbadmin | image/volume/file/system-state backup | `wbadmin start backup …` | reversible |
| vssadmin / VSS | shadow-copy snapshots | `vssadmin list shadows` | read-only |

### Process / handles / text
| Get-CimInstance Win32_Process → JSON | **richest** per-process data (handles, threads, modules) | `Get-CimInstance Win32_Process \| ConvertTo-Json` | read-only |
| tasklist /v /fo csv | process list (CSV) | `tasklist /v /fo csv` | read-only |
| OpenFiles.exe | open files (local/remote) | `openfiles /query /fo csv /v` | read-only |
| findstr | regex text search in files | `findstr /s /i /r "pat" *.log` | read-only |

---

## 2. DOWNLOAD (fetch + verify before running — `download_file` + provenance + license check)

| Tool | Function | Invocation | Risk | License / commercial-use |
|---|---|---|---|---|
| **handle.exe / Handle64** (Sysinternals) | open-handle & file-lock enumeration | `handle.exe -a -v` | read-only | Sysinternals EULA — **use OK**, no resale-as-service |
| **PsList** (Sysinternals) | process/thread/tree detail | `pslist -x` | read-only | Sysinternals EULA — use OK |
| **Autorunsc** (Sysinternals) | autostart/persistence (CSV/XML) | `autorunsc -a * -c` | read-only | Sysinternals EULA — use OK |
| **strings.exe** (Sysinternals) | strings from binaries | `strings -a -n 4 file.exe` | read-only | Sysinternals EULA — use OK |
| **NTFSInfo** (Sysinternals) | NTFS/MFT detail | `NTFSInfo.exe c:` | read-only | Sysinternals EULA — use OK |
| **WinDbg + !analyze** | crash-dump root-cause | `windbg -z dump.dmp` → `!analyze -v` | read-only | MS (Debugging Tools) — free |
| **dumpchk** | verify dump integrity | `dumpchk.exe dump.dmp` | read-only | MS (Debugging Tools) — free |
| **PowerToys File Locksmith CLI** | which process locks a file (+kill) | `FileLocksmithCLI.exe --json --kill --wait "<f>"` | ⚠ reversible (can terminate) | **MIT — commercial OK** |
| **LockHunter** | lock discovery/release | `LockHunter.exe /unlock /kill /silent "<f>"` | ⚠ reversible | Freeware — **commercial terms UNCLEAR** |
| **RECmd** (Zimmerman) | offline-hive registry parse (CSV/JSON) | `RECmd.exe --f hive --csv dir` | read-only | MIT — commercial OK |
| **Regipy** | offline-hive parse (JSON, plugins) | `regipy-dump hive -o out.json` | read-only | MIT — commercial OK |
| **python-registry** | offline-hive lib (no CLI) | `Registry.Registry('hive')` | read-only | Apache-2.0 — commercial OK |
| **RegRipper 3.0** | forensic hive plugins | `rip -a -f system -r SYSTEM` | read-only | **License UNCLEAR** — clarify before use |
| **Tesseract OCR** | OCR of screenshots/dialogs | `tesseract img.png out -l eng tsv` | read-only | Apache-2.0 — commercial OK |
| **LibreHardwareMonitor** | **live sensors** via HTTP JSON | `curl http://localhost:8085/data.json` | read-only | MPL-2.0 — commercial OK |
| **smartctl** (smartmontools) | disk SMART (JSON) | `smartctl -j /dev/sda` | read-only | GPL-2.0 — **commercial USE OK** |
| **CPU-Z** | static CPU/mem/board inventory | `cpuz.exe -txt=report` | read-only | Freeware — **commercial UNCLEAR** |
| **DISKSPD** (Microsoft) | disk stress/benchmark (XML) | `diskspd.exe -c10G -b4K …` | reversible | MIT — commercial OK |
| **Prime95** | CPU/FPU torture | `prime95.exe -t` | reversible | Freeware — **commercial UNCLEAR** |
| **stress-ng** | CPU/mem/IO stressors | `stress-ng --cpu 8 --timeout 60s` | reversible | GPL-2.0 — **commercial USE OK** |
| **OCCT** | CPU/GPU/RAM/PSU suite (CLI) | `OCCT_CommandLine.exe --test=CPU …` | reversible | Proprietary — **PAID for commercial** |
| **FurMark** | GPU stress/thermal | `furmark.exe --demo … --log-gpu-data` | reversible | Free tier **excludes commercial** (PRO Pack req'd) |
| **MemTest86** (PassMark) | USB-boot RAM test | boot from USB | read-only | Free edition — commercial OK (Pro paid) |
| **PresentMon** (Intel) | per-frame frametime capture (CSV) | `PresentMon.exe --process_name app.exe` | read-only | MIT — commercial OK |
| **CapFrameX** | frametime capture + analysis; **native MCP server** | run app → `claude mcp add … --transport http` (HTTP `/mcp`) | read-only | open source — commercial OK (verify) |
| **AMD RGP + Radeon Developer CLI** | GPU profiler (AMD only) | `RadeonDeveloperServiceCLI.exe …` | read-only | Proprietary (AMD) — free |
| **DDU (Display Driver Uninstaller)** | full GPU-driver strip (the 5070→5080 case) | Safe Mode → Clean and restart | ⚠ destructive (reinstall driver after) | Wagnardsoft freeware |
| **Intel Display Driver Clean Install** | Intel GPU clean install | installer → "clean installation" | reversible | Intel (vendor) |
| **AMD Cleanup Utility** | AMD GPU/audio driver removal | `amdcleanuputility.exe` (Safe Mode) | reversible | AMD (vendor) |
| **Dell Command Update / Lenovo Vantage / HP Image Assistant** | vendor firmware/driver update (CLI) | `dcu-cli.exe /applyUpdates` | ⚠ destructive (firmware flash) | vendor (⚠ SOURCES were weak — re-verify) |

---

## 3. GUI-ONLY (built-in; not headless — needs a person / UI automation)
Device Manager (Roll Back / Uninstall+delete driver) · Task Manager (Performance / Startup) · msconfig ·
Event Viewer *(CLI equiv: `Get-WinEvent`)* · WinRE Startup Repair · TPM.msc **⚠ Clear TPM = destructive**
*(suspend BitLocker first)* · rstrui (System Restore) · System Protection · File History · **Reset this PC ⚠
destructive** · Memory Dump config (System Properties) · ShowWhatProcessLocksFile · PowerToys Text Extractor.

---

## 4. Architectural takeaways
- **Built-in CLI is the safe core** — ~60 primitives the engine can register directly with no download,
  no supply chain. This is where `ACTION_VOCABULARY` should grow first.
- **Downloads need a provenance gate** — every §2 tool must be hash/signature-verified before the
  `download_file` action runs it on a customer box (menu item **F4**), and its **commercial-use license**
  recorded (⚠ flagged: LockHunter, CPU-Z, Prime95, RegRipper = unclear; OCCT = paid; FurMark = paid for
  commercial).
- **The ~18 destructive ops (⚠) must all be gated to human sign-off** — the engine's destructive-needs-human
  gate already exists; this list is what feeds it.
- **CapFrameX is the only MCP-native tool** found — first-class fit for the home-brain agent.
