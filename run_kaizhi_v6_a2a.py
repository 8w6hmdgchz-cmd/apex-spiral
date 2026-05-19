#!/usr/bin/env python3
import json, subprocess, sys
from pathlib import Path

STATE = Path('/Users/lihongxin/.openclaw/workspace/apex-enlightenment/state')
BASE = Path('/Users/lihongxin/.openclaw/workspace/apex-enlightenment')
EVO = STATE / 'evolution_log.jsonl'

last_dg = None
for line in EVO.read_text(errors='ignore').splitlines():
    if not line.strip():
        continue
    try:
        obj = json.loads(line)
    except Exception:
        continue
    if 'dg' in obj:
        last_dg = obj['dg']

if last_dg is None:
    print('NO_DG', file=sys.stderr)
    sys.exit(1)

print(f'DG={last_dg}')
cmds = [
    ['bash', str(BASE / 'self-evolve.sh'), str(last_dg)],
    ['python3', str(BASE / 'self-a2a-integrate.py')],
    ['python3', str(BASE / 'self-check-shortboard.py')],
    ['python3', str(BASE / 'self-fix.py')],
]

for cmd in cmds:
    print('\n>>>', ' '.join(cmd), flush=True)
    r = subprocess.run(cmd)
    if r.returncode != 0:
        sys.exit(r.returncode)
