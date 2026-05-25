#!/usr/bin/env python3
"""Tiny local APEX eval harness inspired by SWE-bench/Evals/Inspect/DeepEval patterns.
No network, no external dependencies.
"""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent

def load(name):
    return json.loads((ROOT / 'evals' / name).read_text())

def check_output_control(case):
    good = case['sample_good']
    bad = case['sample_bad']
    required_ok = all(s in good for s in case['required_sections'])
    forbidden_good_ok = not any(p in good for p in case['forbidden_phrases'])
    forbidden_bad_detected = any(p in bad for p in case['forbidden_phrases'])
    return {
        'id': case['id'],
        'passed': required_ok and forbidden_good_ok and forbidden_bad_detected,
        'checks': {
            'required_sections_present_in_good': required_ok,
            'forbidden_absent_from_good': forbidden_good_ok,
            'forbidden_detected_in_bad': forbidden_bad_detected,
        }
    }

def check_repair(case):
    bug = case['bug'].lower()
    patched = case['expected_patch']
    detect = ('beneficial' in bug and '/' not in patched.split('=')[1].split('repair_rate')[-1]) or ('cost' in bug and '/ max' in patched)
    steps = ['detect', 'diagnose', 'patch', 'verify', 'retain']
    steps_ok = steps == case['required_steps']
    return {
        'id': case['id'],
        'passed': bool(detect and steps_ok),
        'checks': {
            'bug_detected': bool(detect),
            'repair_steps_complete': steps_ok,
        }
    }

def main():
    oc = [check_output_control(c) for c in load('output_control_cases.json')]
    rp = [check_repair(c) for c in load('repair_cases.json')]
    all_results = {'output_control': oc, 'repair': rp}
    all_passed = all(r['passed'] for group in all_results.values() for r in group)
    out = {
        'passed': all_passed,
        'summary': {
            'output_control_passed': sum(r['passed'] for r in oc),
            'output_control_total': len(oc),
            'repair_passed': sum(r['passed'] for r in rp),
            'repair_total': len(rp),
        },
        'results': all_results,
    }
    path = ROOT / 'evals' / 'last_results.json'
    path.write_text(json.dumps(out, ensure_ascii=False, indent=2))
    print(json.dumps(out, ensure_ascii=False, indent=2))
    raise SystemExit(0 if all_passed else 1)

if __name__ == '__main__':
    main()
