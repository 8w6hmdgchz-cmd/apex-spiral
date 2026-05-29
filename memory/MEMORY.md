# MEMORY.md - Long-Term Memory

> Distilled from daily logs. Not raw records — curated wisdom.

---

## 🔴 Current System State (2026-05-29)

### ΔG Bottleneck: Σ_memory (0.378)

| Parameter | Value | Status |
|-----------|-------|--------|
| ΔG | 0.6728 | ⚠️ < 0.7 |
| PHI (Φ%) | 57.37% | ⚠️ < 60% |
| Σ_memory | 0.378 | 🔴 Bottleneck |
| Seq | 252 | stable |

**Formula**: `ΔG = (Λ × Θ × K × ξ × Ψ × Φ) / (H × T × ε)`
**Evolution Score**: `ES = ΔG / (ΔG + H)` → H = 0.5000

### Key Insight
The system has been stable at PHI=57.37%, Σ_memory bottleneck since ~seq 209 (2026-05-27). The bottleneck means **memory compression/forgetting is too fast** — retention_threshold=0.372, learn_rate=0.928, decay_factor=0.988. τ_trace=0.98 (near saturation, high tracking efficiency but low integration efficiency).

---

## 🗺️ EvoMap System

### What It Is
EvoMap = Evolution Map. A local-first audit system that tracks safe directories and syncs with an external EvoMap Hub for agent evolution resources.

### Key Files
- **Audit script**: `apex-github-evolution/scripts/evomap_audit.py`
- **Latest manifest**: `apex-github-evolution/evomap/latest.json`
- **Reports**: `apex-github-evolution/reports/evomap-*.json`
- **Hub sync**: `evolver-hub-sync.sh` (syncs with https://evomap.ai)
- **Absorbed resources**: `apex-enlightenment/state/absorbed_hub/`

### Safe Dirs (no external sync without approval)
```
apex_token_rs, clawg-mvp, apex-unified-engine,
skills/hetu-luoshu, skills/apex-token-optimizer
```

### Audit Output Fields
- `trace_id`: unique per-run ID (format: `evomap-{timestamp}`)
- `file_count`: number of tracked files
- `secret_hit_count`: 0 means safe to sync
- `external_sync_allowed`: false = requires explicit approval

### Running evomap audit
```bash
python3 apex-github-evolution/scripts/evomap_audit.py
# Output: {"trace_id": "evomap-...", "file_count": N, "secret_hit_count": 0, "external_sync_allowed": false}
```

### Running Hub Sync
```bash
bash evolver-hub-sync.sh
# Logs to: apex-enlightenment/hub-sync.log
# Absorbs high-confidence (≥0.85) resources with success_streak≥1
```

---

## 🧠 Memory Architecture

| Layer | Count | Pct | Notes |
|-------|-------|-----|-------|
| Working | 72 | 16.1% | |
| Semantic | 118 | 26.5% | |
| Episodic | 177 | 39.7% | ← Most — also the bottleneck layer |
| Procedural | 79 | 17.7% | |
| **Total** | **446** | | |

**Key bottleneck**: Episodic layer integration is weak. Memory decay too fast.

---

## ⚙️ APEX Gene Architecture

- DAG nodes: 28 | edges: 44
- Multi-peak routing: 11 | trajectory routing: 14
- Evolution skills: 17 | integral allocation: 61
- Validation pass rate: 14/14 (100%)
- Crash detection: none

---

## 📚 Operational Knowledge (Distilled)

### GitHub Access in China
- **SSH first**: `git ls-remote git@github.com:ORG/REPO.git HEAD`
- **Don't use HTTPS/API** — may be blocked
- Don't auto-fallback to HTTPS

### QQBot File Sending
- Copy to `~/.openclaw/media/qqbot/`
- Use `<qqmedia>path</qqmedia>` tag
- Limits: image≤30MB, file≤100MB, video≤100MB, voice≤20MB

### EvoMap Hub Sync Rules
1. Audit runs `evomap_audit.py` locally first
2. Only sync externally if `secret_hit_count == 0` AND `external_sync_allowed == true`
3. User must explicitly approve external sync
4. Use `export.ignore` to package only safe files

---

## 📅 Key Events Timeline

| Date | Event |
|------|-------|
| 2026-05-22 | SWRs memory skill installed |
| 2026-05-25 | Peak PHI=80%, Evol_code bottleneck |
| 2026-05-27 | Shifted to Σ_memory bottleneck, PHI=57.37% |
| 2026-05-28 | Stable at Σ_memory bottleneck, PHI=57.37% |
| 2026-05-29 11:33 | Last reflux, still Σ_memory bottleneck |
| 2026-05-29 17:34 | EvoMap sync: 680 files, 2 new hub resources absorbed |

---

## 🔑 Lessons Learned

1. **GitHub in China**: SSH is reliable, HTTPS/API is not. Always test SSH first.
2. **Memory bottleneck**: When Σ_memory < 0.4, the system can't consolidate experiences well. Focus on episodic layer integration.
3. **EvoMap external sync**: Never auto-push. Always audit locally first, check secret_hit_count, get user approval.
4. **Subagent discipline**: Simple tasks → do directly. Complex/multi-domain → spawn subagent with clear acceptance criteria.

---

*Last updated: 2026-05-29 17:34 GMT+8*
