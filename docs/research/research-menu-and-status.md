# Candidate research thrusts — held for comparison against panel findings (2026-07-08)

Owner: hold this menu, compare it to what the two running panels find, THEN decide on more.
Two panels running (do NOT re-pick): (1) broad PC-repair primitives, 12 categories;
(2) MCP/CLI-automatable tools (process/handle introspection, file-lock, registry inspection,
text extraction/OCR, HWiNFO64 + sensor alternatives, CLI stress tools).

## A. Fault & symptom knowledge (feeds the frozen de-id grammar / fingerprints)
- A1 ⭐ Bugcheck/stop-code dictionary → STOP_CODE_NAMES. (M)
- A2 ⭐ Event-ID catalog (kernel-power 41, WHEA, disk…) → ID_PREFIXES / symptom vocab. (M)
- A3 Driver/module allowlist → MODULE_NAMES. (M)
- A4 Error-code namespaces (HRESULT/NTSTATUS/Win32/WHEA/XID) → fingerprint + hex grammar. (M)

## B. Diagnostic data sources (unblocks the F4 Windows collector — keystone)
- B1 ⭐ Windows telemetry surfaces (Event Log, ETW/WPR, WMI/CIM, perf counters, Reliability Monitor, WER/minidumps) — what read-only signals exist + how to pull them. (L)
- B2 Crash-dump analysis automation (WinDbg/!analyze headless → signature). (M)

## C. Remediation actions & safety (feeds Risk classes, rollback, destructive gate)
- C1 ⭐ Remediation catalog with reversal semantics (each action + its undo + snapshot-first). (L)
- C2 Hardware-failure / RMA decision criteria per component → EscalatedHardware part_class. (M)
- C3 Malware/PUP remediation protocol. (M)

## D. Legal, licensing & compliance (commercial shop, autonomous repair)
- D1 ⭐ Liability & consent framework (disclosure before autonomous/destructive actions; liability). (L)
- D2 ⭐ Customer-data privacy law (GDPR/CCPA/state; PCI/HIPAA if applicable) — what corpus may retain. (M)
- D3 Shop-wide tool license sweep (commercial-use terms across the fleet). (M)  [partly in panel #2]

## E. AI / agent architecture
- E1 ⭐ Local/on-prem LLM options for the home brain (models, quant, hardware, throughput). (L)
- E2 LLM-as-judge design (calibration, adversarial robustness, panel) → panel crate. (M)
- E3 Prompt-injection / hostile-input defense (customer text, compromised corpus server). (M)
- E4 Case-based-reasoning / retrieval prior art → sanity-check corpus math. (M)

## F. Integration & deployment
- F1 ⭐ MyOwnMesh / AllMyStuff deep-dive (authenticated API + comms) → targets corpus service. (M)
- F2 MCP server design best practices (expose engine tools to a home-brain agent). (M)
- F3 Windows remote-execution substrates (WinRM/PS-Remoting, WMI, PsExec, RMM patterns). (L)
- F4 Binary provenance for downloaded tools (verify sig/hash before download_file runs). (M)

## G. Domain & competitive
- G1 ⭐ RMM / repair-automation landscape (Ninja/Atera/Level/ConnectWise auto-remediation). (M)
- G2 Shop SOPs / community playbooks → seed corpus + validate authoring format. (M)
- G3 Known-good baselines (sensor ranges / perf per config class) → golden baselines. (M)

## H. Bootstrap data
- H1 Public error→fix knowledge bases (MS KB + community) + ingestion licensing. (M)
- H2 Reference device DBs (PCI/USB IDs, driver catalogs). (S)

## My-take shortlist (highest leverage): B1, A1+A2, C1, D1+D2, E1.

## Comparison plan (when panels land)
For each menu item: is it COVERED by a panel finding (drop), PARTIALLY covered (narrow), or a GAP
the panels revealed as important (raise)? Re-rank the shortlist accordingly, then present for owner pick.

## Panel #2 CORRECTION (needed — Opus audit found the verify bar was mis-calibrated)
Root cause: verify conflated "page mentions license" with "tool is CLI-automatable"; also misapplied GPL
(GPL restricts redistribution, NOT commercial use). Fix in the re-run:
- Split verification into TWO independent checks: (1) automation/capability (CLI? structured output? headless?)
  vs (2) license/commercial-use — each may cite a DIFFERENT authoritative page.
- Score GPL/MPL/Apache/MIT commercial-USE as allowed; only flag redistribution/paid tiers.
- Do not reject a real, CLI-capable tool because one page was silent on license.
Reinstate the wrongly-dropped leaders: System Informer, Get-CimInstance Win32_Process|ConvertTo-Json,
handle.exe, PowerToys File Locksmith CLI (--json), Tesseract, LibreHardwareMonitor (HTTP /data.json),
smartctl (-j), DISKSPD, Prime95, MemTest86, stress-ng. (Regipy already held up.)
- **ADD (owner 2026-07-08): CapFrameX** — frametime/perf capture (PresentMon-based; 1%/0.1% lows, stutter).
  Owner says it ships a NATIVE MCP server — VERIFY that against its real repo/docs (MCP surface, capture
  automation, output format, license/commercial-use). Category: performance validation / post-fix perf
  verification (ties to F4 verification + golden baselines), also relevant to GPU stress/validation.

## STATUS 2026-07-08 (corrections complete)
BANKED (converged, artifacts in scratchpad panel1v2-*/panel2v2-*):
- Panel #2 v2 tools shortlist (29 vetted, leaders restored, CapFrameX MCP CONFIRMED real). Licensing: CPU-Z/Prime95/RegRipper = unclear; FurMark free = non-commercial; OCCT commercial = paid; stress-ng reinstated.
- Panel #1 v2 risk-reclassification (8 mislabeled destructive ops fixed) + 13-15 gap primitives w/ correct risk + actionable dedup (-14 dupes).
STILL OPEN (need owner direction / small targeted run — NOT auto-looped):
- Firmware-flash primitive: highest-consequence; rejected on SOURCING not reality; needs a primary-sourced entry.
- Full-catalog destructive-vs-read_only sweep (bootrec /rebuildbcd, bcdedit, Defender cmdlets).
- Missing common primitives: rstrui (System Restore), systemreset (Reset this PC), ipconfig /flushdns family.
- The A–H knowledge menu (owner to pick) — none of A–H covered by the tool panels.
ENGINE: 4 features (retrieval-as-partial, config-transition, retirement, authoring normalizer) pushed on branch @ 73c97e1, green, NO PR opened yet (awaiting owner).
