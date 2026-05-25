#!/usr/bin/env python3
from __future__ import annotations

import json
import subprocess
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / "skills" / "apex-tiangong-skill"))

from tiangong_native import TiangongTask, SandboxAdapter, EvolverLoop, CognitiveRouter, SuperpowersGate, run_native_loop


class TianGongNativeTests(unittest.TestCase):
    def test_task_serializable(self):
        task = TiangongTask("verify native loop")
        data = task.__dict__
        self.assertIn("task_id", data)
        self.assertEqual(data["risk"], "low")

    def test_high_risk_blocks_execution(self):
        task = TiangongTask("danger", risk="high")
        event = SandboxAdapter().execute(task, ["python3", "--version"])
        self.assertEqual(event.status, "blocked")

    def test_rust_sandbox_blocks_disallowed_command(self):
        task = TiangongTask("block unsafe command")
        event = SandboxAdapter().execute(task, ["bash", "-lc", "echo nope"])
        self.assertEqual(event.status, "failed")
        self.assertIn("audit", event.evidence)
        self.assertEqual(event.evidence["audit"].get("status"), "blocked")

    def test_evolver_core_promotes_verified_task(self):
        task = TiangongTask("verify evolver core")
        event = EvolverLoop().consolidate(task, ROOT / "state" / "tiangong" / "native" / "dummy.json")
        self.assertEqual(event.status, "ok")
        self.assertEqual(event.evidence["core"].get("promotion"), "pass")

    def test_cognitive_router_ranks_options(self):
        task = TiangongTask("implement rust cognitive router")
        event = CognitiveRouter().rank_options(task)
        self.assertEqual(event.status, "ok")
        options = event.evidence.get("options", [])
        self.assertTrue(options)
        self.assertGreaterEqual(options[0]["score"], options[-1]["score"])
        self.assertIn("systems_engineer", event.evidence["core"].get("roles", []))

    def test_superpowers_gate_blocks_high_risk_review(self):
        task = TiangongTask("dangerous promotion", risk="high")
        event = SuperpowersGate().review(task, [])
        self.assertEqual(event.status, "blocked")
        self.assertFalse(event.evidence["core"].get("passed"))

    def test_native_loop_promotes(self):
        report = run_native_loop("unit test native TianGong loop")
        self.assertEqual(report.promotion, "pass")
        self.assertGreaterEqual(report.scores["fitness"], 0.7)
        self.assertTrue((ROOT / "state" / "tiangong" / "native" / "latest.json").exists())


if __name__ == "__main__":
    unittest.main()
