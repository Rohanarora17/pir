#!/usr/bin/env python3
"""Verify and materialize retained upstream archives; never modify their source."""
import hashlib
import pathlib
import tarfile
ROOT = pathlib.Path(__file__).resolve().parent
SOURCES = [
    ("helios.tar.gz", "helios-2af9b41887e069144ee56cd710f3ddca195473f8", "4481af452e68dce08a810b8f90802c3f273c6eef1c63db2575ca0d1305494322"),
    ("ethereum_hashing.tar.gz", "ethereum_hashing-7ee70944ed4fabe301551da8c447e4f4ae5e6c35", "5d179c5229158ab349454c575f4ec1e628543558abcb1d97c717606f58abd93f"),
]
def materialize(root, sources):
    for filename, prefix, digest in sources:
        archive = root / "source" / filename
        if hashlib.sha256(archive.read_bytes()).hexdigest() != digest:
            raise SystemExit(f"source checksum mismatch: {filename}")
        with tarfile.open(archive, "r:gz") as source:
            for member in source.getmembers():
                parts = pathlib.PurePosixPath(member.name).parts
                if not parts or parts[0] != prefix or ".." in parts or not (member.isfile() or member.isdir()):
                    raise SystemExit(f"unexpected archive member: {member.name}")
            destination = root / "upstream"
            tree = destination / prefix
            if destination.is_symlink() or tree.is_symlink():
                raise SystemExit("source root must not be a symlink")
            if not (destination / prefix).exists():
                destination.mkdir(exist_ok=True)
                source.extractall(destination, filter="data")
            # Cargo also discovers files absent from Cargo.toml, such as build.rs.
            # Compare the complete inventory, including implicit parent directories.
            expected = {pathlib.PurePosixPath(prefix)}
            for member in source.getmembers():
                path = pathlib.PurePosixPath(member.name)
                expected.add(path)
                expected.update(parent for parent in path.parents if str(parent) != ".")
            actual = {pathlib.PurePosixPath(prefix)}
            for path in tree.rglob("*"):
                if path.is_symlink() or not (path.is_file() or path.is_dir()):
                    raise SystemExit(f"unexpected source file type: {path}")
                actual.add(pathlib.PurePosixPath(path.relative_to(destination).as_posix()))
            if actual != expected:
                raise SystemExit("extracted source inventory differs from retained archive")
            for member in source.getmembers():
                path = destination / member.name
                if member.isdir() and not path.is_dir():
                    raise SystemExit(f"source directory type changed: {member.name}")
                if member.isfile() and (not path.is_file() or path.read_bytes() != source.extractfile(member).read()):
                    raise SystemExit(f"modified upstream file: {member.name}")
        print(f"verified unmodified source: {prefix}")

if __name__ == "__main__":
    materialize(ROOT, SOURCES)
