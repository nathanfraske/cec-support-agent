# Local-agent infrastructure

> Current as of **2026-06-14** (fresh live-probe recon). The local-inference stack
> **changed substantially in the days before this date** — see
> [What changed recently](#what-changed-recently). If anything here disagrees with
> older docs (`local-compute-exploration.md`, `self-hosted-router.md`), this document
> wins.

This box runs a **hybrid** local-LLM setup behind a **single front door**. No agent,
pipeline, or script ever talks to a model server directly: everything sends an
OpenAI-compatible request — with the `model` field set to a *broker alias* — to one
reverse proxy that routes, boots backends on demand, and arbitrates the single GPU.

---

## The single front door: `cec-llm-broker`

`cec-llm-broker` is an **OpenAI-compatible, on-demand model orchestrator / reverse
proxy** listening on `0.0.0.0:8080`. It is the only thing clients address. It runs as
the **SYSTEM systemd unit `cec-llm-broker.service`** (enabled, auto-restart). On the
last probe it was up ~1.5 days, PID 122048, having served 917 requests with 16
backend starts and 7 evictions.

What it does:

- **Routes by model alias.** The request's `"model"` field selects a `models.json`
  entry. The broker proxies to that backend's host port and **rewrites the alias to
  the backend's real served name** before forwarding.
- **Boots managed backends on demand.** If a *managed* WSL backend is cold, the broker
  runs `docker compose --profile <p> up -d <svc>` and polls its health URL.
  **Never `compose up/down` an LLM service by hand** — the broker owns that lifecycle.
- **Arbitrates the single RTX 5090.** The card has 32 GB; the broker enforces a
  **30 GB VRAM budget** (`gpu_budget_gb`). Before starting a backend it checks
  `vram_gb` against what is already resident; if the start would overflow the budget it
  **evicts the least-recently-used (LRU) GPU backend first** (after letting that
  backend's in-flight generations drain, `drain_timeout_s` = 180 s).
- **Reaps idle backends.** Any managed backend idle longer than `idle_stop_s`
  (**30 min**) is stopped to free VRAM.
- **Proxies — but never manages — external seats.** External Windows-native seats are
  `managed:false`: the broker proxies to them, counts their VRAM in arbitration, but
  **never starts, stops, reaps, or evicts them**.
- **Catalog always answers.** `GET /v1/models` lists the full registry with a
  per-model `running` flag even when nothing is up (doubles as a broker-liveness probe).
- **Ride-through.** Concurrent requests for a cold model block on a single shared
  start — nobody double-starts, nobody 503s during a multi-minute cold load.

**Fail-safe:** a backend that won't come up surfaces an error; the client's own
deterministic fallback (e.g. `cec_judge_local`) then takes over. A broker that is down
yields connection-refused, which clients also treat as "fall back."

### API

| Method | Path | Purpose |
|---|---|---|
| GET | `/v1/models` (or `/models`) | full catalog `{data:[{id, running, ...}]}` |
| POST | `/v1/chat/completions` | route by `model`, boot-on-demand, proxy/stream |
| POST | `/v1/completions`, `/v1/embeddings` | same routing, generic proxy |
| GET | `/health` | liveness |
| GET | `/broker/stats` | running backends, GPU budget/use, request counts |

The broker honors an `X-CEC-Client` request header for stats attribution.

---

## The hybrid backends

Two kinds of backend sit behind the broker.

### (a) Managed WSL2 docker-compose seats

Defined in `CEC-Platform/docker/compose.yaml`, started/stopped by the broker via
`docker compose`. Engines are **vLLM** (the `cec-judge` AWQ seat) and **llama.cpp
`server-cuda`** (everything else). Their **GGUFs live on `E:/AI Models`**, read through
the WSL **`/mnt/e` drvfs** mount — which is **slow**, so cold loads take **minutes**.
MoE seats run `-ngl 99` with `--n-cpu-moe` to keep experts in host RAM and shrink the
GPU footprint. **The broker owns the lifecycle of these seats; never bring them up or
down by hand.**

### (b) External Windows-native `llama-server` seats

`managed:false` seats served by native Windows `llama-server.exe` reading the same
GGUFs from `E:` at **native NTFS speed** (this skips the drvfs cold-load tax). The
broker resolves `host: windows-host` to the **WSL default gateway `172.27.192.1`** and
proxies to them; it **does not** start, stop, reap, or evict them. They are launched
**Windows-side only** via Scheduled Tasks (not WSL-spawnable). Binaries live on
`E:/llama-cpp-win/` (llama.cpp **b9611**, **CUDA 13.3**, pinned by sha256 in
`versions.env`).

### Registered aliases (10) — live registry `models.json`

> The registry now contains **8 managed WSL seats + 2 external Windows-native seats**.
> (Older README text listing "8 registered models (2026-06-12)" pre-dates the two
> external entries being added — the live `models.json` below is authoritative.)

| Alias | Backend | Port | Managed | Model / role | VRAM budget |
|---|---|---|---|---|---|
| `cec-judge` | vLLM (WSL) | 8000 | yes | Qwen3-Coder-30B-A3B-AWQ — original routing-loop judge | 27 GB |
| `cec-worker` | llama.cpp (WSL) | 8002 | yes | Qwen3.6-35B-A3B UD-Q4_K_M — high-throughput volume worker | 24 GB |
| `cec-worker-quality` | llama.cpp (WSL) | 8004 | yes | Qwen3.6-27B dense Q4_K_M — max per-call quality | 19 GB |
| `cec-vision-judge` | llama.cpp (WSL) | 8006 | yes | Qwen3-VL-32B-Instruct + mmproj — vision judge | 21 GB |
| `cec-worker-vision` | llama.cpp (WSL) | 8012 | yes | **Qwen3.6-35B-A3B + mmproj — DEFAULT unified text+vision seat** | 10 GB |
| `cec-worker-quality-vision` | llama.cpp (WSL) | 8014 | yes | Qwen3.6-27B + mmproj — quality unified seat | 20 GB |
| `cec-manager` | llama.cpp (WSL) | 8003 | yes | MiniMax-M2.7 229B-A10B UD-Q3_K_XL — **retired from CEC paths**, kept registered for the other project | 22 GB |
| `cec-manager-fast` | llama.cpp (WSL) | 8005 | yes | gpt-oss-120b MXFP4 — **DEFAULT reviewer tier** (experts in RAM, ~6 GB GPU) | 8 GB |
| `cec-worker-vision-win` | llama-server (Windows-native) | 8090 | **no** | Qwen3.6-35B-A3B + mmproj at NTFS speed — phase-B migration target; **NOT reachable now** | 25 GB |
| `deepseek-v4-flash` | llama-server (Windows-native) | 8007 | **no** | **DeepSeek-V4-Flash-284B Q4_K_M-XL — deep auditor (T5); LIVE now** | 13 GB |

> **Note on `cec-worker-vision` VRAM:** lowered 25 → 10 GB on 2026-06-13 so it can
> co-reside with V4-Flash. When V4 is co-resident the compose command runs
> `--n-cpu-moe 99` (experts in host RAM) so its GPU footprint is ~8–10 GB instead of
> ~24 GB. Set `CEC_VISION_NCPUMOE=0` and raise `vram_gb` back to 25 when V4 is unloaded.

### What is LIVE vs cold vs unreachable (probe 2026-06-14)

- **LIVE:** `deepseek-v4-flash` (`:8007`, **Windows-native deep auditor**). It is the
  *only* running backend. Routed experts sit in host RAM (~135–160 GB); attention/KV
  sit on the 5090 (~10–13 GB measured), so `nvidia-smi` shows ~14.6 / 32 GB used.
- **Cold:** **every managed WSL docker seat.** No docker containers are running; each
  would be booted by the broker on its first request (expect a multi-minute drvfs cold
  load).
- **Unreachable:** `cec-worker-vision-win` (`:8090`). It is registered but does **not**
  answer. The **`libomp140.dll` `System32` `ENTRYPOINT_NOT_FOUND` load blocker** from
  2026-06-12 is still unresolved (needs one elevated `System32` replace). Because of
  this, the **WSL `cec-worker-vision` (`:8012`) remains the live vision/worker
  fallback** — the worker-vision migration to Windows-native has **not** actually
  happened yet.

> **GPU co-residency consequence:** because the broker counts but never evicts the
> external V4 seat (~13 GB), and the budget is 30 GB, a managed seat larger than
> ~17 GB **cannot** co-reside with V4 on the 32 GB card. The route worker uses no GPU
> seat and the 8 GB `gpt-oss` reviewer coexists with V4 fine.

---

## How an agent or pipeline reaches it

Send a normal **OpenAI-compatible request** whose `model` field is a **broker alias**:

- From the **host (WSL/Windows)**: `http://localhost:8080/v1`
- From **inside a container** (e.g. the routing container): `http://host.docker.internal:8080/v1`

The broker looks the alias up in `models.json`. For a **managed WSL seat** it
`docker compose up`s the profile (evicting the LRU GPU backend if the 30 GB budget
would overflow), polls health, then proxies and streams the reply; the seat is reaped
after 30 min idle. For an **external Windows-native seat** it resolves
`host: windows-host` → `172.27.192.1`, proxies if the seat answers, and otherwise
returns an error. **Everything is fail-safe:** broker or seat down → connection-refused
→ the client falls back to its deterministic path.

CEC pipeline tiers pick their alias + endpoint through env knobs in
`scripts/cec_judge_local.py` (`CEC_VLLM_URL`, `CEC_VLLM_WORKER/MANAGER/REVIEWER_URL`,
`CEC_VLLM_MODEL_NAME`): **default seat `cec-worker-vision`, default reviewer
`cec-manager-fast`, opt-in deep auditor `deepseek-v4-flash`** (dispatched via
`scripts/cec_v4_task.py`). Right now only the V4 seat (`:8007`) is up; every WSL seat
is cold and gets booted on first request.

### For THIS repo (`cec-support-agent` / `crates/inference`)

`crates/inference` **is** the OpenAI-compatible HTTP client. Point it at the broker:

```
--endpoint http://localhost:8080/v1 --model <alias>
```

Use the alias for the tier you want, e.g.:

- `--model cec-worker-vision` — default unified text+vision seat (WSL `:8012`)
- `--model cec-manager-fast` — default reviewer (gpt-oss-120b, WSL `:8005`)
- `--model deepseek-v4-flash` — deep auditor (Windows-native `:8007`, live now)

In-container, swap `localhost` for `host.docker.internal`. **Cold-start works with no
endpoint configured at all**: if no endpoint is supplied the client defaults to the
broker on `8080` and the broker boots the seat on demand — and if the broker/seat is
unreachable the client falls back to its deterministic path.

---

## What changed recently

- **Broker rebuilt from spec, v2 (2026-06-12).** A WSL move to `E:` wiped the original
  `cec-llm-broker`; the model files survived on `E:/AI Models`. The broker was rebuilt
  as a **stdlib-only on-demand orchestrator** from the CEC `CLAUDE.md` spec. Older docs
  describe the simpler earlier proxy (vLLM `cec-judge` 8000 + a 235B `cec-manager`
  llama.cpp on 8001) — that design is gone.
- **Windows-native serving migration opened (2026-06-12).** Motivated by killing the
  drvfs cold-load tax. The broker gained **`managed:false` external-backend support**
  (proxy-only; never start/stop/reap; count-but-never-evict VRAM). Two external seats
  were registered.
- **Deep auditor is now DeepSeek-V4-Flash-284B (`:8007`).** It is the **first and only
  working Windows-native seat** and replaced MiniMax-M2.7 as the deep/T5 tier
  (owner decision 2026-06-11). It is running now.
- **`cec-worker-vision-win` (`:8090`) registered but not reachable** — the
  `libomp140` `System32` blocker is still open; the **WSL `cec-worker-vision` (`:8012`)
  stays the live fallback**.
- **Model lineup churn:**
  - **Qwen3-235B manager retired** + service removed and files deleted (2026-06-09).
  - **MiniMax-M2.7 (`cec-manager` 8003) retired from CEC paths** (2026-06-11) but kept
    registered for the other project.
  - **Default reviewer is now `cec-manager-fast` (gpt-oss-120b, `:8005`).**
  - **Default seat unified to `cec-worker-vision`** (text+vision); its `vram_gb`
    lowered 25 → 10 (2026-06-13) for V4 co-residency via `--n-cpu-moe 99`.
- **Broker is now a SYSTEM systemd unit** (enabled, auto-restart), deployed by
  `ops/provision.sh` from the vendored `ops/cec-llm-broker` copy (WSL-ephemeral
  durability policy).
- **Claude RC / interactive session survivability rebuilt onto tmux + systemd
  auto-reup (2026-06-14)** (`claude-rc-tmux.sh`, `claude-session.sh`, `rc-recover.sh`;
  new `ops/secrets` bot identity + a `pre-push` guard).
- **`versions.env` grew a windows-serving section** pinning llama.cpp `b9611` /
  CUDA `13.3` with a sha256 per binary.

---

## Operating it

**Service control (the broker is a SYSTEM unit, hence `sudo`):**

```bash
sudo systemctl status cec-llm-broker
sudo systemctl restart cec-llm-broker
journalctl -u cec-llm-broker -f
```

**Health / inspection (no privileges needed):**

```bash
curl -s http://localhost:8080/health
curl -s http://localhost:8080/v1/models      # full catalog + per-model running flag
curl -s http://localhost:8080/broker/stats   # running backends, GPU budget/use, counts
```

**Dev / foreground run:** `python3 broker.py`.

**Where the source lives:**

- **Working copy:** `/home/nathan/cec-llm-broker/` — `broker.py`, `models.json`
  (the registry / source of truth for the 10 aliases), `cec-llm-broker.service`,
  `README.md`.
- **Installed unit:** `/etc/systemd/system/cec-llm-broker.service`.
- **Durable vendored copy:** `/home/nathan/CEC-Platform/ops/cec-llm-broker/`, deployed
  to the working copy + installed as the unit by `ops/provision.sh` (WSL is ephemeral,
  so the vendored copy is the durable one).

**Key files:**

| File | Purpose |
|---|---|
| `/home/nathan/cec-llm-broker/broker.py` | the orchestrator/proxy |
| `/home/nathan/cec-llm-broker/models.json` | registry: alias → port/served/vram/backend/managed/host; `gpu_budget_gb`, `idle_stop_s`, `cold_load_timeout_s`, `compose_file` |
| `/home/nathan/CEC-Platform/docker/compose.yaml` | the 8 managed WSL seats (profiles → ports) |
| `/home/nathan/CEC-Platform/ops/windows-serving/` | external-seat launchers (`run-worker-vision.bat`, `install-task.ps1`, `close-dialogs.ps1`, README) |
| `/home/nathan/CEC-Platform/scripts/cec_judge_local.py` | CEC client adapter / seat resolver (env knobs, fail-safe fallback) |
| `/home/nathan/CEC-Platform/scripts/cec_v4_task.py` | DeepSeek-V4-Flash deep-reasoner dispatcher (recovers answers from `reasoning_content`) |
| `/home/nathan/CEC-Platform/versions.env` | pins (incl. windows-serving: llama.cpp b9611 / CUDA 13.3 + sha256) |

**Config knobs:** `models.json` (`gpu_budget_gb`, `idle_stop_s`, per-alias `vram_gb`).
Env overrides: `CEC_BROKER_PORT`, `CEC_BROKER_HOST`, `CEC_BROKER_REGISTRY`,
`CEC_BROKER_UPSTREAM_HOST`. Windows-native seats are operated **Windows-side only** via
the `CEC-WorkerVision` / V4 logon Scheduled Tasks (`install-task.ps1` run once,
elevated); they are not WSL-spawnable.

---

## Open questions / known-unverified

These are carried straight from the recon and are **not yet confirmed**:

- **`cec-worker-vision-win` (`:8090`) `libomp140` blocker.** Is the `System32`
  `libomp140.dll` fix still pending (elevated action not taken), or has the owner
  decided to leave worker-vision on WSL? Disk evidence says blocked; the live probe
  confirms it is down. The WSL `:8012` seat is the working fallback meanwhile.
- **WSL networking mode.** Currently **NAT** (gateway `172.27.192.1`). **Mirrored**
  networking is explicitly deferred pending an owner decision.
- **AI-box upgrade undecided.** The Threadripper PRO 9965WX / 2× RTX 5090 / 256 GB
  upgrade is an analysis doc, **not a committed purchase** — unclear if ordered. It
  would change the single-GPU swap-arbitration design.
- **V4 launcher lives on Windows `E:`.** The `deepseek-v4-flash` launcher
  (`E:/toolchain/run-v4-flash.bat`) and the V4 llama.cpp fork build
  (`E:/toolchain/llama.cpp-v4`) live on the Windows `E:` drive, not in repo/WSL, and
  could not be inspected — only the registry note documents them.
- **`cec_v4_queue` idle-queue worker not traced.** The V4 idle-queue (with
  `pending/`/`done/` dirs) is referenced but its worker script was not inspected; how
  V4 idle tasks are dispatched/drained is not fully understood.
- **`ik_llama.cpp` MoE-offload Windows-native build gated on CUDA 12.8+** (box has
  12.1); no on-disk evidence it was resolved, so those seats remain WSL-only.
- **Native-binary pins unverified.** `versions.env` pins the Windows binaries by
  sha256, but no native-binary verification step was found running on this box;
  whether the running V4/native binaries match the pins is unverified from WSL.
```
