import subprocess
import json
import os

def get_lsp_diagnostics(project_dir: str) -> list[dict]:
    result = subprocess.run(
        ["cargo", "check", "--message-format", "json"],
        cwd=os.path.abspath(project_dir),
        capture_output=True,
        text=True
    )
    diagnostics = []
    for line in result.stdout.splitlines():
        try:
            obj = json.loads(line)
            # We only want compiler-message entries with level "error" or "warning"
            if obj.get("reason") != "compiler-message":
                continue
            message = obj.get("message", {})
            level = message.get("level", "")
            if level not in ("error", "warning"):
                continue
            diagnostics.append(message)
        except json.JSONDecodeError:
            continue
    return diagnostics

def format_diagnostics_for_prompt(diagnostics: list[dict]) -> str:
    if not diagnostics:
        return "No diagnostics returned."
    lines = []
    for d in diagnostics:
        level = d.get("level", "error")
        message = d.get("message", "")
        code = d.get("code") or {}
        code_str = code.get("code", "unknown") if isinstance(code, dict) else "unknown"
        spans = d.get("spans", [])
        primary_spans = [s for s in spans if s.get("is_primary")]
        if primary_spans:
            span = primary_spans[0]
            file_name = span.get("file_name", "?")
            line_start = span.get("line_start", "?")
            col_start = span.get("column_start", "?")
            label = span.get("label", "")
            location = f"{file_name}:{line_start}:{col_start}"
            label_str = f" ({label})" if label else ""
            lines.append(f"[{level}] {code_str} at {location}{label_str}: {message}")
        else:
            lines.append(f"[{level}] {code_str}: {message}")
    return "\n".join(lines)
