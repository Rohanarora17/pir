"""Regression for exact retained-source inventory (no extra Cargo build inputs)."""
import hashlib
import io
import pathlib
import tarfile
import tempfile
import unittest
import prepare_source

class SourceIntegrity(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = pathlib.Path(self.temp.name)
        (self.root / 'source').mkdir()
        archive = self.root / 'source/test.tar.gz'
        with tarfile.open(archive, 'w:gz') as tar:
            info = tarfile.TarInfo('fixture/Cargo.toml')
            data = b'[package]\nname="fixture"\n'
            info.size = len(data)
            tar.addfile(info, io.BytesIO(data))
        self.sources = [('test.tar.gz', 'fixture', hashlib.sha256(archive.read_bytes()).hexdigest())]
        prepare_source.materialize(self.root, self.sources)
        self.tree = self.root / 'upstream/fixture'

    def test_clean_tree_is_reusable(self):
        prepare_source.materialize(self.root, self.sources)

    def test_extra_build_script_is_rejected(self):
        (self.tree / 'build.rs').write_text('fn main() {}')
        with self.assertRaises(SystemExit): prepare_source.materialize(self.root, self.sources)

    def test_modified_source_is_rejected(self):
        (self.tree / 'Cargo.toml').write_text('changed')
        with self.assertRaises(SystemExit): prepare_source.materialize(self.root, self.sources)

    def test_symlink_with_matching_bytes_is_rejected(self):
        data = (self.tree / 'Cargo.toml').read_bytes()
        outside = self.root / 'matching.toml'
        outside.write_bytes(data)
        (self.tree / 'Cargo.toml').unlink()
        (self.tree / 'Cargo.toml').symlink_to(outside)
        with self.assertRaises(SystemExit): prepare_source.materialize(self.root, self.sources)

if __name__ == '__main__': unittest.main()
