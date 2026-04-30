# CI check

import os

SRC_DIR = os.path.abspath(
    os.path.join(os.path.dirname(os.path.realpath(__file__)), "..", "src")
)
LICENSE_STR = "// SPDX-License-Identifier: AGPL-3.0-or-later\n\n"
LICENSE_LEN = len(LICENSE_STR)

if __name__ == "__main__":
    exit_code = 0
    for root, _, files in os.walk(SRC_DIR):
        for file in files:
            if file.endswith(".rs"):
                file_path = os.path.join(root, file)
                with open(file_path, "r") as f:
                    starts_with = f.read(LICENSE_LEN)
                    if starts_with != LICENSE_STR:
                        print(
                            f"Missing SPDX-License-Identifier in {file_path}\n(note: needs empty line)"
                        )
                        exit_code = 1
    exit(exit_code)
