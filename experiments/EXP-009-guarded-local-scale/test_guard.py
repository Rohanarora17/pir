import json
import os
from pathlib import Path
import signal
import sys
import tempfile
import unittest
from guarded_run import admission, parse_telemetry, stop_reason, watch_child, GIB

NORMAL = {"pressure_flag": 1, "swap_used_bytes": 7 * GIB, "free_percent_indicator": 45}


class GuardTests(unittest.TestCase):
    def test_parse_and_fail_on_unknown_telemetry(self):
        sample = parse_telemetry("kern.memorystatus_vm_pressure_level: 2\nvm.swapusage: used = 7246.75M", "System-wide memory free percentage: 38%")
        self.assertEqual(sample["pressure_flag"], 2)
        self.assertEqual(sample["swap_used_bytes"], int(7246.75 * (1 << 20)))
        with self.assertRaises(ValueError):
            parse_telemetry("missing", "missing")

    def test_admission_requires_normal_pressure_and_headroom(self):
        self.assertIsNone(admission(NORMAL, 24 * GIB, int(2.42 * GIB))[0])
        self.assertEqual(admission(dict(NORMAL, pressure_flag=2), 24 * GIB, GIB)[0], "memory_pressure_not_normal")
        self.assertEqual(admission(dict(NORMAL, free_percent_indicator=29), 24 * GIB, GIB)[0], "insufficient_start_headroom")
        self.assertEqual(admission(NORMAL, 24 * GIB, 8 * GIB)[0], "estimate_above_safe_budget")
        self.assertEqual(admission(dict(NORMAL, free_percent_indicator=30), 24 * GIB, 2 * GIB)[0], "estimate_above_safe_budget")

    def test_each_runtime_limit_stops_the_worker(self):
        self.assertEqual(stop_reason(dict(NORMAL, pressure_flag=4), NORMAL, 0, 0), "memory_pressure_not_normal")
        self.assertEqual(stop_reason(dict(NORMAL, free_percent_indicator=24), NORMAL, 0, 0), "free_indicator_below_reserve")
        self.assertEqual(stop_reason(dict(NORMAL, swap_used_bytes=8 * GIB), NORMAL, 0, 0), "swap_growth")
        self.assertEqual(stop_reason(NORMAL, NORMAL, 3 * GIB, 0), "worker_rss_limit")
        self.assertEqual(stop_reason(NORMAL, NORMAL, 0, 240), "timeout")

    def test_real_sleeping_child_is_killed_and_reaped_on_simulated_pressure(self):
        with tempfile.TemporaryDirectory() as tmp:
            result = watch_child([sys.executable, "-c", "import time; time.sleep(10)"], tmp,
                                 NORMAL, sample_fn=lambda: dict(NORMAL, pressure_flag=2),
                                 rss_fn=lambda pid: 0, poll=0.01)
            self.assertEqual(result["status"], "aborted")
            self.assertEqual(result["exit_code"], -signal.SIGKILL)
            self.assertLess(result["wall_seconds"], 3)

    def test_unavailable_rss_for_live_child_stops_instead_of_assuming_exit(self):
        with tempfile.TemporaryDirectory() as tmp:
            result = watch_child([sys.executable, "-c", "import time; time.sleep(10)"], tmp,
                                 NORMAL, sample_fn=lambda: NORMAL,
                                 rss_fn=lambda pid: None, poll=0.01)
            self.assertEqual(result["reason"], "telemetry_unavailable")
            self.assertEqual(result["exit_code"], -signal.SIGKILL)

    def test_healthy_tiny_child_completes_with_priority_lowering(self):
        with tempfile.TemporaryDirectory() as tmp:
            result = watch_child([sys.executable, "-c", "print('guard-ok')"], tmp,
                                 NORMAL, sample_fn=lambda: NORMAL,
                                 rss_fn=lambda pid: 0, poll=0.01)
            self.assertEqual(result["status"], "completed")
            self.assertEqual(result["exit_code"], 0)
            self.assertEqual((Path(tmp) / "worker.json").read_text().strip(), "guard-ok")

    def test_telemetry_failure_kills_child_instead_of_continuing_unmonitored(self):
        def unavailable():
            raise RuntimeError("mock telemetry unavailable")
        with tempfile.TemporaryDirectory() as tmp:
            result = watch_child([sys.executable, "-c", "import time; time.sleep(10)"], tmp,
                                 NORMAL, sample_fn=unavailable, rss_fn=lambda pid: 0, poll=0.01)
            self.assertEqual(result["reason"], "telemetry_unavailable")
            self.assertEqual(result["exit_code"], -signal.SIGKILL)


if __name__ == "__main__":
    unittest.main()
