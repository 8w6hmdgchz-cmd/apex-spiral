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
    f_hunt = round(min(1.0, max(0.0, 1 - success_rate + pending_count / max(absorbed_count + pending_count + 1, 1))), 4)
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
    out_path.write_text(json.dumps(output, ensure_ascii=False, indent=2) + '\n')
    print(out_path)
    print(json.dumps(output, ensure_ascii=False, indent=2))


if __name__ == '__main__':
    main()
