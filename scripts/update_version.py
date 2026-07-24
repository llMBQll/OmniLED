import argparse
import os
import re


def update_file(path, new_version):
    VERSION_LINE_RE = re.compile(r"^version = \"\d+\.\d+\.\d+\"$")

    with open(path, "r", encoding="utf-8") as f:
        lines = f.readlines()

    for index, line in enumerate(lines):
        if VERSION_LINE_RE.match(line):
            lines[index] = f'version = "{new_version}"\n'

            with open(path, "w", encoding="utf-8") as f:
                print(f"Updating {path}")
                f.writelines(lines)

            break


parser = argparse.ArgumentParser(description="Update release versions")
parser.add_argument("version", help="New release version")
args = parser.parse_args()

for root, _, files in os.walk("."):
    for file in files:
        if file in {"Cargo.toml", "Packager.toml"}:
            path = os.path.join(root, file)
            update_file(path, args.version)
