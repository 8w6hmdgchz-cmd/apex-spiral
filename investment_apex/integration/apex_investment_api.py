#!/usr/bin/env python3
"""
APEX Investment Master API
==========================
投资大师 + APEX 自我进化系统
Port: 18524
"""

from flask import Flask, request, jsonify
from apex_investment import (
    get_investment_engine,
    INVESTMENT_MASTERS,
    ANALYSIS_AGENTS
)
import json

app = Flask(__name__)
engine = get_investment_engine()


@app.route("/health")
def health():
    return jsonify({"status": "ok", "service": "apex-investment"})


@app.route("/investment/masters")
def masters():
    """获取13位投资大师列表"""
    return jsonify({
        "masters": INVESTMENT_MASTERS,
        "count": len(INVESTMENT_MASTERS)
    })


@app.route("/investment/analysis-agents")
def analysis():
    """获取分析Agent列表"""
    return jsonify({
        "agents": ANALYSIS_AGENTS,
        "count": len(ANALYSIS_AGENTS)
    })


@app.route("/investment/signal", methods=["POST"])
def add_signal():
    """添加大师信号"""
    data = request.get_json() or {}
    
    required = ["master", "signal", "confidence"]
    if not all(k in data for k in required):
        return jsonify({"error": f"Missing fields. Required: {required}"}), 400
    
    engine.add_signal(data)
    
    return jsonify({
        "status": "added",
        "signal": data,
        "master_weights": engine.get_master_weights()
    })


@app.route("/investment/consensus", methods=["GET"])
def consensus():
    """获取大师共识"""
    return jsonify(engine.generate_master_consensus(engine.signal_history))


@app.route("/investment/evolve", methods=["POST"])
def evolve():
    """触发 APEX 进化"""
    result = engine.evolve()
    return jsonify(result)


@app.route("/investment/state")
def state():
    """获取当前 APEX 状态"""
    return jsonify({
        "delta_g": engine.state.delta_g,
        "theta_llm": engine.state.theta_llm,
        "phi_cycle": engine.state.phi_cycle,
        "signal_count": len(engine.signal_history),
        "master_weights": engine.get_master_weights()
    })


@app.route("/investment/backtest", methods=["POST"])
def backtest():
    """回测信号历史"""
    data = request.get_json() or {}
    period = data.get("period", "1mo")
    
    if len(engine.signal_history) < 5:
        return jsonify({
            "error": "Need at least 5 signals for backtest",
            "current": len(engine.signal_history)
        })
    
    # 简化回测计算
    total_return = 0
    for i, sig in enumerate(engine.signal_history):
        if sig["signal"] == "bullish":
            total_return += sig["confidence"] / 100 * 0.02  # 假设平均2%收益
        elif sig["signal"] == "bearish":
            total_return -= sig["confidence"] / 100 * 0.01  # 假设平均1%亏损
    
    return jsonify({
        "period": period,
        "signal_count": len(engine.signal_history),
        "estimated_return": round(total_return * 100, 2),
        "delta_g": engine.state.delta_g,
        "recommendation": "buy" if total_return > 0.05 else "sell" if total_return < -0.03 else "hold"
    })


if __name__ == "__main__":
    print("📈 APEX Investment Master API starting on :18524")
    print("=" * 50)
    print("13 Investment Masters + APEX Self-Evolution")
    print("=" * 50)
    app.run(host="127.0.0.1", port=18524, debug=False)
