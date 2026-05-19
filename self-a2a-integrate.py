#!/usr/bin/env python3
"""
开智V6 - A2A资源整合模块 (v1)
读取A2A猎食结果 → 注入Φ_all → 更新短板状态
"""
import json
from pathlib import Path
from datetime import datetime, timezone, timedelta

STATE_DIR = Path("/Users/lihongxin/.openclaw/workspace/apex-enlightenment/state")
DESKTOP_KAIZHI = Path.home() / "Desktop/开智"

def log(msg):
    print(f"[{datetime.now().strftime('%Y-%m-%d %H:%M:%S')}] {msg}", file=__import__('sys').stderr)

def read_a2a_results():
    """读取A2A猎食结果"""
    state_file = DESKTOP_KAIZHI / "a2a_open_source_evolver_state.json"
    report_file = DESKTOP_KAIZHI / "a2a_open_source_evolver_report.md"
    
    results = {
        "found_resources": [],
        "credit_balance": None,
        "top_candidates": [],
        "last_run_ts": None
    }
    
    if state_file.exists():
        try:
            state = json.loads(state_file.read_text())
            
            # 正确的字段路径
            a2a_data = state.get("a2a", {})
            results["credit_balance"] = a2a_data.get("credit_balance")
            results["last_run_ts"] = state.get("timestamp")
            
            # selected数组是TOP资源
            selected = state.get("selected", [])
            results["top_candidates"] = [
                (s["name"], s.get("fitness", 0), s.get("ssh_ok", False))
                for s in selected
            ]
            # ssh_ok=true的算"已吸收"
            results["found_resources"] = [s["name"] for s in selected if s.get("ssh_ok", False)]
            
        except Exception as e:
            log(f"读取A2A state失败: {e}")
    
    # 从report提取信息
    if report_file.exists():
        try:
            content = report_file.read_text()
            lines = [l for l in content.splitlines() if l.strip()]
            if lines:
                results["report_summary"] = lines[0] if lines else ""
        except:
            pass
    
    return results

def compute_phi_a2a(a2a_results):
    """
    根据A2A猎食结果计算Φ_a2a
    Φ_a2a = 已吸收资源数 / 候选资源总数 × 质量系数
    """
    found = len(a2a_results.get("found_resources", []))
    top = len(a2a_results.get("top_candidates", []))
    credit = a2a_results.get("credit_balance") or 0
    
    if top == 0:
        return 0.0, "无A2A猎食结果"
    
    # 吸收率
    absorb_rate = found / max(top, 1)
    
    # 积分充足度
    credit_score = min(credit / 30.0, 1.0)  # 30积分为满分
    
    # 综合Φ_a2a
    phi_a2a = absorb_rate * 0.6 + credit_score * 0.4
    
    reason = f"已吸收{found}/{top}资源, 积分{credit:.2f}"
    return round(phi_a2a, 3), reason

def update_shortboard_with_a2a(a2a_results, phi_a2a):
    """用A2A结果更新短板扫描"""
    sb_file = STATE_DIR / "shortboard_scan.jsonl"
    
    if not sb_file.exists():
        return
    
    try:
        lines = sb_file.read_text(errors='ignore').splitlines()
        if not lines:
            return
        
        latest = json.loads(lines[-1])
        
        # 更新xi_self：吸收了A2A资源算"真实进步"
        if a2a_results.get("found_resources"):
            # 有资源被吸收 → Ξ_我提升
            current_xi = latest.get("xi_self", 0)
            # 每吸收一个资源，Ξ提升0.05，上限0.5
            new_resources = len(a2a_results.get("found_resources", []))
            xi_boost = min(new_resources * 0.05, 0.3)
            latest["xi_self"] = round(min(current_xi + xi_boost, 0.5), 4)
            latest["xi_boost_reason"] = f"A2A吸收{new_resources}个资源"
        
        # 更新phi_vals，加一个A2A相关的phi值
        phi_vals = latest.get("phi_vals", [])
        if phi_a2a > 0:
            phi_vals.append(phi_a2a)
            latest["phi_vals"] = phi_vals[-10:]  # 保留最近10个
            latest["phi_a2a_injected"] = phi_a2a
        
        latest["a2a_integration"] = {
            "found": len(a2a_results.get("found_resources", [])),
            "top": len(a2a_results.get("top_candidates", [])),
            "credit_balance": a2a_results.get("credit_balance"),
            "phi_a2a": phi_a2a
        }
        
        # 追加回去（更新最后一条）
        lines[-1] = json.dumps(latest, ensure_ascii=False)
        sb_file.write_text("\n".join(lines) + "\n")
        
    except Exception as e:
        log(f"更新shortboard失败: {e}")

def generate_a2a_shortboard(a2a_results, phi_a2a):
    """生成A2A相关的短板条目"""
    issues = []
    
    credit = a2a_results.get("credit_balance") or 0
    if credit < 5:
        issues.append({
            "id": "a2a_low_credits",
            "category": "evolution",
            "desc": f"A2A积分不足({credit:.2f})，猎食受限",
            "score": 0.8,
            "reasons": [f"积分{credit:.2f}<5"]
        })
    
    found = len(a2a_results.get("found_resources", []))
    top = len(a2a_results.get("top_candidates", []))
    if top > 0 and found == 0:
        issues.append({
            "id": "a2a_not_absorbed",
            "category": "evolution",
            "desc": f"A2A找到{top}个候选但0个吸收",
            "score": 0.7,
            "reasons": [f"吸收率0/{top}"]
        })
    
    if phi_a2a < 0.3:
        issues.append({
            "id": "phi_a2a_low",
            "category": "evolution",
            "desc": f"Φ_a2a={phi_a2a:.3f}，外部资源整合不足",
            "score": 0.6,
            "reasons": [f"phi_a2a={phi_a2a}"]
        })
    
    return issues

def main():
    log("=" * 50)
    log("🔗 开智V6 A2A资源整合模块")
    log("=" * 50)
    
    # 读取A2A猎食结果
    a2a_results = read_a2a_results()
    
    log(f"A2A状态: credit={a2a_results.get('credit_balance')}, found={len(a2a_results.get('found_resources',[]))}, top={len(a2a_results.get('top_candidates',[]))}")
    
    # 计算Φ_a2a
    phi_a2a, reason = compute_phi_a2a(a2a_results)
    log(f"Φ_a2a={phi_a2a} ({reason})")
    
    # 生成A2A相关短板
    a2a_issues = generate_a2a_shortboard(a2a_results, phi_a2a)
    
    if a2a_issues:
        log("A2A相关短板:")
        for issue in a2a_issues:
            log(f"  - {issue['id']}: {issue['desc']} (得分:{issue['score']})")
    else:
        log("无A2A相关短板")
    
    # 更新shortboard
    update_shortboard_with_a2a(a2a_results, phi_a2a)
    
    # 整合报告
    summary = {
        "phi_a2a": phi_a2a,
        "credit_balance": a2a_results.get("credit_balance"),
        "found": len(a2a_results.get("found_resources", [])),
        "top": len(a2a_results.get("top_candidates", [])),
        "a2a_issues": a2a_issues,
        "a2a_top_resources": a2a_results.get("found_resources", [])[:3]
    }
    
    log(f"\n整合摘要: Φ_a2a={phi_a2a}, credit={a2a_results.get('credit_balance')}, found={len(a2a_results.get('found_resources',[]))}")
    log("=" * 50)
    
    # 输出JSON（供父流程读取）
    print(json.dumps(summary, ensure_ascii=False))

if __name__ == "__main__":
    main()
