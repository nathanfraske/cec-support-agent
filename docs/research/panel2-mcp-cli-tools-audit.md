## Confirmation-bias & research-quality audit

**Overall trust verdict: MODERATE — the catalog is safe to ship as a shortlist, but the structured verdict records are not trustworthy and three items need action.** The original inversion defect (license-mention treated as automation proof) is **genuinely fixed** at the decision level: every accepted tool has a real CLI/headless surface, and the GUI-only / library-only tools (Text Extractor, ShowWhatProcessLocksFile, python-registry, PresentMon SDK) are correctly identified and disqualified *in prose*. But a **new, subtler bias replaced part of it** — over-rejection on a single "structured-output format" technicality, applied inconsistently — plus several verdict records whose stated reasoning contradicts the recorded facts. Details below.

### (1) Are the category leaders present?
Mostly yes, with one real miss.
- **Process introspection:** native CIM-JSON present (Get-CimInstance). *System Informer* is absent, but it is GUI-first with only a thin CLI, so its omission is defensible. **OK.**
- **File locks:** PowerToys File Locksmith CLI ✓ and handle.exe ✓ both present. **OK.**
- **OCR:** Tesseract ✓. **OK.**
- **Sensors:** LibreHardwareMonitor ✓ and smartctl ✓. **OK.**
- **Stress:** DISKSPD ✓, Prime95 ✓, but **stress-ng is wrongly rejected** — see (3)/(5). **PARTIAL FAIL.**

### (2) Is CapFrameX's MCP-server claim substantiated, or asserted?
**Substantiated by a real source** — I confirmed it in the CapFrameX README: CapFrameX ships an **in-process HTTP MCP server** hosted by `CapFrameX.exe`, reachable at `http://localhost:<port>/mcp` and registerable with `claude mcp add … --transport http`. So the rejection's underlying *claim* ("does host a genuine MCP server") is true and sourced, and its *stated reason* (no `CapFrameX.exe --mcp` CLI flag exists) is also correct — there is no such flag; it's an HTTP endpoint, not a CLI switch.
**But this exposes a scoping gap:** the catalog is titled "**MCP**/CLI-Automatable" and CapFrameX is the *only* tool in the entire set with a first-class, documented MCP server aimed at exactly this use case. Rejecting it purely because the *invocation string* was wrong (should be "run CapFrameX, register the HTTP endpoint," not `--mcp`) throws out the most on-topic tool on a syntax technicality. It should be **re-added to §7 as an MCP-integration option** (read_only; caveat: requires the CapFrameX app running; not a headless one-shot).

### (3) Any automation surface still OVERSTATED (GUI sold as CLI)?
**No accepted tool is GUI-only-sold-as-CLI** — that specific bias is corrected. However there are two integrity problems in the *verdict records*:
- **`automation_supported: true` is set on tools that are explicitly NOT automatable** — ShowWhatProcessLocksFile, PowerToys Text Extractor, PresentMon Service+SDK, and python-registry all carry `automation_cli:false` yet `verdict.automation_supported:true`. The prose disqualifies them correctly, but any downstream consumer trusting the structured field would wrongly conclude Text Extractor is scriptable. The field is being used to mean "the claim matches the source" rather than "the tool is automatable." **Relabel these `automation_supported:false`.**
- **AMD RGP is mildly overstated:** `RadeonDeveloperServiceCLI.exe` can trigger a *capture* headlessly, but the profile is a binary RGP file that still requires the **GUI RGP tool** to analyze; "api" output + AMD-only is a fair flag, but the entry reads as more headless-end-to-end than it is. Downgrade note only.

### (4) Licensing errors
GPL is handled correctly (smartctl GPL-2.0 → commercial use OK — good, not mistreated as non-commercial). But four problems remain:
- **RegRipper3.0 — fabricated license in the verdict.** The record's top-level fields correctly say *"Unknown / unclear,"* and GitHub itself reports *"Unknown, Unknown licenses found"* (I confirmed). Yet `verdict.reason` asserts with full confidence: *"MIT License — commercial use is unrestricted."* That is invented and contradicts the very same record. The catalog's caution table lands on the right answer ("unclear"), so the *decision* is fine, but the audit trail is unreliable. **Fix the reason.**
- **FurMark — verdict contradicts its own catalog and under-verifies.** The catalog table correctly says the free tier excludes commercial use and a **Geeks3D PRO Pack** is required (I confirmed the PRO Pack is real and is the commercial license). But the `verdict.commercial_use_note` says the PRO Pack *"could not be verified"* and *"commercial use of the free version appears permitted."* The conclusion in the table is correct; the verdict reasoning is wrong and internally contradictory. **Fix the note.**
- **CPU-Z — optimism error.** Marked `commercial_use:"yes"` and **omitted from the commercial-use caution table**, yet its own note admits *"the freeware version's commercial-use policy is not clearly specified"* and *"freeware licenses typically restrict commercial use unless explicitly permitted."* This is exactly the old pattern (freeware-mention → assumed commercial-OK). CPUID does sell a separate commercial license/SDK. **Downgrade `commercial_use` to `unclear` and add it to the caution table** alongside LockHunter.
- **Prime95 — borderline optimism.** `commercial_use:"yes"` while the note hedges ("not entirely explicit"). The GIMPS/Mersenne license is genuinely ambiguous for commercial use. Not a hard error, but should read `unclear`, not `yes`.

### (5) Coverage gaps and the residual methodology bias
The most important finding sits here. Three rejections share one flawed pattern: **a tool with a genuinely working CLI/headless surface is failed on `automation_supported` solely because one *output-format* attribute was overstated — and the rule is applied inconsistently.**
- **stress-ng — rejected on a factually false premise.** The rejection says *"No JSON output… text-based metrics output only."* This is **wrong**: stress-ng supports `-Y/--yaml <file>` (and JSON) structured output with `--metrics` (confirmed). It is a genuine, industry-standard, headless CLI stressor and a category leader you named. Its exclusion also leaves the stress category with **no free RAM-validation tool at all** (DISKSPD=disk, Prime95=CPU, OCCT/FurMark=paid) — `stress-ng --vm` fills that hole. **Re-add stress-ng (structured_output: yaml).** This is the single clearest error in the set.
- **Get-Process | ConvertTo-Json and PowerShell Select-String — inconsistent standard.** Both were rejected because "ConvertTo-Json is a separate cmdlet not on the source page." Yet the catalog **accepts `Get-CimInstance … | ConvertTo-Json` and `Get-ItemProperty … | ConvertTo-Json` using the identical pipe.** Same construct, opposite rulings. Either the pipe is valid automation (it is) or it isn't. Select-String is a real, headless log-search CLI and should be accepted with `text` output exactly as findstr was. These aren't hard coverage gaps (Get-CimInstance and findstr cover the use cases), but the double standard is the residual bias and undermines confidence in the gate.
- **RAM stress validation** is genuinely uncovered by any *accepted* free tool (MemTest86's headless config is Pro-only, so that rejection is legitimate). stress-ng would mitigate.

### Concrete action list
**Re-add:**
1. **stress-ng** — rejection rests on a false "no structured output" claim; it has YAML/JSON + metrics. Fills the missing free CPU/RAM/disk stressor slot.
2. **CapFrameX (MCP integration)** — the only tool with a native, documented MCP server; belongs in an "MCP/CLI" catalog. Note: requires the app running; correct the invocation from `--mcp` to "register the `/mcp` HTTP endpoint."
3. *(Optional)* **Select-String** with `text` output, for parity with the accepted findstr and the accepted ConvertTo-Json pipes.

**Downgrade / relabel:**
4. **CPU-Z** → `commercial_use: unclear`; add to the commercial-use caution table.
5. **Prime95** → `commercial_use: unclear`.
6. **RegRipper3.0** → rewrite the verdict reason to match the recorded "Unknown" license (delete the fabricated "MIT" assertion).
7. **FurMark** → rewrite the verdict note to match the (correct) catalog table: PRO Pack confirmed required for commercial use.
8. **ShowWhatProcessLocksFile, PowerToys Text Extractor, PresentMon Service+SDK, python-registry** → set `verdict.automation_supported: false` so the structured field stops contradicting the prose.
9. **AMD RGP** → add a note that analysis still requires the GUI RGP tool (CLI only triggers capture).

**Cosmetic:** handle.exe is counted twice (one entry per category), so "29 tools" is really ~28 distinct binaries — fine, but state it's counted per-category.

**Net:** the headline recommendations (Get-CimInstance, File Locksmith + handle.exe, RECmd/Regipy, Tesseract, LibreHardwareMonitor + smartctl, DISKSPD, PresentMon Console) are all sound and correctly reasoned. The old inversion is fixed. Trust the catalog's *prose picks*; **do not** trust the per-tool `verdict.automation_supported` fields or the RegRipper/FurMark/CPU-Z license reasoning without the fixes above.

Sources: [CapFrameX README](https://github.com/CXWorld/CapFrameX/blob/master/README.md), [Geeks3D PRO Pack](https://geeks3d.com/g3dpp/), [stress-ng repo/issue #45](https://github.com/ColinIanKing/stress-ng/issues/45), [RegRipper3.0 repo](https://github.com/keydet89/RegRipper3.0).