#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."

echo "[1/3] Generate Task_APEX dataset"
python3 py/data_pipeline/generate_tasks.py

echo "[2/3] Run Bench: AutoVerify60 + LLMVerify40"
python3 py/bench/run_bench.py

echo "[3/3] Aggregate Score_APEX"
python3 py/bench/aggregate_score.py
