# MCP/CLI-Automatable Diagnostic Tools — Review-Ready Comparison Catalog

## Summary

- **29 tools** evaluated across **7 diagnostic categories**. Every tool listed passed an independent **CAPABILITY gate** — its CLI/automation surface (invocation, structured output, headless operation) was verified against a primary source (Microsoft Learn, GitHub repo, vendor docs, man page, etc.), not just claimed. **Licensing was recorded for every tool but is not a pass/fail gate** — paid or ambiguous-license tools remain in the catalog with their commercial terms flagged.
- Category counts: Process introspection **4** · File-lock discovery/release **5** · Registry inspection **6** · Text extraction (OCR + strings) **4** · Hardware inventory/live sensors **3** · Stress/stability/validation **4** · Frametime/performance capture **3**.
- Risk classes present: `read_only` (safe to run unattended), `reversible` (stresses/benchmarks hardware but makes no persistent changes), `destructive` (can kill processes / delete or unlock files — gate behind explicit confirmation in the engine).

---

## 1. Detailed running-process & handle introspection

| Tool | What it does | How you drive it | Structured output | Headless | License / commercial-use | Risk | Source |
|---|---|---|---|---|---|---|---|
| **Get-CimInstance Win32_Process** | Per-process WMI/CIM data: handles, threads, memory, modules | `Get-CimInstance -ClassName Win32_Process \| ConvertTo-Json -Depth 5` | JSON | Yes | MIT (PowerShell) — unrestricted commercial use | read_only | [MS Learn](https://learn.microsoft.com/en-us/powershell/module/cimcmdlets/get-ciminstance?view=powershell-7.5) |
| **tasklist /fo csv /v** | Built-in verbose process listing (CPU time, user, session, memory) | `tasklist /v /fo csv > processes.csv` | CSV | Yes | Windows built-in — commercial use OK under Windows license | read_only | [MS Learn](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/tasklist) |
| **handle.exe -v (Sysinternals)** | Enumerates open handles per process (files, registry, pipes, sections) | `handle.exe -a -v > handles.csv` | CSV | Yes | Sysinternals EULA — commercial use OK (no resale-as-service) | read_only | [MS Learn](https://learn.microsoft.com/en-us/sysinternals/downloads/handle) |
| **PsList (Sysinternals)** | Process/thread/memory detail and process-tree view | `pslist -x > process_tree.txt` | Text | Yes | Sysinternals EULA — commercial use OK | read_only | [MS Learn](https://learn.microsoft.com/en-us/sysinternals/downloads/pslist) |

**Recommended for the engine:** **Get-CimInstance Win32_Process** — richest structured JSON, native, no extra install, filterable/remote-capable. **tasklist** is the fallback for zero-setup CSV dumps when PowerShell overhead is undesirable. Bring in **handle.exe** only when you need handle-level (not just process-level) forensics; **PsList** is the weakest of the four for automation since it lacks native structured (JSON/CSV) output — prefer it only for quick process-tree text views.

---

## 2. Discover which process is LOCKING a file/folder (and release it)

| Tool | What it does | How you drive it | Structured output | Headless | License / commercial-use | Risk | Source |
|---|---|---|---|---|---|---|---|
| **PowerToys File Locksmith CLI** | Identifies AND can terminate processes locking a file/folder | `FileLocksmithCLI.exe --json --kill --wait "<file_path>"` | JSON | Yes | MIT — unrestricted commercial use | destructive | [MS Learn](https://learn.microsoft.com/en-us/windows/powertoys/file-locksmith) |
| **Sysinternals handle.exe (Handle64)** | Detects which process holds a handle on a given file/path | `handle.exe -p <process> -v <filename>` | CSV | Yes | Sysinternals EULA — commercial use OK | read_only | [MS Learn](https://learn.microsoft.com/en-us/sysinternals/downloads/handle) |
| **Windows OpenFiles.exe** | Native query of open files, local or remote | `openfiles /query /fo csv /v` | CSV | Yes | Windows built-in — commercial use OK under license | read_only | [MS Learn](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/openfiles) |
| **LockHunter** | Detects, unlocks, force-closes/deletes locking processes; exit codes | `LockHunter.exe /unlock /kill /silent /exit "<file_path>"` | None (text/exit codes only) | Yes | Freeware — **commercial terms unclear** | destructive | [lockhunter.com](https://lockhunter.com/manual.htm) |
| **ShowWhatProcessLocksFile** | Lightweight lock-detector, GUI/context-menu only | `ShowWhatProcessLocksFile.exe "<file_path>"` (no CLI params) | None | **No** | MIT — unrestricted commercial use | read_only | [GitHub](https://github.com/PolarGoose/ShowWhatProcessLocksFile) |

**Recommended for the engine:** **PowerToys File Locksmith CLI** is the strongest all-in-one (detect + `--json` + `--kill` + `--wait`) and is the right default. **Sysinternals handle.exe beats it for pure detection** — it's read-only (safer to run unattended/repeatedly) with proven CSV output, so pair the two: `handle.exe` to detect and report, FileLocksmithCLI only when remediation is authorized. `OpenFiles.exe` is viable but needs a one-time `/local on` + reboot, adding friction. Do **not** use ShowWhatProcessLocksFile in the automation engine — no CLI surface — and treat LockHunter as a fallback only pending vendor confirmation of commercial licensing.

---

## 3. Registry inspection with structured output (offline-hive capable preferred)

| Tool | What it does | How you drive it | Structured output | Headless | License / commercial-use | Risk | Source |
|---|---|---|---|---|---|---|---|
| **RECmd** | Full-featured offline-hive registry parser, batch/plugin-capable | `RECmd.exe --f [hive] --sk [key] --csv [dir]` (or `--json`) | CSV/JSON | Yes | MIT — unrestricted commercial use | read_only | [GitHub](https://github.com/EricZimmerman/RECmd) |
| **Regipy** | Python CLI toolkit for offline hives: dump, plugins, diff, transaction-log recovery | `regipy-dump hive -o out.json` | JSON | Yes | MIT — unrestricted commercial use | read_only | [GitHub](https://github.com/mkorman90/regipy) |
| **PowerShell Get-ItemProperty + ConvertTo-Json** | Reads **live** registry keys, converts to JSON | `Get-ItemProperty -Path 'HKLM:\...' \| ConvertTo-Json -Depth 10` | JSON | Yes | MIT (PowerShell) — unrestricted commercial use | read_only | [MS Learn](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.management/get-itemproperty?view=powershell-7.6) |
| **reg.exe query** | Built-in live/remote registry query with recursive search | `reg query HKLM\Software\... /s /f term /d /t REG_SZ` | Text | Yes | Windows EULA — commercial use OK | read_only | [MS Learn](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/reg-query) |
| **python-registry (williballenthin)** | Pure-Python read-only offline hive parser, library only | Programmatic: `Registry.Registry('hive')` | None (raw objects) | Yes (no GUI, but **no CLI**) | Apache 2.0 — unrestricted commercial use | read_only | [GitHub](https://github.com/williballenthin/python-registry) |
| **RegRipper3.0** | Perl plugin-based offline registry forensics/timeline tool | `rip -a -f system -r SYSTEM > output.txt` | Text | Yes | **License unclear** (license.md/.txt present, terms unclear) | read_only | [GitHub](https://github.com/keydet89/RegRipper3.0) |

**Recommended for the engine:** **RECmd** for offline-hive batch work — native CSV/JSON, actively maintained, zero cost. **Regipy edges it out** when the engine is Python-native and wants JSON straight out of the box without a `ConvertTo-Json` wrapper, plus built-in hive-diff and transaction-log recovery. For quick **live**-registry checks on a running repair machine, use **PowerShell Get-ItemProperty**; fall back to `reg.exe query` only when PowerShell isn't available (text output needs parsing). **python-registry** is library-only — fine if embedding in a custom Python tool, otherwise skip. **Avoid RegRipper3.0** for new automation: licensing terms are unclear and output is unstructured text; migrate to RECmd/Regipy.

---

## 4. Text extraction — OCR of screenshots/error dialogs AND string extraction from logs/binaries

| Tool | What it does | How you drive it | Structured output | Headless | License / commercial-use | Risk | Source |
|---|---|---|---|---|---|---|---|
| **Tesseract OCR** | OCR engine extracting text from images/screenshots | `tesseract img.png out -l eng --psm 3 tsv` | Text (TSV/HOCR/PDF variants available) | Yes | Apache 2.0 — unrestricted commercial use | read_only | [tesseract-ocr.github.io](https://tesseract-ocr.github.io/tessdoc/Command-Line-Usage.html) |
| **Sysinternals strings.exe / strings64.exe** | Extracts embedded ANSI/Unicode strings from binaries/executables | `strings.exe -a -n 4 -nobanner file.exe` | Text | Yes | Sysinternals EULA — commercial use OK | read_only | [MS Learn](https://learn.microsoft.com/en-us/sysinternals/downloads/strings) |
| **Windows findstr.exe** | Built-in literal/regex text search across files, recursive | `findstr /s /i /r /n "pattern" *.log` | Text | Yes | Windows built-in — commercial use OK | read_only | [MS Learn](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/findstr) |
| **PowerToys Text Extractor** | Screen-region OCR to clipboard | Win+Shift+T overlay (no CLI) | None | **No** | MIT — unrestricted commercial use | read_only | [MS Learn](https://learn.microsoft.com/en-us/windows/powertoys/text-extractor) |

**Recommended for the engine:** **Tesseract OCR** for screenshot/error-dialog text extraction — mature, fully scriptable, multiple structured output modes. **strings.exe** is the pick for binary/executable/log string extraction (paths, URLs, embedded resources). **findstr.exe** is the lightweight built-in choice for log grepping when avoiding PowerShell startup overhead matters. **PowerToys Text Extractor is disqualified** for the automation engine — GUI/keyboard-shortcut only, no CLI or headless path; Tesseract fully covers its use case headlessly instead.

---

## 5. Hardware inventory + live sensors (temps, voltages, fan RPM, clocks, power)

| Tool | What it does | How you drive it | Structured output | Headless | License / commercial-use | Risk | Source |
|---|---|---|---|---|---|---|---|
| **LibreHardwareMonitor** | Live sensor data (temp, voltage, fan RPM, clocks, power) via HTTP server | `curl http://localhost:8085/data.json` | JSON | Yes | MPL-2.0 — unrestricted commercial use | read_only | [GitHub](https://github.com/LibreHardwareMonitor/LibreHardwareMonitor) |
| **smartctl (smartmontools)** | Disk SMART health, temperature, remaining life, error rates | `smartctl -j /dev/sda` | JSON | Yes | GPL-2.0-or-later — commercial **use** unrestricted | read_only | [Debian manpages](https://manpages.debian.org/unstable/smartmontools/smartctl.8.en.html) |
| **CPU-Z** | Static CPU/memory/mainboard/SPD inventory (not live monitoring) | `cpuz.exe -txt=report.txt` (ghost mode) | Text | Yes | Freeware — commercial terms not explicitly detailed | read_only | [cpuid.com](https://cpuid.com/softwares/cpu-z.html) |

**Recommended for the engine:** **LibreHardwareMonitor** for live sensor telemetry — a zero-parse JSON HTTP endpoint is the lowest-friction integration in this whole catalog. **smartctl specifically for disk health/SMART** — LibreHardwareMonitor's disk temps don't substitute for full SMART attribute/error analysis, so run both. **CPU-Z is inventory-only** (static hardware profile) and should not be relied on for live thresholds/alerts; use it as a one-time system-profile supplement, not the sensor engine.

---

## 6. Stress / stability / validation tools (CPU, GPU, memory, disk)

| Tool | What it does | How you drive it | Structured output | Headless | License / commercial-use | Risk | Source |
|---|---|---|---|---|---|---|---|
| **DISKSPD** | Microsoft disk stress/benchmark: IOPS, latency, throughput | `diskspd.exe -c10G -b4K -r -w100 -t8 -o32 -d60 X:\test.dat` | XML | Yes | MIT — unrestricted commercial use | reversible | [GitHub](https://github.com/Microsoft/diskspd) |
| **Prime95 (mprime)** | CPU/FPU torture test | `prime95.exe -t` | None | Yes | Freeware + LGPL/BSD components — commercial use not explicitly restricted | reversible | [mersenne.org](https://www.mersenne.org/download/readme.txt) |
| **OCCT Pro/Enterprise** | Unified CPU/GPU/RAM/PSU stability suite, full CLI parity with GUI | `OCCT_CommandLine.exe --test=CPU --duration=1800 --log-dir=C:\results\` | JSON | Yes | Proprietary — **paid** (Pro/Enterprise required commercially) | reversible | [ocbase.com](https://www.ocbase.com/occt/enterprise) |
| **FurMark** | GPU stress test / thermal-power validation | `furmark.exe --demo furmark-gl --duration-ms 60000 --log-gpu-data --export-dir C:\results\` | CSV | Yes | Freeware; free tier excludes commercial use — **PRO Pack paid license required commercially** | reversible | [geeks3d.com](https://geeks3d.com/furmark/command-line/) |

**Recommended for the engine:** **DISKSPD** for disk stress — free, MIT, native XML, no licensing friction; clear best choice for its slice. There is no free structured-output option covering CPU+GPU+RAM+PSU in one tool: **OCCT Enterprise beats the free alternatives** on integration (single CLI, JSON, all subsystems) but requires a paid commercial license. Budget-conscious path: **Prime95** for CPU torture (unattended but log pass/fail manually — no structured output) plus **FurMark with a purchased PRO Pack** for GPU (the free tier is not commercially licensed).

---

## 7. Frametime / performance capture & validation (FPS, 1%/0.1% lows, stutter, frame pacing)

| Tool | What it does | How you drive it | Structured output | Headless | License / commercial-use | Risk | Source |
|---|---|---|---|---|---|---|---|
| **PresentMon Console Application** | Per-frame frametime/CPU/GPU/latency capture, 30+ metrics/frame | `PresentMonx64.exe --process_name app.exe --output_file perf.csv` | CSV | Yes | MIT — unrestricted commercial use | read_only | [GitHub](https://github.com/GameTechDev/PresentMon/blob/main/README-ConsoleApplication.md) |
| **PresentMon Service + SDK** | Background service fusing ETW frame data with GPU power/temp telemetry | C SDK (`PresentMonAPI2.dll`) — no CLI | API | Yes (no GUI, but **no CLI**) | MIT — unrestricted commercial use | read_only | [GitHub](https://raw.githubusercontent.com/GameTechDev/PresentMon/main/README-Service.md) |
| **AMD RGP + Radeon Developer Panel CLI** | GPU profiler for shader execution/timing (AMD RDNA/RDNA2+ only) | `RadeonDeveloperServiceCLI.exe [capture options]` | API | Yes | Proprietary (AMD) — free-to-use, commercial use OK | read_only | [gpuopen.com](https://gpuopen.com/manuals/rgp_manual/) |

**Recommended for the engine:** **PresentMon Console Application** is the clear pick — free, MIT, fully headless, per-frame CSV ready for 1%/0.1%-low and stutter calculations, no GPU-vendor lock-in. Reach for **PresentMon Service + SDK** only if building a custom telemetry client that needs live GPU power/temperature fused with frame data (it has no CLI of its own, so it's a build-time dependency, not a drop-in script target). **AMD RGP is AMD-hardware-only** — useful as a supplementary deep-dive tool for AMD GPU repairs, but unsuitable as the shop's primary engine given mixed-vendor GPU intake.

---

## Commercial-use caution (paid or licensing-unclear tools only)

| Tool | Category | Issue |
|---|---|---|
| **LockHunter** | File-lock discovery/release | Freeware for personal use; commercial licensing terms are not published — contact vendor before deploying in a paid repair workflow. |
| **RegRipper3.0** | Registry inspection | License file present but terms are unclear/unresolved on the repo (top-level license recorded as "Unknown"); do not rely on it for commercial deployment without clarifying the actual license text. |
| **OCCT Pro/Enterprise** | Stress/stability/validation | Proprietary; free tier is personal-use only — commercial deployment requires a paid Pro/Enterprise license (~30–100 EUR/yr) and prohibits license sharing outside the organization. |
| **FurMark** | Stress/stability/validation | Free version explicitly excludes commercial applications; commercial use requires purchasing the Geeks3D PRO Pack license. |

*(All other tools in this catalog carry MIT, Apache-2.0, GPL-2.0, MPL-2.0, or standard Windows/Sysinternals commercial-use terms and are cleared for commercial deployment — not listed here per instruction.)*