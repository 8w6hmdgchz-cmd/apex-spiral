"""
APEX Integration Skill for OpenClaw
将APEX公式集成到OpenClaw的Skill系统

使用方法:
  skill run apex-status    # 查看当前APEX状态
  skill run apex-calculate # 计算APEX
  skill run apex-evolve   # 触发自进化
"""

import json
import subprocess
import urllib.request
from typing import Dict, Any

APEX_SERVICE_URL = "http://127.0.0.1:18521"

def call_apex(method: str, data: Dict = None) -> Dict[str, Any]:
    """调用APEX服务"""
    url = f"{APEX_SERVICE_URL}/apex/{method}"
    body = json.dumps(data or {}).encode() if data else None
    req = urllib.request.Request(url, data=body, headers={"Content-Type": "application/json"})
    try:
        with urllib.request.urlopen(req, timeout=5) as resp:
            return json.loads(resp.read())
    except Exception as e:
        return {"error": str(e)}

def apex_status() -> str:
    """获取APEX状态"""
    result = call_apex("status")
    if "error" in result:
        return f"❌ APEX服务未运行: {result['error']}"
    
    return f"""
╔══════════════════════════════════════════════════════════════╗
║  APEX V10 状态                                          ║
╠══════════════════════════════════════════════════════════════╣
║  ΔG = {result.get('delta_g', 0):.6f}                                           ║
║  Evolution Score = {result.get('evolution_score', 0):.6f}                           ║
╠══════════════════════════════════════════════════════════════╣
║  参数:                                                    ║
║    Θ (LLM效能)     = {result.get('theta_llm', 0):.4f}                             ║
║    K (技能掌握)     = {result.get('k_master', 0):.4f}                             ║
║    ε (自修复成本)   = {result.get('epsilon_self_repair', 0):.4f}                            ║
║    Φ (循环增益)     = {result.get('phi_cycle', 0):.4f}                             ║
║    Ψ (主机健康)     = {result.get('psi_host', 0):.4f}                             ║
╚══════════════════════════════════════════════════════════════╝
  历史记录: {result.get('history_len', 0)} 条
"""

def apex_calculate(lambda_root: float = 0.95, theta_llm: float = 0.75,
                   k_master: float = 1.2, psi_host: float = 0.95) -> str:
    """计算APEX"""
    result = call_apex("calculate", {
        "lambda_root": lambda_root,
        "theta_llm": theta_llm,
        "k_master": k_master,
        "psi_host": psi_host
    })
    
    if "error" in result:
        return f"❌ 计算失败: {result['error']}"
    
    return f"""
╔══════════════════════════════════════════════════════════════╗
║  APEX V10 计算结果                                      ║
╠══════════════════════════════════════════════════════════════╣
║  ΔG = {result.get('delta_g', 0):.6f}                                           ║
║  Evolution Score = {result.get('evolution_score', 0):.6f}                           ║
╚══════════════════════════════════════════════════════════════╝
"""

def apex_evolve() -> str:
    """触发自进化"""
    result = call_apex("evolve", {})
    
    if "error" in result:
        return f"❌ 进化失败: {result['error']}"
    
    before = result.get("before", {})
    after = result.get("after", {})
    improvement = result.get("improvement", 0)
    
    return f"""
╔══════════════════════════════════════════════════════════════╗
║  APEX V10 自进化                                         ║
╠══════════════════════════════════════════════════════════════╣
║  进化前: ΔG = {before.get('delta_g', 0):.6f}                                     ║
║  进化后: ΔG = {after.get('delta_g', 0):.6f}                                     ║
║  提升:   ΔG = {improvement:+.6f}                                     ║
╚══════════════════════════════════════════════════════════════╝
"""

def apex_event(tokens_used: int = 0, error_count: int = 0, 
               cycle_count: int = 0, task_type: str = "") -> str:
    """上报Agent事件"""
    result = call_apex("event", {
        "tokens_used": tokens_used,
        "error_count": error_count,
        "cycle_count": cycle_count,
        "task_type": task_type
    })
    
    if "error" in result:
        return f"❌ 事件上报失败: {result['error']}"
    
    return f"✅ 事件已记录: ΔG = {result.get('delta_g', 0):.6f}"

# Skill入口点
if __name__ == "__main__":
    import sys
    action = sys.argv[1] if len(sys.argv) > 1 else "status"
    
    if action == "status":
        print(apex_status())
    elif action == "calculate":
        print(apex_calculate())
    elif action == "evolve":
        print(apex_evolve())
    elif action == "event":
        print(apex_event(0, 0, 0, ""))
    else:
        print(f"未知操作: {action}")
