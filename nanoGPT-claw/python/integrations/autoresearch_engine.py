
"""
NanoGPT-Claw AutoResearch Engine
Inspired by Andrej Karpathy's autoresearch
"""

import os
import subprocess
import sys
import time
import json
import shlex
from datetime import datetime
from typing import Optional, Dict, Any, List
from pathlib import Path

from core.logging import setup_logging

logger = setup_logging()


class AutoResearchEngine:
    """
    The core Autoresearch engine for autonomous iterations
    """

    def __init__(
        self,
        project_root: str = "/workspace/nanoGPT-claw",
        max_iterations: int = 25,
        iteration_timeout: int = 300  # 5 minutes
    ):
        self.project_root = Path(project_root)
        self.max_iterations = max_iterations
        self.iteration_timeout = iteration_timeout
        self.results_log = []
        self.baseline_metrics = None
        self.current_iteration = 0

        # Load program.md
        self._load_program()

    def _load_program(self):
        """Load configuration from program.md"""
        program_path = self.project_root / "program.md"
        if program_path.exists():
            logger.info(f"Loaded program from {program_path}")
        else:
            logger.warning("program.md not found, using defaults")

    def _run_command(self, cmd: str, timeout: Optional[int] = None) -> tuple[str, str, int]:
        """Run a command and return (stdout, stderr, returncode)"""
        try:
            result = subprocess.run(
                shlex.split(cmd),
                cwd=self.project_root,
                capture_output=True,
                text=True,
                timeout=timeout
            )
            return result.stdout, result.stderr, result.returncode
        except subprocess.TimeoutExpired:
            return "", f"Command timed out after {timeout}s", -1
        except Exception as e:
            return "", str(e), -2

    def _git_commit(self, message: str) -&gt; bool:
        """Create git commit"""
        _, _, code = self._run_command("git add .")
        if code != 0:
            return False
        
        _, _, code = self._run_command(f'git commit -m "{message}"')
        return code == 0

    def _git_revert(self) -&gt; bool:
        """Revert last commit"""
        _, _, code = self._run_command("git reset HEAD~1 --hard")
        return code == 0

    def _measure_metrics(self) -&gt; Dict[str, Any]:
        """Measure current state metrics"""
        metrics = {
            "timestamp": datetime.now().isoformat(),
            "compile": False,
            "test_pass": 0,
            "warnings": 0,
            "clippy": 0
        }

        # Check compilation
        logger.info("Checking compilation...")
        _, _, compile_code = self._run_command("cargo check --quiet")
        metrics["compile"] = (compile_code == 0)

        # Count warnings
        logger.info("Counting warnings...")
        stdout, stderr, _ = self._run_command("cargo check --message-format=json 2&gt;&amp;1 | grep -E '\"level\":\"warning\"' | wc -l")
        try:
            metrics["warnings"] = int(stdout.strip())
        except:
            pass

        # Run tests
        if metrics["compile"]:
            logger.info("Running tests...")
            stdout, _, _ = self._run_command("cargo test -- --list 2&gt;&amp;1 | grep ': test$' | wc -l")
            try:
                metrics["test_pass"] = int(stdout.strip())
            except:
                pass

            # Clippy check
            logger.info("Running clippy...")
            stdout, _, _ = self._run_command("cargo clippy -- -W warnings -W clippy::nursery -W clippy::pedantic 2&gt;&amp;1 | grep -c 'warning:'")
            try:
                metrics["clippy"] = int(stdout.strip())
            except:
                pass

        return metrics

    def _is_improvement(self, new_metrics: Dict[str, Any], baseline: Dict[str, Any]) -&gt; bool:
        """Determine if new state is an improvement"""
        # Compile must pass
        if not new_metrics["compile"]:
            return False

        # Warnings must be lower or equal
        if new_metrics["warnings"] &gt; baseline["warnings"]:
            return False

        # Tests must be same or higher
        if new_metrics["test_pass"] &lt; baseline["test_pass"]:
            return False

        # Check for any improvement
        if new_metrics["warnings"] &lt; baseline["warnings"]:
            return True
        if new_metrics["test_pass"] &gt; baseline["test_pass"]:
            return True
        if new_metrics["clippy"] &lt; baseline["clippy"]:
            return True

        # No regression is acceptable, but not counted as improvement
        return False

    def run_iteration(self, iteration: int) -&gt; Dict[str, Any]:
        """Run one iteration of the loop"""
        logger.info(f"=== Starting iteration {iteration}/{self.max_iterations} ===")
        
        start_time = time.time()
        
        # 1. Propose a change (placeholder - this is where the AI would make changes)
        result = {
            "iteration": iteration,
            "timestamp": datetime.now().isoformat(),
            "change_made": False,
            "change_description": "No change (baseline)",
            "metrics": self._measure_metrics(),
            "kept": False,
            "duration": 0
        }

        # 2. Measure baseline
        baseline = result["metrics"].copy()
        
        # 3. Make commit before changes
        if iteration &gt; 0:
            commit_msg = f"experiment: iteration {iteration} - {result['change_description']}"
            self._git_commit(commit_msg)

        # 4. Verify and keep/revert
        new_metrics = self._measure_metrics()
        result["metrics"] = new_metrics
        
        if self._is_improvement(new_metrics, baseline):
            logger.info(f"✓ Iteration {iteration}: IMPROVEMENT - keeping!")
            result["kept"] = True
        elif iteration &gt; 0:
            logger.info(f"✗ Iteration {iteration}: no improvement - reverting")
            self._git_revert()
            result["kept"] = False

        result["duration"] = time.time() - start_time
        
        self.results_log.append(result)
        self._save_results()
        
        return result

    def _save_results(self):
        """Save results to JSON"""
        results_file = self.project_root / "autoresearch_results.json"
        with open(results_file, "w", encoding="utf-8") as f:
            json.dump(self.results_log, f, indent=2, ensure_ascii=False)
        logger.info(f"Results saved to {results_file}")

    def run_full_loop(self):
        """Run the full autoresearch loop"""
        logger.info("🚀 Starting NanoGPT-Claw AutoResearch Loop")
        
        # Establish baseline
        logger.info("Establishing baseline...")
        self.baseline_metrics = self._measure_metrics()
        logger.info(f"Baseline: {json.dumps(self.baseline_metrics, indent=2)}")
        
        # Run iterations
        for i in range(1, self.max_iterations + 1):
            self.current_iteration = i
            try:
                result = self.run_iteration(i)
                
                if result["kept"]:
                    logger.info(f"✅ Kept iteration {i}")
                else:
                    logger.info(f"✗ Reverted iteration {i}")
                
            except Exception as e:
                logger.error(f"Error in iteration {i}: {e}")
                # Attempt to recover
                self._git_revert()
        
        # Final report
        logger.info("\n" + "="*50)
        logger.info("Loop Complete! Summary:")
        logger.info(f"Total iterations: {self.current_iteration}")
        
        kept = sum(1 for r in self.results_log if r.get("kept"))
        logger.info(f"Improvements kept: {kept}/{len(self.results_log)}")
        
        logger.info(f"Final metrics: {json.dumps(self.results_log[-1]['metrics'], indent=2)}")
        logger.info("="*50)


def main():
    """Entry point"""
    engine = AutoResearchEngine(
        max_iterations=10  # Start with 10 for testing
    )
    engine.run_full_loop()


if __name__ == "__main__":
    main()

