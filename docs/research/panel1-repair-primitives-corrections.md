# Panel #1 Correction — Windows Repair Command Catalog

**Prepared for:** repair-shop owner / catalog operator
**Scope:** corrects mis-classified risk labels already live in the execution catalog, adds primary-sourced primitives to fill cluster gaps, and reconciles the inflated primitive count.

---

## Risk reclassification (existing catalog)

These primitives are **already wired into the agent and can be executed**. The defect being corrected here is destructive operations carrying a `read_only` or `reversible` label — that is the most dangerous class of bug this catalog can have, because the agent trusts the label to decide whether to ask a human first.

| Primitive | Old risk | Correct risk | Why |
|---|---|---|---|
| `defrag` | read_only | **Reversible** | It writes to disk — moves/consolidates file fragments and issues TRIM/retrim. Not a query operation. (Cross-ref: `Optimize-Volume`, added in Part 2, is the modern PowerShell equivalent and is already correctly sourced as reversible.) |
| `TPM.msc` → "Clear TPM" | Reversible | **Destructive** | Invalidates all TPM-sealed keys immediately. If BitLocker protection isn't suspended first, this causes a recovery-key lockout on next boot — permanent data loss if the 48-digit key wasn't saved. (Cross-ref: `manage-bde -protectors -disable`, added in Part 2, is the correct pre-step.) |
| `chkdsk /r` and `chkdsk /f /r` | Reversible | **Destructive** | Repair mode (`/f`, `/r`) can orphan or truncate files while resolving filesystem inconsistencies; `/r` implies `/f`. (Read-only `chkdsk` with no `/f`/`/r` is unaffected by this correction.) |
| `pnputil /delete-driver` | Reversible | **Destructive** | Permanently removes the driver package from the driver store; `/uninstall /force` strips it off live devices immediately, which can leave hardware without a functioning driver. |
| `DISM /Online /Cleanup-Image /StartComponentCleanup` (no `/ResetBase`) | Reversible | **Destructive** | Immediately deletes superseded component versions from the component store — those specific updates can no longer be uninstalled afterward. |
| `Set-SecureBootUEFI` | Reversible | **Destructive / high-risk** | Writes Secure Boot key material (PK/KEK/DB/DBX). A bad write can render the machine unbootable, with no simple recovery path. |
| `Start-MpWDOScan` (Defender Offline) | *(no risk-class change)* | *(add operational warning)* | Not a data-risk reclass — but the scan **forces an unannounced reboot** into a special offline scan environment. Flag as an availability/scheduling risk: warn the customer/tech before running it. |
| `netsh advfirewall reset` | Reversible | **Reversible — confirmed** | The command's own `export`/`import` mechanism is the backup path (see Part 2). Do **not** lump this with the two rows below — it behaves differently from the other `netsh` resets. |
| `netsh int ip reset` | Reversible | **Destructive / effectively irreversible** | Rewrites the TCP/IP registry subsystem. Windows creates **no automatic backup**. Recovery requires a pre-made export the command itself never produces. Warn "export config BEFORE running." |
| `netsh winsock reset` | Reversible | **Destructive / effectively irreversible** | Same defect as the row above: the documented recovery path (`netsh winsock dump`) only helps if a tech ran it *before* the reset — Windows never creates that backup on its own. Do not classify as reversible by default. |
| `bootrec /fixmbr` and `bootrec /fixboot` | read_only | **Destructive** | Both **write** the boot sector/MBR — an overwrite is never read-only. On UEFI+GPT systems `/fixmbr` is a no-op and `/fixboot` frequently returns "Access denied" — flag both as **legacy BIOS/MBR-era**, not universal fixes. |

---

## Newly-added primitives (gap fill)

15 new, primary-sourced primitives across 5 clusters. Table columns: Primitive | What it does | Command | Risk | Reversal | Source.

### storage_gaps

| Primitive | What it does | Command | Risk | Reversal | Source |
|---|---|---|---|---|---|
| `diskpart clean` | Erases the partition table on the disk with focus | `diskpart > clean` | **Destructive — irreversible** | None. Partition table is gone; no normal-means recovery. | Microsoft Learn |
| `diskpart clean all` | Erases partitions **and** zero-fills every sector | `diskpart > clean all` | **Destructive — irreversible** (most severe variant — full zero-fill) | None. | Microsoft Learn |
| `diskpart list` | Enumerates disks/partitions/volumes/vdisks | `diskpart > list disk \| list partition \| list volume \| list vdisk` | Read-only | N/A | Microsoft Learn |
| `diskpart detail` | Displays detailed properties of the selected object | `diskpart > detail disk \| detail partition \| detail volume \| detail vdisk` | Read-only | N/A | Microsoft Learn |
| `Optimize-Volume` | Defrag / TRIM-retrim / slab consolidation / tier optimization | `Optimize-Volume -DriveLetter X -Defrag \| -ReTrim \| -Analyze` | Reversible | Re-runnable maintenance op; no data loss. | Microsoft Learn |

### boot_gaps

| Primitive | What it does | Command | Risk | Reversal | Source |
|---|---|---|---|---|---|
| `bcdboot` | Rebuilds the EFI System Partition and boot files; recreates the BCD store; updates UEFI NVRAM entries | `bcdboot C:\Windows` | Reversible | Re-run from WinRE/WinPE; `/m` preserves existing boot-entry values; restore a prior BCD backup if available. | Microsoft Learn |

### driver_gaps

| Primitive | What it does | Command | Risk | Reversal | Source |
|---|---|---|---|---|---|
| Display Driver Uninstaller (DDU) | Fully strips GPU drivers (AMD/NVIDIA/Intel) including registry keys, folders, and driver-store remnants | Run in Safe Mode → select GPU → "Clean and restart" / "Clean and Shutdown" | **DESTRUCTIVE / high-effort recovery (GPU driver reinstall required)** | Reinstall the GPU driver immediately — stage the installer *before* running DDU; create a System Restore point beforehand. Not irreversible: full display function returns once a driver is reinstalled, but a bad sequence leaves the shop with no video output in the interim. | Wagnardsoft (official DDU developer) |

### security_gaps

| Primitive | What it does | Command | Risk | Reversal | Source |
|---|---|---|---|---|---|
| `manage-bde -protectors -disable` | Temporarily suspends BitLocker protection (key made unsecured on drive) | `manage-bde -protectors -disable <drive> [-rebootcount N]` | Reversible | Auto-resumes on next reboot (unless `-rebootcount 0`), or re-enable manually with `-protectors -enable`. | Microsoft Learn |
| `Start-MpScan` | Runs a Windows Defender scan of a path or the system | `Start-MpScan [-ScanPath <path>] [-ScanType <type>] [-AsJob]` | Read-only | N/A | Microsoft Learn |

### networking_gaps

| Primitive | What it does | Command | Risk | Reversal | Source |
|---|---|---|---|---|---|
| `tracert` | Traces the route to a destination via incrementing-TTL ICMP | `tracert <target>` | Read-only | N/A | Microsoft Learn |
| `pathping` | Latency/loss stats at each intermediate hop | `pathping <target>` | Read-only | N/A | Microsoft Learn |
| `Get-NetAdapter` | Lists network adapter properties | `Get-NetAdapter [-Name] [-Physical] ...` | Read-only | N/A | Microsoft Learn |
| `Restart-NetAdapter` | Disables and re-enables an adapter | `Restart-NetAdapter -Name <adapter>` | Reversible | Brief disruption only; re-run or re-enable manually. | Microsoft Learn |
| `netsh advfirewall reset` | Resets firewall policy to defaults | `netsh advfirewall reset [export <path>]` | Reversible | Built-in `export`/`import` is the backup path for this specific reset. | Microsoft Learn |
| `netsh winsock reset` | Resets the Winsock catalog, strips custom LSPs | `netsh winsock reset` | **Destructive / effectively irreversible** (corrects the source verdict's "reversible" label — see Part 1, `netsh int ip reset` / `netsh winsock reset` row) | Only reversible if `netsh winsock dump` was run **before** the reset — Windows does not create that backup automatically. | Microsoft Learn |

### Destructive — requires human sign-off before execution

From this gap-fill batch: **`diskpart clean`, `diskpart clean all`** (irreversible), **DDU** (destructive, high-effort recovery), **`netsh winsock reset`** (destructive, no auto-backup).

These sit alongside the Part 1 corrections that are *already* in the live catalog and now correctly flagged destructive: `TPM.msc` "Clear TPM", `chkdsk /r`/`/f /r`, `pnputil /delete-driver`, `DISM /StartComponentCleanup`, `Set-SecureBootUEFI`, `bootrec /fixmbr`/`/fixboot`, `netsh int ip reset`. Any execution path that auto-runs primitives by risk class must gate all of the above on human confirmation.

---

## Count dedup (actionable)

**Duplicate collapse — explicit family list:**

| Duplicate family | Raw rows before | Canonical primitive kept | Rows removed |
|---|---|---|---|
| chkdsk (`/r`, `/f`, `/f /r`) | 3 | `chkdsk /r` (implies `/f`) | −2 |
| sfc (`/scannow`, `/verifyonly`, `/scanfile`, offline, + Memory-cluster dup) | 5 | `sfc /scannow` | −4 |
| DISM `/RestoreHealth` (OS-image + Memory-cluster dup + offline-Boot-cluster variant) | 3 | `DISM /Online /Cleanup-Image /RestoreHealth` | −2 |
| bootrec granular (`/fixmbr`, `/fixboot`, `/rebuildbcd`) + combined "Bootrec.exe" wrapper | 4 | 3 granular commands kept distinct; wrapper merged away | −1 |
| bcdedit (Boot-cluster entry + Firmware-cluster entry) | 2 | `bcdedit` | −1 |
| WinRE Startup Repair (+ "WinRE Startup Repair and Command Prompt") | 2 | WinRE Startup Repair | −1 |
| Defender Offline (`Start-MpWDOScan` + GUI-launch dup) | 2 | `Start-MpWDOScan` | −1 |
| Event Viewer (crash-log + Defender-log + Performance entries) | 3 | Event Viewer (log source is a parameter, not a separate primitive) | −2 |
| **Total** | **24** | **10** | **−14** |

Additionally: `chkdsk`, `sfc`, and `DISM` are moved **out of** the mis-filed "Memory & hardware diagnostics" category (this is what produced the "Memory-dup" rows collapsed above) — a taxonomy fix with no further count impact beyond the removals already listed.

**Gap-fill additions (Part 2, verified):** 5 storage_gaps + 1 boot_gaps + 1 driver_gaps + 2 security_gaps + 6 networking_gaps = **+15**.

**Arithmetic:**

```
starting count (N)  −  14 (duplicate collapse)  +  15 (verified gap additions)  =  N + 1
```

`N` — the pre-audit catalog total — was **not** included in this handoff and no full row-by-row catalog export was available to this pass to re-derive it independently, so the absolute deduped total cannot be stated as a verified number here. What *is* fully shown and verifiable is the delta: **net +1 row** (14 duplicate/miscategorized rows removed, 15 new sourced rows added). The practical takeaway for the shop owner: this pass does not meaningfully change catalog *size* — it changes catalog *correctness* (14 fewer duplicate/miscategorized rows, 15 more properly sourced ones). The final absolute figure (`N + 1`) needs `N` confirmed against a full row-by-row catalog pass before it can be reported as anything more than an estimate.

---

## Still open

- A full destructive-vs-read_only sweep of the **remaining** catalog has not been done — e.g. `bootrec /rebuildbcd` itself, `bcdedit`, and the broader set of Defender scan/threat-remediation cmdlets beyond `Start-MpScan`/`Start-MpWDOScan` are unaudited.
- Firmware-flash primitives and a fuller boot cluster (`bootrec /rebuildbcd`, Startup Repair/`reagentc`, a WinRE entry) remain **gaps to re-source** — the firmware entries were rejected for *sourcing*, not because the operation isn't real. Firmware flashing is the highest-consequence destructive action in this domain and needs a primary-sourced catalog entry before it can be added.
- `netsh int ip reset`'s risk correction (Part 1) is applied on the audit's directive but has no accompanying primary-sourced object in this handoff — attach a citation.
- The starting catalog count (`N`) needed to finalize Part 3's deduped total is not available in this pass — requires a full row-by-row catalog export.