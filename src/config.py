import pathlib

PROJECT_DIR = str(pathlib.Path("./outputs/rust_project").resolve())
RELATIVE_FILE = "src/output.rs"
OUTPUT_PATH = f"{PROJECT_DIR}/{RELATIVE_FILE}"