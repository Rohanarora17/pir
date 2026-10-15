#!/usr/bin/env python3
"""Materialize the retained, unmodified upstream archive after verifying its digest."""
import hashlib
import pathlib
import tarfile

ROOT = pathlib.Path(__file__).resolve().parent
REVISION = "fcfb2b540c1cfa92c2de5ec818db3aa9fc817395"
DIGEST = "c271a8be463a94d62467f9cae4dd17014aa2bb983bb3f6e0f0ed8877e41efe57"
archive = ROOT / "source/ikpir.tar.gz"
if hashlib.sha256(archive.read_bytes()).hexdigest() != DIGEST:
    raise SystemExit("upstream source archive checksum mismatch")
destination = ROOT / "upstream"
tree = destination / f"IKPIR-{REVISION}"
if destination.is_symlink() or tree.is_symlink():
    raise SystemExit("source root must not be a symlink")
with tarfile.open(archive, "r:gz") as source:
    for member in source.getmembers():
        parts = pathlib.PurePosixPath(member.name).parts
        if not parts or parts[0] != f"IKPIR-{REVISION}" or ".." in parts:
            raise SystemExit("unexpected archive member")
        if not (member.isfile() or member.isdir()):
            raise SystemExit("only regular files and directories are admitted")
    if not tree.exists():
        destination.mkdir(exist_ok=True)
        source.extractall(destination, filter="data")
    # Cargo can automatically execute unlisted build.rs files. Admit the exact tree only.
    expected = {pathlib.PurePosixPath(f"IKPIR-{REVISION}")}
    for member in source.getmembers():
        path = pathlib.PurePosixPath(member.name)
        expected.add(path)
        expected.update(parent for parent in path.parents if str(parent) != ".")
    actual = {pathlib.PurePosixPath(f"IKPIR-{REVISION}")}
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
        if member.isfile():
            if not path.is_file() or path.read_bytes() != source.extractfile(member).read():
                raise SystemExit(f"modified upstream file: {member.name}")
print(f"verified unmodified IKPIR source {REVISION}")
