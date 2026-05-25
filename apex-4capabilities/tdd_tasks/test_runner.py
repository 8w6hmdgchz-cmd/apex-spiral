"""TDD Green Phase: Minimal eval runner implementation"""
import sys, json, argparse
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent / 'apex-4capabilities'

def run_tests(eval_type):
    if eval_type == 'all':
        eval_type = ['output_control', 'repair']
    else:
        eval_type = [eval_type]
    
    results = {}
    all_passed = True
    
    for et in eval_type:
        if et == 'output_control':
            # Run actual eval
            sys.path.insert(0, str(ROOT.parent / 'apex-self-improve'))
            from run_local_eval import load as l1, check_output_control
            try:
                cases = l1('output_control_cases.json')
                r = [check_output_control(c) for c in cases]
                results['output_control'] = r
                all_passed = all(x['passed'] for x in r)
            except Exception as e:
                results['output_control'] = [{'passed': False, 'error': str(e)}]
                all_passed = False
        elif et == 'repair':
            sys.path.insert(0, str(ROOT.parent / 'apex-self-improve'))
            from run_local_eval import load as l2, check_repair
            try:
                cases = l2('repair_cases.json')
                r = [check_repair(c) for c in cases]
                results['repair'] = r
                all_passed = all_passed and all(x['passed'] for x in r)
            except Exception as e:
                results['repair'] = [{'passed': False, 'error': str(e)}]
                all_passed = False
    
    summary = {}
    for k, v in results.items():
        passed = sum(1 for x in v if x.get('passed'))
        total = len(v)
        summary[f'{k}_passed'] = passed
        summary[f'{k}_total'] = total
    
    return {"passed": all_passed, "summary": summary, "results": results}

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--eval', default='all', choices=['output_control', 'repair', 'all'])
    args = parser.parse_args()
    
    result = run_tests(args.eval)
    print(json.dumps(result, indent=2, ensure_ascii=False))
    sys.exit(0 if result['passed'] else 1)
