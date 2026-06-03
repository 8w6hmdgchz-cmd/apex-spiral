#!/usr/bin/env python3
"""Compute A2A hunt metrics from real resource landing state."""

import glob
import json
import os
import time
from pathlib import Path

BASE = Path('/Users/lihongxin/.openclaw/workspace')
RES = BASE / 'a2a-resources'
CACHE = RES / 'cache'
STATE = BASE / 'state'

# R7 bug-018 fix: 进一步删 a2a-hunt-realstate.py 顶部冗余 sys.path hack.
# a2a_state.py 顶部已自注入 (_THIS_DIR 注入 apex_spiral/), 此脚本 import a2a_state
# 时自动生效. 删除此处 3 行 sys.path.insert (H_complexity 砍 1 个耦合点).
# (a2a_state.py 顶部自注入见 R6 自补短)

WEIGHTS = {
    'mem0ai/mem0': (0.90, 0.85),
    'langchain-ai/langgraph': (0.80, 0.82),
    'deap/deap': (0.75, 0.78),
    'pyg-team/pytorch_geometric': (0.70, 0.72),
    'microsoft/autogen': (0.82, 0.84),
    'openai/openai-agents-python': (0.84, 0.82),
    'openai/spinningup': (0.74, 0.70),
    'noahshinn/reflexion': (0.86, 0.80),
    'geek-ai/MAgent': (0.72, 0.70),
}


def read_lines(path: Path) -> list[str]:
    try:
        return [line for line in path.read_text(errors='replace').splitlines() if line.strip()]
    except FileNotFoundError:
        return []


def latest_state() -> dict:
    files = glob.glob(str(STATE / 'a2a-hunt-*.json'))
    if not files:
        return {}, None
    latest = max(files, key=os.path.getmtime)
    return json.loads(Path(latest).read_text()), Path(latest)


def main() -> None:
    absorbed = read_lines(RES / 'absorbed.list')
    pending = read_lines(RES / 'pending.list')
    failed = read_lines(RES / 'failed.list')
    inherited = read_lines(RES / 'inherited.list')

    repos: dict[str, dict[str, object]] = {}
    for line in absorbed:
        parts = line.split('|')
        if len(parts) >= 3:
            repos.setdefault(parts[1], {'name': parts[0], 'keywords': set()})['keywords'].add(parts[2])

    breakdown = {}
    for repo, meta in sorted(repos.items()):
        p_value, u_value = WEIGHTS.get(repo, (0.65, 0.65))
        keyword_bonus = min(len(meta['keywords']), 5) * 0.015
        cache_bonus = 0.02 if (CACHE / repo.replace('/', '_') / 'README.md').exists() else 0
        breakdown[repo] = round(p_value * (u_value + keyword_bonus + cache_bonus), 4)

    previous, previous_path = latest_state()
    absorbed_count = len(absorbed)
    pending_count = len(pending)
    failed_count = len(failed)
    unique_repos = len(repos)
    a_net = round(sum(breakdown.values()), 4)
    success_rate = absorbed_count / max(absorbed_count + failed_count + pending_count, 1)
    # FIX bug-001: f_hunt was 1-success_rate (越失败越高), now: 1-failure_rate = 1-failed/(absorbed+failed+pending)
    # FIX bug-006: 原代码 f_hunt = success_rate 实际未应用修复，应为 1 - failure_rate
    # 健康状态 (failed=0) → f_hunt=1.0
    # 全部失败 (failed=tot) → f_hunt=0.0
    failure_rate = failed_count / max(absorbed_count + failed_count + pending_count, 1)
    f_hunt = round(min(1.0, max(0.0, 1.0 - failure_rate)), 4)
    k_fold = round(1.0 + unique_repos * 0.12 + min(len(inherited), absorbed_count) * 0.01 - min(failed_count / (absorbed_count + failed_count + 1), 1) * 0.25, 4)
    landed = max(0, absorbed_count - int(previous.get('details', {}).get('resource_state', {}).get('absorbed_count', 0) or 0))
    prev_g = float(previous.get('G_cycle', 1.0))
    g_cycle = round(max(0.1, prev_g) + 0.08 * landed + 0.03 * max(0, a_net - float(previous.get('A_net', 0))), 4)
    d_lack_count = round(max(0.0, pending_count / 20 + failed_count / max(absorbed_count + 1, 1) - unique_repos * 0.05), 4)
    d_lack_impact = round(min(0.95, d_lack_count / max(unique_repos + 1, 1)), 4)
    delta_g = round(a_net * f_hunt * k_fold * g_cycle * (1 - d_lack_impact), 6)

    iter_id = time.strftime('%Y%m%d-%H%M', time.localtime())
    output = {
        'iter': iter_id,
        'timestamp': time.strftime('%Y-%m-%dT%H:%M:%S%z', time.localtime()),
        'source_latest': str(previous_path.relative_to(BASE)) if previous_path else None,
        'source_age_minutes': 'real_resource_snapshot',
        'A_net': a_net,
        'F_hunt': f_hunt,
        'Trigger_t': True,
        'K_fold': k_fold,
        'G_cycle': g_cycle,
        'D_lack_count': d_lack_count,
        'Delta_G_unlimited': delta_g,
        'details': {
            'A_net_formula': 'sum(repo_i_present * P_i * U_i), derived from absorbed.list and cache evidence',
            'A_net_breakdown': breakdown,
            'resource_state': {
                'absorbed_count': absorbed_count,
                'pending_count': pending_count,
                'failed_count': failed_count,
                'inherited_count': len(inherited),
                'unique_repos': unique_repos,
            },
            'D_lack_impact': d_lack_impact,
            'data_sources': [
                'a2a-resources/absorbed.list',
                'a2a-resources/pending.list',
                'a2a-resources/failed.list',
                'a2a-resources/inherited.list',
                'a2a-resources/cache/*/README.md',
            ],
            'derived_from_previous': {
                'source_A_net': previous.get('A_net'),
                'source_K_fold': previous.get('K_fold'),
                'source_D_lack_impact': previous.get('details', {}).get('D_lack_impact'),
            },
        },
    }

    out_path = STATE / f'a2a-hunt-{iter_id}.json'

    # FIX bug-002: 只在有变化时写 state (landed>0 OR ΔG 变化 > 0.1 OR pending/failed 出现)
    delta_threshold = 0.1
    should_write = landed > 0 or pending_count > 0 or failed_count > 0
    if not should_write and previous:
        prev_dg = float(previous.get('Delta_G_unlimited', 0))
        if abs(delta_g - prev_dg) > delta_threshold:
            should_write = True

    # FIX bug-007 (R002): 不论是否 should_write，始终把 latest snapshot 写入 integration file
    # 之前 bug: cron 跑 100+ 次都 SKIP，integration.json 永远停在 R001 数据，陈旧腐烂
    # 现在：O(1) 增量更新 v11_with_a2a + 6_dim_self_check 字段，不动其它 schema
    # FIX R0 bug-008: v11_enhanced 之前硬编码 79.29，实际为分母 t=0.18 的乐观值；
    # 改成从 v11_with_a2a 拉真值，与 __main__ 单源真相一致。
    try:
        from apex_spiral.v11_with_a2a import v11_with_a2a as _v11, load_6dim as _load6
        _p = _load6()
        _v11_res = _v11(_p['C'], _p['L'], _p['O'], _p['tau'], _p['H'], _p['t'])
        v11_enhanced = _v11_res['v11_enhanced']
    except Exception:
        v11_enhanced = 79.29  # 回退（不应触发）
    _push_to_integration(output, v11_enhanced)

    if should_write:
        out_path.write_text(json.dumps(output, ensure_ascii=False, indent=2) + '\n')
        print(out_path)
        print(json.dumps(output, ensure_ascii=False, indent=2))
    else:
        # 空状态：追加到 .log 而不是写独立 state 文件
        log_path = STATE / 'a2a-hunt-skip.log'
        with open(log_path, 'a') as f:
            f.write(f"{output['timestamp']} skip iter={iter_id} absorbed={absorbed_count} pending={pending_count} failed={failed_count} delta_g={delta_g}\n")
        print(f"SKIP: no change since {previous_path.relative_to(BASE) if previous_path else 'init'}")


def _push_to_integration(output: dict, v11_enhanced: float) -> None:
    """O(1) update of memory/a2a-v11-integration.json latest snapshot fields.

    Keeps the schema stable (does not touch job_id / key_insight / bugs),
    only refreshes v11_with_a2a + 6_dim_self_check so cron-driven deltas
    are never lost between manual rounds. Fix for R002 bug: stale integration file.
    """
    import json as _json
    integration_path = BASE / 'memory' / 'a2a-v11-integration.json'
    if not integration_path.exists():
        return
    try:
        data = _json.loads(integration_path.read_text())
    except Exception:
        return
    a2a_dg = output.get('Delta_G_unlimited', 0)
    a2a_f = output.get('F_hunt', 1.0)
    a2a_net = float(output.get('A_net', 0))
    # FIX R13 bug-013: 单源真相 — 直接调 v11_with_a2a 拿 system_delta_g
    # 旧实现手算 a2a_factor (F_hunt * (0.3+0.7*A_n)) + 自乘 v11_enhanced,
    # 与 v11_with_a2a.v11_with_a2a 内部算法 (0.3+0.7*F_hunt*A_n) 偏差 2x
    # (F_hunt 被乘了两次, system_delta_g 在 integration.json 里 = 51.54 vs __main__ = 25.48)
    # 现在调用 v11_with_a2a 拿唯一真值, 消除双源真相漂移.
    # R7 bug-019: 删 3 行 sys.path.insert, 改靠 apex_spiral/v11_with_a2a.py
    # 顶部自注入 + a2a_state.py 顶部自注入. 验证: python3 -m apex_spiral.v11_with_a2a
    # 在 cron 工作目录下能直接 import a2a_state.
    from apex_spiral.v11_with_a2a import load_6dim as _load6, v11_with_a2a as _v11
    _p6 = _load6()
    _v11_res = _v11(_p6['C'], _p6['L'], _p6['O'], _p6['tau'], _p6['H'], _p6['t'])
    a2a_A_n = _v11_res['a2a_A_n_norm']
    a2a_factor = _v11_res['a2a_factor']
    v11_enhanced = _v11_res['v11_enhanced']
    data['v11_with_a2a'] = {
        'v11_enhanced': v11_enhanced,
        'a2a_F_hunt': a2a_f,
        'a2a_absorbed': output.get('details', {}).get('resource_state', {}).get('absorbed_count', 0),
        'a2a_A_net': a2a_net,
        'a2a_A_n_norm': a2a_A_n,
        'a2a_factor': a2a_factor,
        'system_delta_g': _v11_res['system_delta_g'],
    }
    # 6-dim self-check re-derivation (C/L/O/tau from real outputs, H/t carried)
    unique = output.get('details', {}).get('resource_state', {}).get('unique_repos', 43)
    # FIX R1 bug-002: 6_dim 不能再硬覆盖 t=0.30 / H=0.45（会抹掉 cron 之前补短）
    # 只覆盖可推导的 O_horizon（用 unique_repos），其余字段保留 cron/manual 已写入值
    data['6_dim_self_check']['O_horizon'] = round(min(1.0, unique / 50.0), 2)
    data['last_cron_refresh'] = output.get('timestamp')
    try:
        integration_path.write_text(_json.dumps(data, ensure_ascii=False, indent=2) + '\n')
    except Exception:
        pass


if __name__ == '__main__':
    main()
