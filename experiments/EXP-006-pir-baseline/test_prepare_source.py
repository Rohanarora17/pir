"""Retained IKPIR source must be the complete build input, with no extra files."""
import pathlib
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = pathlib.Path(__file__).resolve().parent
REVISION = "fcfb2b540c1cfa92c2de5ec818db3aa9fc817395"


class SourceIntegrity(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = pathlib.Path(self.temp.name)
        (self.root / "source").mkdir()
        shutil.copyfile(ROOT / "prepare_source.py", self.root / "prepare_source.py")
        shutil.copyfile(ROOT / "source/ikpir.tar.gz", self.root / "source/ikpir.tar.gz")
        self.assertEqual(self.prepare().returncode, 0)
        self.tree = self.root / "upstream" / f"IKPIR-{REVISION}"

    def prepare(self):
        return subprocess.run(
            [sys.executable, str(self.root / "prepare_source.py")],
            capture_output=True, text=True, check=False,
        )

    def test_clean_tree_is_reusable(self):
        self.assertEqual(self.prepare().returncode, 0)

    def test_added_cargo_build_script_is_rejected(self):
        (self.tree / "crates/ikpir-common/build.rs").write_text("fn main() {}\n")
        self.assertNotEqual(self.prepare().returncode, 0)

    def test_modified_source_is_rejected(self):
        (self.tree / "Cargo.toml").write_text("changed")
        self.assertNotEqual(self.prepare().returncode, 0)

    def test_symlink_with_matching_contents_is_rejected(self):
        original = self.tree / "Cargo.toml"
        outside = self.root / "matching.toml"
        outside.write_bytes(original.read_bytes())
        original.unlink()
        original.symlink_to(outside)
        self.assertNotEqual(self.prepare().returncode, 0)


if __name__ == "__main__":
    unittest.main()
