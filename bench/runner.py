#!/usr/bin/env python3
"""
APEX Benchmark Runner v1
真跑10任务出基线：调用大模型生成代码 → 执行 → 比对期望输出
"""

import json
import time
import traceback
import signal
import os
from typing import Any, Callable
from dataclasses import dataclass, asdict
import urllib.request
import urllib.error

# ========== 配置 ==========
API_ENDPOINT = "https://token-plan-cn.xiaomimimo.com/v1/chat/completions"
API_KEY = "tp-c7vjjat3tu3wtwt229dg4ojkl85ydc2f5azaei9yiaq1nrh3"
MODEL = "mimo-v2.5-pro"
BENCHMARK_FILE = "/Users/lihongxin/.openclaw/workspace/bench/apex_benchmark_v1.yaml"
OUTPUT_FILE = "/Users/lihongxin/.openclaw/workspace/bench/baseline_v1.json"
FAILURE_LOG = "/Users/lihongxin/.openclaw/workspace/memory/failure_cases.jsonl"

# ========== 数据结构 ==========
@dataclass
class TaskResult:
    task_id: str
    passed: bool
    runtime_sec: float
    error_msg: str
    generated_code: str
    test_results: list = None  # 详细测试结果

@dataclass
class TestCaseResult:
    inputs: Any
    expected: Any
    actual: Any
    passed: bool
    error: str = ""

# ========== YAML 解析 ==========
import yaml

def parse_yaml_raw(filepath: str) -> dict:
    """使用PyYAML解析benchmark文件"""
    with open(filepath, 'r', encoding='utf-8') as f:
        data = yaml.safe_load(f)
    return data

# ========== LLM 调用 ==========
def call_llm(prompt: str, timeout: int = 30) -> str:
    """调用大模型生成代码"""
    payload = {
        "model": MODEL,
        "messages": [{"role": "user", "content": prompt}],
        "max_tokens": 2048,
        "temperature": 0.2
    }
    
    data = json.dumps(payload).encode('utf-8')
    req = urllib.request.Request(
        API_ENDPOINT,
        data=data,
        headers={
            'Content-Type': 'application/json',
            'api-key': API_KEY
        },
        method='POST'
    )
    
    try:
        with urllib.request.urlopen(req, timeout=timeout) as response:
            result = json.loads(response.read().decode('utf-8'))
            if 'choices' in result and len(result['choices']) > 0:
                return result['choices'][0]['message']['content']
            else:
                raise Exception(f"Unexpected response format: {result}")
    except urllib.error.HTTPError as e:
        error_body = e.read().decode('utf-8') if e.fp else str(e)
        raise Exception(f"HTTP {e.code}: {error_body}")
    except Exception as e:
        raise Exception(f"API call failed: {e}")

def extract_code(response: str) -> str:
    """从LLM响应中提取Python代码"""
    # 尝试从markdown代码块中提取
    lines = response.split('\n')
    code_lines = []
    in_code_block = False
    
    for line in lines:
        if line.strip().startswith('```'):
            if in_code_block:
                # 代码块结束
                break
            else:
                in_code_block = True
                # 跳过 ```python 或 ```
                if line.strip() == '```' or line.strip() == '```python':
                    continue
        elif in_code_block:
            code_lines.append(line)
    
    if code_lines:
        return '\n'.join(code_lines)
    
    # 如果没有代码块，返回整个响应
    return response

# ========== 代码执行与评分 ==========
class TimeoutError(Exception):
    pass

def timeout_handler(signum, frame):
    raise TimeoutError("Code execution timed out")

def compare_values(a: Any, b: Any, atol: float) -> bool:
    """比较两个值是否相等"""
    if isinstance(a, (list, tuple)) and isinstance(b, (list, tuple)):
        if len(a) != len(b):
            return False
        return all(compare_values(ai, bi, atol) for ai, bi in zip(a, b))
    if isinstance(a, (int, float)) and isinstance(b, (int, float)):
        return abs(a - b) <= atol
    if isinstance(a, bool) or isinstance(b, bool):
        return bool(a) == bool(b)
    if isinstance(a, str) and isinstance(b, str):
        return a.strip() == b.strip()
    return a == b

def run_test(code: str, task: dict) -> tuple[bool, str, list]:
    """执行代码并测试所有用例"""
    entry_point = task['entry_point']
    test_cases = task['test_cases']
    atol = task.get('atol', 0)
    timeout_sec = task.get('timeout', 5)
    
    # 编译代码
    try:
        compiled = compile(code, '<ast>', 'exec')
    except SyntaxError as e:
        return False, f"Syntax Error: {e}", []
    
    # 执行代码
    ns = {}
    try:
        exec(compiled, ns)
    except Exception as e:
        return False, f"Compile/Exec Error: {e}", []
    
    if entry_point not in ns:
        return False, f"Function '{entry_point}' not found", []
    
    fn = ns[entry_point]
    test_results = []
    all_passed = True
    
    for tc in test_cases:
        inputs = tc['inputs']
        expected = tc['expected']
        
        try:
            if isinstance(inputs, list):
                result = fn(*inputs)
            else:
                result = fn(inputs)
        except Exception as e:
            test_results.append(TestCaseResult(
                inputs=inputs,
                expected=expected,
                actual=None,
                passed=False,
                error=str(e)
            ))
            all_passed = False
            continue
        
        passed = compare_values(result, expected, atol)
        test_results.append(TestCaseResult(
            inputs=inputs,
            expected=expected,
            actual=result,
            passed=passed,
            error="" if passed else f"Expected {expected}, got {result}"
        ))
        
        if not passed:
            all_passed = False
    
    return all_passed, "", test_results

# ========== 主流程 ==========
def run_benchmark():
    """运行基准测试"""
    print("=" * 60)
    print("APEX Benchmark Runner v1 - 真跑10任务出基线")
    print("=" * 60)
    
    # 解析benchmark
    print("\n[1/4] 解析benchmark文件...")
    benchmark = parse_yaml_raw(BENCHMARK_FILE)
    tasks = benchmark.get('tasks', [])
    print(f"    加载了 {len(tasks)} 个任务")
    
    results = []
    
    # 确保failure log目录存在
    failure_log_dir = os.path.dirname(FAILURE_LOG)
    if failure_log_dir:
        os.makedirs(failure_log_dir, exist_ok=True)
    
    # 运行每个任务
    print("\n[2/4] 调用大模型生成代码...")
    for idx, task in enumerate(tasks):
        task_id = task['id']
        print(f"\n    [{idx+1}/{len(tasks)}] 任务: {task_id} ({task.get('difficulty', '?')})")
        
        start_time = time.time()
        generated_code = ""
        error_msg = ""
        passed = False
        test_results = []
        
        try:
            # 构建prompt
            prompt = f"""请实现以下Python函数：

{task['prompt']}

请只输出Python代码，不要包含解释说明。用```python和```包裹代码块。

注意：
- 函数签名必须完全按照描述实现
- 返回值类型必须正确
- 代码必须可以直接运行
"""
            
            # 调用LLM
            print(f"        调用LLM中...")
            llm_response = call_llm(prompt, timeout=60)
            generated_code = extract_code(llm_response)
            print(f"        代码生成成功 ({len(generated_code)} 字符)")
            
            # 运行测试
            print(f"        执行测试中...")
            passed, error_msg, test_results = run_test(generated_code, task)
            
            if passed:
                print(f"        ✅ 通过 ({len(test_results)}/{len(test_results)})")
            else:
                print(f"        ❌ 失败: {error_msg[:80] if error_msg else '测试用例未全部通过'}")
                
        except Exception as e:
            error_msg = str(e)
            print(f"        ❌ 错误: {error_msg[:80]}")
        
        runtime = time.time() - start_time
        
        result = TaskResult(
            task_id=task_id,
            passed=passed,
            runtime_sec=round(runtime, 2),
            error_msg=error_msg,
            generated_code=generated_code,
            test_results=[asdict(tr) for tr in test_results] if test_results else []
        )
        results.append(result)
        
        # 记录失败案例
        if not passed:
            failure_entry = {
                "task_id": task_id,
                "prompt": task['prompt'],
                "generated_code": generated_code,
                "error_msg": error_msg,
                "test_results": [asdict(tr) for tr in test_results] if test_results else [],
                "timestamp": time.strftime("%Y-%m-%d %H:%M:%S")
            }
            with open(FAILURE_LOG, 'a', encoding='utf-8') as f:
                f.write(json.dumps(failure_entry, ensure_ascii=False) + '\n')
            print(f"        📝 已记录到failure_cases.jsonl")
    
    # 生成报告
    print("\n[3/4] 生成基线报告...")
    total = len(results)
    passed_count = sum(1 for r in results if r.passed)
    pass_rate = passed_count / total if total > 0 else 0
    avg_runtime = sum(r.runtime_sec for r in results) / total if total > 0 else 0
    
    report = {
        "benchmark": "apex_benchmark_v1",
        "version": "1.0",
        "total_tasks": total,
        "passed": passed_count,
        "pass_rate": round(pass_rate, 4),
        "average_runtime_sec": round(avg_runtime, 2),
        "results": [asdict(r) for r in results]
    }
    
    # 写入JSON
    with open(OUTPUT_FILE, 'w', encoding='utf-8') as f:
        json.dump(report, f, ensure_ascii=False, indent=2)
    
    print(f"    报告已保存到: {OUTPUT_FILE}")
    
    # 打印摘要
    print("\n[4/4] 基线结果摘要")
    print("=" * 60)
    print(f"  总任务数: {total}")
    print(f"  通过数:   {passed_count}")
    print(f"  通过率:   {pass_rate:.1%}")
    print(f"  平均耗时: {avg_runtime:.2f}秒")
    print("=" * 60)
    
    # 详细结果
    print("\n各任务结果:")
    for r in results:
        status = "✅" if r.passed else "❌"
        print(f"  {status} {r.task_id}: {r.runtime_sec}s - {r.error_msg[:40] if r.error_msg else 'OK'}")
    
    return report

if __name__ == "__main__":
    try:
        report = run_benchmark()
        print("\n✅ Benchmark完成!")
    except Exception as e:
        print(f"\n❌ Benchmark失败: {e}")
        traceback.print_exc()
        exit(1)
