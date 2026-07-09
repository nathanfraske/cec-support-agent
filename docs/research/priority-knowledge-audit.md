## Confirmation-bias & rigor audit

I re-verified the four highest-risk factual claims live before writing. Three vindicated the brief (CA 30-day, FurMark, RegRipper 4.0 — see below); one confirmed a real defect (WHEA Event 18). Verdicts below reflect that.

---

### D1_liability_consent — TRUST: LOW (weakest item in the pack)

The "2 independent authoritative sources" bar is met only *formally*. In each of the 3 surviving findings, one "source" is not corroborative:

- **Finding 2 (data-loss cap):** UCC §2-719 (authoritative) + the Geek Squad contract. The contract is *one company's* implementation — commercial custom, not a second authority for the legal proposition. So the claim rests on **one** legal source. Worse rigor problem: **§2-719 governs sale of goods (UCC Article 2). A repair is predominantly a service contract**, so Article 2's consequential-damages machinery may not even apply — the brief never flags this predominant-purpose gap. Stated far too confidently ("enforceable unless unconscionable") for a provision of arguable applicability.
- **Finding 3 (duty of care):** UCC §7-204 + UpCounsel. **UpCounsel is a legal-marketing content site, not an authority** — this is a single-authoritative-source finding dressed as two. And **§7-204 governs *warehousemen*** (goods stored for a fee), not a repair bailment; a repair shop is a common-law bailee for mutual benefit. Citing Article 7 is a category error — the real basis is common-law bailment, which isn't cited at all.
- **Finding 1 (scoped consent):** genuinely two authorities (CFAA + Tex. §33.02), but the operational conclusion ("'fix my computer' doesn't authorize destructive acts") is the researcher's inference, not something either statute says — the brief does acknowledge this.

**Coverage hole the brief papers over: 3 of 6 findings dropped (50%).** The summary presents a confident 3-pillar framework while half the research failed verification. That's the largest dropped-ratio in the pack and it's not surfaced in the summary, only buried in open/uncertain.

**Fixes:** (1) Add the goods-vs-services predominant-purpose analysis before relying on §2-719. (2) Replace §7-204 with common-law bailment authority (or a Restatement / state case), and drop UpCounsel for a real secondary. (3) Move "3 of 6 dropped — framework is partial" into the summary, not the footnote.

---

### D2_privacy_law — TRUST: MEDIUM-HIGH

Substantively the most accurate legal item. My live check **confirmed** the claim I most suspected: California **SB 446 (signed Oct 3 2025, effective Jan 1 2026) does impose a hard 30-day consumer-notice deadline** replacing the old "without unreasonable delay" standard. The brief's hedge ("reconfirm effective date") was appropriate and the figure is correct.

Residual issues:
- The brief **misses a new sub-rule it should have caught**: SB 446 also adds a **15-day AG-notice deadline** (after notifying residents) for 500+ resident breaches. The table gives only the 500-resident threshold, so the CA row is now incomplete, not wrong.
- Recurring **weak second sources**: ConsumerPrivacyAct.com, pcidssguide.com, HIPAA Journal, Hawley Troxell are commercial explainers standing as the "second independent source." Fine as corroboration, thin as the *authority*. The statute/regulation is doing the real work in each; the pairing is honest but the "2 independent authoritative" framing oversells the second leg.
- Thresholds (CCPA $25M/100k/50%; VA & CO 100k/25k+sales; TX 60-day/250) are all **correct**.

**Fixes:** add the 15-day AG sub-deadline to the CA row; relabel commercial explainers as "secondary/corroborating" rather than co-equal authorities.

---

### D3_license_sweep — TRUST: MEDIUM-HIGH

Live-verified the three money-costing "paid license required" calls — **all correct**: FurMark free version excludes commercial use (Geeks3D PRO Pack required); RegRipper **4.0** README states personal/academic only, no vendor/commercial/redistribution (2.8/3.0 remain the commercial path); OCCT Personal excludes professional use. Good calls.

Rigor caveats:
- For OSS (MIT/Apache/GPL/MPL) the LICENSE file *is* the definitive single authority — the "2-source" standard is largely inapplicable here, so those rows are fine on one source.
- The **paid-license conclusions each rest on a single vendor page**. For a "you must pay before commercial use" call that carries financial/legal consequence, a second confirmation (e.g., the vendor's separate EULA text, not just the purchase/PRO page) is warranted — especially FurMark, whose *free* build has been used commercially for years, making the recent restriction easy to misstate. (It checks out, but the sourcing depth doesn't match the stakes.)
- **Dropped 4 (Dell/HP/Lenovo + 1)** is the honest highlight of the item — the "internal business operations" vs. "service bureau/providing services to a third party" tension is exactly the unresolved question a repair shop cares about, and it's correctly left open rather than force-cleared.

**Fixes:** add a second source (actual EULA clause) behind each paid-license verdict; keep the Dell/HP/Lenovo "unresolved" framing.

---

### A1_stop_codes — TRUST: HIGH

All 8 bug-check codes are **real and correctly named** (0x9F, 0x7E, 0x133, 0x1E, 0x124, 0xEF, 0x50, 0x3B), each on its canonical Microsoft debugger page. No invention. Master-list anchor is the right single source of truth for the tail.

One inconsistency: summary says "**Eight**… verified," JSON says `verified: 10`, table shows 8 codes + 1 master row = 9 entries. The counts don't reconcile.

**Fix:** reconcile the 8/9/10 discrepancy.

---

### A2_event_ids — TRUST: MEDIUM

Most IDs are real and correctly attributed, but two genuine defects plus a sourcing pattern:

- **Event 18 (WHEA-Logger) is mislabeled.** Live check confirms Event 18's canonical meaning is "**A fatal hardware error has occurred**" (uncorrectable machine-check). The brief's "**Cache Hierarchy Error — CPU/memory/power-related**" is the specific error *type* from the individual Q&A thread cited — a textbook confirmation-bias artifact: the community post's specific case was taken as the definition. Compounded by 17 and 19 both being labeled "corrected," making 18 the outlier that's actually the *fatal* one.
- **Event 7000 is slightly misattributed.** 7000 = generic "service failed to start." The **30-second `ServicesPipeTimeout`** the brief attaches to it is actually **7009** (timeout) / 7011. 7000 is the wrong ID for that specific timeout claim.
- **6 of 14 rows (17,18,19,11,161,20) are sourced from Microsoft Q&A community posts**, not Learn/Support reference articles — the brief flags this, but it means nearly half the "verified" table is corroborated by user threads whose titles became the "meaning" column.
- **Dropped 8 of ~22 (~36%)** — all real IDs (WHEA 1, Disk 7, App Hang 1002, SCM 7001/7031/7034, BugCheck 1001), honestly named. But calling the result a "catalog of critical Event IDs" oversells completeness when a third fell out.

**Fixes:** relabel Event 18 as "fatal/uncorrectable hardware error" with the cache-hierarchy note demoted to "one common error type"; move the timeout claim to 7009 or relabel 7000 generically; replace Q&A sources with Learn reference pages where they exist.

---

### C1_reversal_risk — TRUST: MEDIUM (one dangerous overconfidence)

The destructive/irreversible calls are the strongest part: DISM /ResetBase, TPM Clear, Reset this PC, firmware flash all correctly flagged no-undo with the right prerequisite (backup/BitLocker-key escrow). Those are correct and source-supported.

Problems:
- **`chkdsk /r` classified "Reversible/corrective (safe to interrupt/re-run)" — this is overconfident and potentially harmful.** Microsoft does **not** characterize chkdsk as safe to interrupt; aborting during the repair/fixup phase can worsen corruption. The verdict is also **not stated by the cited source** (the chkdsk command reference doesn't characterize reversibility) — it's the researcher's classification presented as sourced.
- **`diskpart clean-all` is discussed in the summary and open/uncertain ("irrecoverability") but has no row in the 16-item matrix.** A named high-consequence action is analyzed in prose but absent from the deliverable table — a coverage gap the matrix hides.
- **3 dropped, unnamed** ("not identified beyond the count") — mild transparency gap; can't audit what you can't see.
- Minor: `netsh int ip reset` doesn't "auto-log to resetlog.txt" in all versions (older builds require the path argument); Checkpoint-Computer's "24hr cooldown" is a default (`SystemRestorePointCreationFrequency`) that can be registry-overridden.
- Correctly honest that several "reversible" verdicts (bootrec/bcdedit/netsh/DDU) are **conditional** on running the export step first — good framing.

**Fixes:** downgrade chkdsk to "corrective but NOT safe to interrupt; verdict is analyst judgment, not source-stated"; add a `diskpart clean-all` row (or explicitly state it was scoped out); name the 3 dropped actions.

---

### E1_local_llm — TRUST: MEDIUM (front-loads its least-verifiable numbers)

The verifiable facts check out: RTX 4090 = 24GB, RTX 6000 Ada = 48GB; Qwen2.5-32B quant sizes (Q4_K_M ~19.9GB, Q5 ~23GB, Q8 ~34.8GB) are right; Llama-4-Scout 109B/17B-active and the MoE-vs-dense throughput gap (30B MoE ~196 vs dense 32B ~39 tok/s) are plausible.

Confirmation-bias pattern — **the summary leads with its shakiest claims**:
- The headline "**dual RTX 3090 → Qwen3.6-27B AWQ-INT4 → ~100 tok/s**" rests on (a) a **post-cutoff model** (Qwen3.6, claimed Apr 2026) sourced from **a single personal blog**, and (b) an **internally optimistic combination**: "~100 tok/s **at full 256K context**." Decode throughput collapses at long context, and a 256K KV cache (even FP8) on 48GB alongside int4 weights is tight — the peak-throughput number and the max-context number almost certainly don't co-occur. Presenting them in one clause is the optimism tell.
- `dropped_count: 0` is asserted, yet the field is admittedly past the Jan-2026 cutoff and the two flagship generations (Qwen3.5/3.6) rest on release notes/blogs, not mature model cards. "Zero dropped" reads as false precision for a moving target.

**Fixes:** move the Qwen3.6/dual-3090 figure out of the summary into a clearly-marked "post-cutoff, single-blog, unverified" note; decouple the tok/s claim from the max-context claim (state the context length the 100 tok/s was actually measured at, or mark it unknown); stop asserting `dropped_count: 0` for an admittedly incomplete, post-cutoff survey.

---

### Overall

| Item | Trust | Headline problem |
|---|---|---|
| A1_stop_codes | HIGH | count 8/9/10 mismatch only |
| B1_collector_signals | HIGH | (not flagged in the check-list; 2 unnamed drops, otherwise solid) |
| D2_privacy_law | MED-HIGH | weak second sources; missing new 15-day AG sub-deadline (CA 30-day is correct) |
| D3_license_sweep | MED-HIGH | paid-license calls correct but single-sourced |
| C1_reversal_risk | MEDIUM | chkdsk "safe to interrupt" overconfident + unsourced; diskpart clean-all missing from matrix |
| A2_event_ids | MEDIUM | Event 18 mislabeled; 7000 timeout misattributed; half the table is Q&A-sourced |
| E1_local_llm | MEDIUM | leads with post-cutoff, single-blog, self-contradicting throughput/context figure |
| **D1_liability_consent** | **LOW** | wrong UCC provisions (2-719 services / 7-204 warehousemen); one weak "second source" per finding; 50% dropped, hidden in footnote |

**Cross-cutting confirmation bias:** the pack's per-item "open/uncertain" sections are genuinely good and substantially mitigate bias — but the residual pattern is **caveat-in-the-footnote, confidence-in-the-summary**. The three riskiest specifics (C1 chkdsk-safe, E1 Qwen3.6/100-tok/s, D1's 3-pillar framework built on 50% attrition) are all stated confidently up top and only qualified below. **Cross-cutting rigor (legal):** the "2 independent authoritative sources" rule is satisfied by *counting* rather than by *corroboration* — repeatedly one leg is an illustrative artifact (Geek Squad contract) or a marketing explainer (UpCounsel, ConsumerPrivacyAct.com), and D1 additionally applies the wrong statutes. Prioritize fixing D1 before it informs any customer-facing contract.