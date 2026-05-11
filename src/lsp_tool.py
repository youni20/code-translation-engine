import asyncio
import pathlib
from collections.abc import Sequence
from typing import Any
from multilspy import SyncLanguageServer
from multilspy.multilspy_config import MultilspyConfig, Language
from multilspy.multilspy_logger import MultilspyLogger
from config import LSP_PROJECT_DIR, LSP_RELATIVE_FILE, LSP_OUTPUT_PATH


def get_lsp_diagnostics(rust_code: str) -> str:
    """
    Write rust_code to disk and pull structured diagnostics from rust-analyzer.
    Returns a formatted diagnostic srting to use as repair feedback.
    Args:
        rust_code (str): The complete Rust source code that failed to compile.
    Returns:
        str: Structured LSP diagnostics with error codes, line/column numbers,
             severity levels, and messages.
    """
    output_path = pathlib.Path(LSP_OUTPUT_PATH)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(rust_code, encoding="utf-8")
    config = MultilspyConfig(code_language=Language.RUST)
    logger = MultilspyLogger()
    lsp = SyncLanguageServer.create(config, logger, LSP_PROJECT_DIR)
    diagnostics: Sequence[Any] = []
    with lsp.start_server():
        with lsp.open_file(LSP_RELATIVE_FILE):
            loop = asyncio.new_event_loop()
            try:
                result = loop.run_until_complete(
                    lsp.language_server.server.send.text_document_diagnostic({
                        "textDocument": {
                            "uri": pathlib.Path(LSP_OUTPUT_PATH).as_uri()
                        }
                    })
                )
                diagnostics = result.get("items", [])
            finally:
                loop.close()
    if not diagnostics:
        return "No diagnostics returned by rust-analyzer."
    return _format_diagnostics(diagnostics)


def _format_diagnostics(diagnostics: Sequence[Any]) -> str:
    severity_map = {1: "error", 2: "warning", 3: "information", 4: "hint"}
    lines = []
    for d in diagnostics:
        severity = severity_map.get(d.get("severity", 1), "error")
        message = d.get("message", "")
        code = d.get("code", "unknown")
        rng = d.get("range", {})
        start = rng.get("start", {})
        line = start.get("line", "?")
        col = start.get("character", "?")
        lines.append(
            f"[{severity}] {code} at line {line}, col {col}: {message}"
        )
    return "\n".join(lines)