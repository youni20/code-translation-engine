import subprocess
import json
import os

def get_lsp_diagnostics(project_dir: str) -> list[dict]:
    result = subprocess.run(
        ["rust-analyzer", "diagnostics", "--output-format", "json", "."],
        cwd=os.path.abspath(project_dir),
        capture_output=True,
        text=True
    )
    diagnostics = []
    for line in result.stdout.splitlines():
        try:
            diagnostics.append(json.loads(line))
        except json.JSONDecodeError:
            continue
    return diagnostics

def format_diagnostics_for_prompt(diagnostics: list[dict]) -> str:
    if not diagnostics:
        return "No diagnostics returned from rust-analyzer."
    lines = []
    for d in diagnostics:
        severity = d.get("severity", "error")
        code = d.get("code", {})
        code_str = code.get("code", "unknown") if isinstance(code, dict) else str(code)
        message = d.get("message", "")
        spans = d.get("spans", [])
        if spans:
            span = spans[0]
            line_num = span.get("line_start", "?")
            col = span.get("column_start", "?")
            lines.append(f"[{severity}] {code_str} at line {line_num}, col {col}: {message}")
        else:
            lines.append(f"[{severity}] {code_str}: {message}")
    return "\n".join(lines)