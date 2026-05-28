#!/usr/bin/env python3
"""Gist状态上传脚本，供 gist-sync.yml workflow 调用"""
import json
import os
import subprocess

def main():
    file_list = [
        'counter.txt',
        'score-state.env',
        'latest-report.md',
        'state/phi_history.jsonl',
        'state/defect_history.jsonl',
        'state/repair_history.jsonl',
        'state/bug_streak.jsonl',
        'state/consistency_log.jsonl',
        'state/lesson_bank.jsonl',
        'state/metacognition_log.jsonl',
    ]
    
    files = {}
    for f in file_list:
        if os.path.isfile(f):
            try:
                with open(f) as fh:
                    files[os.path.basename(f)] = {'content': fh.read()}
            except Exception:
                pass
    
    desc = 'APEX Evolver State ' + subprocess.check_output(['date']).decode().strip()
    result = {
        'description': desc,
        'public': False,
        'files': files
    }
    
    outpath = os.environ.get('GITHUB_ENV', '/tmp/gist_payload.json')
    with open(outpath, 'w') as fh:
        json.dump(result, fh)
    
    print(f'Prepared {len(files)} files -> {outpath}')

if __name__ == '__main__':
    main()
