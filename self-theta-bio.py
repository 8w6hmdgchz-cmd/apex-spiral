#!/usr/bin/env python3
"""
开智V6 - 自动验证模块 (v1)
Θ_bio = f(外部真实指标) → 不依赖人工反馈
通过追踪客观指标变化来验证进化是否有效
"""
import json, math
from pathlib import Path
from datetime import datetime, timezone, timedelta

STATE_DIR = Path("/Users/lihongxin/.openclaw/workspace/a2a-resources/state")
LOG_DIR = Path("/Users/lihongxin/.openclaw/workspace/a2a-resources")

def log(msg):
    print(f"[{datetime.now().strftime('%Y-%m-%d %H:%M:%S')}] {msg}", file=__import__('sys').stderr)

def load_jsonl(file_path):
    """读取jsonl文件，返回列表"""
    if not file_path.exists():
        return []
    lines = file_path.read_text(errors='ignore').splitlines()
    result = []
    for line in lines:
        if line.strip():
            try:
                result.append(json.loads(line))
            except:
                pass
    return result

def compute_theta_bio():
    """
    计算Θ_bio（真实反馈）
    Θ_bio = 系统自我验证反馈，不依赖人工
    """
    scores = []
    weights = []
    
    # ========== 指标1: Cron任务成功率 ==========
    cron_success = get_cron_success_rate()
    scores.append(cron_success)
    weights.append(0.25)
    log(f"  Cron任务成功率: {cron_success:.3f}")
    
    # ========== 指标2: φ_history变化趋势 ==========
    phi_trend = get_phi_trend()
    scores.append(phi_trend)
    weights.append(0.25)
    log(f"  φ变化趋势: {phi_trend:.3f}")
    
    # ========== 指标3: evolution_score连续变化率 ==========
    evo_rate = get_evolution_rate()
    scores.append(evo_rate)
    weights.append(0.25)
    log(f"  进化速率: {evo_rate:.3f}")
    
    # ========== 指标4: 自我修复有效性 ==========
    fix_rate = get_fix_effectiveness()
    scores.append(fix_rate)
    weights.append(0.25)
    log(f"  修复有效率: {fix_rate:.3f}")
    
    # 加权平均
    total = sum(s * w for s, w in zip(scores, weights))
    weight_sum = sum(weights)
    theta_bio = total / weight_sum if weight_sum > 0 else 0.0
    
    return round(theta_bio, 4), dict(zip(["cron", "phi_trend", "evo_rate", "fix_rate"], scores))

def get_cron_success_rate():
    """
    Cron任务成功率
    检查OpenClaw cron最近运行状态
    """
    cron_run_file = LOG_DIR / "cron/runs/bfc8cccb-f2aa-4fb1-b955-a42cfb54ed58.jsonl"
    if not cron_run_file.exists():
        return 0.5  # 未知状态
    
    entries = load_jsonl(cron_run_file)
    if not entries:
        return 0.5
    
    # 看最近10条
    recent = entries[-10:] if len(entries) >= 10 else entries
    successes = sum(1 for e in recent if e.get("status") == "finished")
    
    return successes / len(recent) if recent else 0.5

def get_phi_trend():
    """
    φ_history变化趋势
    如果φ一直在涨→高反馈
    如果φ来回跳动→低反馈
    如果φ稳定不变→中等反馈
    """
    phi_file = STATE_DIR / "phi_history.jsonl"
    entries = load_jsonl(phi_file)
    
    if len(entries) < 4:
        return 0.5  # 数据不足
    
    # 取最近8个phi值
    phi_vals = [float(e.get("phi", 0)) for e in entries[-8:]]
    
    # 计算趋势：上涨=高，震荡=低，稳定=中
    # 简单判断：标准差小且均值高 → 稳定在高位=好
    mean_phi = sum(phi_vals) / len(phi_vals)
    std_phi = math.sqrt(sum((p - mean_phi)**2 for p in phi_vals) / len(phi_vals))
    
    # 标准差小(<=0.3)且均值高(>=7.5) → 好
    if std_phi <= 0.3 and mean_phi >= 7.5:
        return 0.9
    # 标准差大(>0.5) → 来回震荡 → 差
    elif std_phi > 0.5:
        return 0.3
    # 标准差中等 → 一般
    else:
        return 0.6

def get_evolution_rate():
    """
    evolution_score连续变化率
    持续上升→高反馈，停滞→低反馈
    """
    evo_file = STATE_DIR / "evolution_log.jsonl"
    entries = load_jsonl(evo_file)
    
    if len(entries) < 3:
        return 0.5
    
    # 取最近5条的evolution_score
    scores = []
    for e in entries[-5:]:
        if "metrics" in e:
            try:
                scores.append(float(e["metrics"].get("evolution_score", 0)))
            except:
                pass
    
    if len(scores) < 2:
        return 0.5
    
    # 计算变化方向
    # 简单判断：最近3条是否在上升
    recent = scores[-3:]
    if len(recent) >= 2:
        if recent[-1] > recent[0]:
            return 0.8  # 在上升
        elif recent[-1] == recent[0]:
            return 0.4  # 停滞
        else:
            return 0.3  # 下降
    
    return 0.5

def get_fix_effectiveness():
    """
    自我修复有效性
    检查fix_log：修复后问题是否再出现
    """
    fix_file = STATE_DIR / "fix_log.jsonl"
    entries = load_jsonl(fix_file)
    
    if len(entries) < 2:
        return 0.5
    
    # 检查是否有"修复后又出现同类问题"的情况
    # 简单判断：fix次数 vs 修复成功的次数
    fixes = [e for e in entries if "fixes" in e]
    
    if not fixes:
        return 0.5
    
    # 统计fix action类型
    fix_actions = []
    for f in fixes:
        for fix in f.get("fixes", []):
            fix_actions.append(fix.get("action"))
    
    # 有"fix"成功的记录 → 高
    # 只有"request_feedback" → 低
    if "fix_" in str(fix_actions):
        return 0.7
    elif "request_feedback" in str(fix_actions):
        return 0.4
    else:
        return 0.5

def update_theta_bio_history(theta_bio, components):
    """写入theta_bio历史"""
    theta_file = STATE_DIR / "theta_bio_history.jsonl"
    
    entry = {
        "ts": int(datetime.now(timezone(timedelta(hours=8))).timestamp()),
        "theta_bio": theta_bio,
        "components": components,
        "source": "auto_verification"
    }
    
    with theta_file.open("a") as f:
        f.write(json.dumps(entry, ensure_ascii=False) + "\n")
    
    return entry

def main():
    log("=" * 50)
    log("🔬 开智V6 自动验证模块")
    log("=" * 50)
    
    theta_bio, components = compute_theta_bio()
    
    log(f"\nΘ_bio(自动验证) = {theta_bio:.4f}")
    log(f"  组成: cron={components['cron']:.3f}, phi_trend={components['phi_trend']:.3f}, evo_rate={components['evo_rate']:.3f}, fix_rate={components['fix_rate']:.3f}")
    
    # 写入历史
    entry = update_theta_bio_history(theta_bio, components)
    
    log(f"\n已写入theta_bio_history")
    log("=" * 50)
    
    # 输出JSON
    print(json.dumps(entry, ensure_ascii=False))

if __name__ == "__main__":
    main()
