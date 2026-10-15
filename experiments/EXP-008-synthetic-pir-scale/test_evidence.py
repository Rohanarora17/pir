import json
from pathlib import Path
import shutil
import tempfile
import unittest
from check_evidence import check

ROOT = Path(__file__).resolve().parent


class EvidenceTests(unittest.TestCase):
    def test_missing_cases_are_not_silently_dropped(self):
        with tempfile.TemporaryDirectory() as tmp:
            target = Path(tmp) / "run"
            shutil.copytree(ROOT / "raw/run", target)
            path = target / "processes.json"
            path.write_text(json.dumps(json.loads(path.read_text())[1:]))
            with self.assertRaisesRegex(ValueError, "missing, duplicated"):
                check(target)

    def test_reduced_security_is_rejected(self):
        with tempfile.TemporaryDirectory() as tmp:
            target = Path(tmp) / "run"
            shutil.copytree(ROOT / "raw/run", target)
            path = target / "simple-index-32-3072.json"
            item = json.loads(path.read_text())
            item["security"]["lwe_dimension"] = 32
            path.write_text(json.dumps(item))
            with self.assertRaisesRegex(ValueError, "cryptographic settings changed"):
                check(target)

    def test_edited_derived_results_do_not_pass_as_raw_evidence(self):
        with tempfile.TemporaryDirectory() as tmp:
            target = Path(tmp) / "run"
            shutil.copytree(ROOT / "raw/run", target)
            path = target / "summary.json"
            item = json.loads(path.read_text())
            item["cases"][0]["setup_payload_min_bytes"] = 0
            path.write_text(json.dumps(item))
            with self.assertRaisesRegex(ValueError, "derived summary differs"):
                check(target)


if __name__ == "__main__":
    unittest.main()
