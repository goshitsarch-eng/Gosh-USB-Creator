"""Pipe locked cargo metadata into this script after changing Rust dependencies."""
import json
import sys
from collections import defaultdict
from pathlib import Path

groups = defaultdict(list)
for package in json.load(sys.stdin)["packages"]:
    if package["name"] == "gosh-core":
        continue
    root = Path(package["manifest_path"]).parent
    files = sorted(
        path for path in root.iterdir()
        if path.is_file() and path.name.upper().startswith(("LICENSE", "COPYRIGHT"))
    )
    text = "\n\n".join(path.read_text(errors="replace") for path in files)
    if not text:
        text = "License: " + str(package.get("license")) + "\nUpstream: " + str(
            package.get("repository") or "https://crates.io/crates/" + package["name"]
        )
    groups[text.strip()].append(package["name"] + " " + package["version"])

output = Path(__file__).resolve().parents[1] / "assets/rust_licenses.json"
output.parent.mkdir(exist_ok=True)
output.write_text(json.dumps([
    {"packages": sorted(packages), "text": text}
    for text, packages in groups.items()
], ensure_ascii=False, indent=2) + "\n")
