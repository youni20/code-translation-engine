"""LSP diagnostics tool — synchronous push-mode client over stdio.

Replaces the prior multilspy-based pull-mode implementation, which hung
indefinitely because the sync wrapper did not flush rust-analyzer's
textDocument/diagnostic response. The new implementation spawns
rust-analyzer directly, performs the initialize/initialized/didOpen
handshake, and collects textDocument/publishDiagnostics notifications
with a 3 s settle window and a 30 s hard ceiling on the whole call.
"""

import json
import os
import pathlib
import queue
import subprocess
import threading
import time
from collections.abc import Sequence
from typing import Any

import lsprotocol.converters
import lsprotocol.types as lsp_types

from config import LSP_PROJECT_DIR, LSP_RELATIVE_FILE, LSP_OUTPUT_PATH


# region: multilspy reference implementation (deprecated 2026-05-11)
# import asyncio
# from multilspy import SyncLanguageServer
# from multilspy.multilspy_config import MultilspyConfig, Language
# from multilspy.multilspy_logger import MultilspyLogger
#
# def get_lsp_diagnostics(rust_code: str) -> str:
#     output_path = pathlib.Path(LSP_OUTPUT_PATH)
#     output_path.parent.mkdir(parents=True, exist_ok=True)
#     output_path.write_text(rust_code, encoding="utf-8")
#     config = MultilspyConfig(code_language=Language.RUST)
#     logger = MultilspyLogger()
#     lsp = SyncLanguageServer.create(config, logger, LSP_PROJECT_DIR)
#     diagnostics: Sequence[Any] = []
#     with lsp.start_server():
#         with lsp.open_file(LSP_RELATIVE_FILE):
#             loop = asyncio.new_event_loop()
#             try:
#                 result = loop.run_until_complete(
#                     lsp.language_server.server.send.text_document_diagnostic({
#                         "textDocument": {
#                             "uri": pathlib.Path(LSP_OUTPUT_PATH).as_uri()
#                         }
#                     })
#                 )
#                 diagnostics = result.get("items", [])
#             finally:
#                 loop.close()
#     if not diagnostics:
#         return "No diagnostics returned by rust-analyzer."
#     return _format_diagnostics(diagnostics)
# endregion


_HANDSHAKE_BUDGET_SECONDS = 30.0
_SETTLE_WINDOW_SECONDS = 3.0


def get_lsp_diagnostics(rust_code: str) -> str:
    """Write rust_code to disk and pull structured diagnostics from rust-analyzer.

    Spawns rust-analyzer over stdio, performs the LSP handshake
    (initialize → initialized → textDocument/didOpen), and listens for
    textDocument/publishDiagnostics notifications (push-mode). The whole
    call is bounded by a 30 s hard ceiling. Once the first publish for our
    URI arrives, an additional 3 s settle window lets us capture flycheck
    refinement batches (publishDiagnostics is a per-URI replacement, so
    each batch overwrites the previous one — last-write-wins).

    Returns a formatted diagnostic string for use as repair feedback, or
    "No diagnostics returned by rust-analyzer." for a clean file or any
    failure to obtain diagnostics within the timeout.

    Args:
        rust_code (str): The complete Rust source code that failed to compile.
    Returns:
        str: Structured LSP diagnostics with error codes, line/column numbers,
             severity levels, and messages.
    """
    output_path = pathlib.Path(LSP_OUTPUT_PATH)
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(rust_code, encoding="utf-8")

    our_uri = output_path.as_uri()
    converter = lsprotocol.converters.get_converter()
    deadline = time.monotonic() + _HANDSHAKE_BUDGET_SECONDS

    proc = subprocess.Popen(
        ["rust-analyzer"],
        cwd=LSP_PROJECT_DIR,
        stdin=subprocess.PIPE,
        stdout=subprocess.PIPE,
        stderr=subprocess.DEVNULL,
        bufsize=0,
    )

    incoming: "queue.Queue[dict[str, Any]]" = queue.Queue()

    def _read_message() -> dict[str, Any] | None:
        headers: dict[str, str] = {}
        while True:
            line = proc.stdout.readline()
            if not line:
                return None
            if line in (b"\r\n", b"\n"):
                break
            try:
                key, _, value = line.decode("ascii").rstrip("\r\n").partition(":")
            except UnicodeDecodeError:
                continue
            headers[key.strip().lower()] = value.strip()
        length = int(headers.get("content-length", "0") or "0")
        if length <= 0:
            return None
        body = bytearray()
        while len(body) < length:
            chunk = proc.stdout.read(length - len(body))
            if not chunk:
                return None
            body.extend(chunk)
        try:
            return json.loads(body.decode("utf-8"))
        except json.JSONDecodeError:
            return None

    def _reader() -> None:
        while True:
            msg = _read_message()
            if msg is None:
                return
            incoming.put(msg)

    reader_thread = threading.Thread(target=_reader, daemon=True)
    reader_thread.start()

    def _send(message: dict[str, Any]) -> None:
        body = json.dumps(message).encode("utf-8")
        header = f"Content-Length: {len(body)}\r\n\r\n".encode("ascii")
        try:
            proc.stdin.write(header + body)
            proc.stdin.flush()
        except (BrokenPipeError, ValueError, OSError):
            pass

    diagnostics: list[dict[str, Any]] = []

    try:
        init_params = lsp_types.InitializeParams(
            process_id=os.getpid(),
            root_uri=pathlib.Path(LSP_PROJECT_DIR).as_uri(),
            capabilities=lsp_types.ClientCapabilities(),
        )
        _send({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": converter.unstructure(init_params),
        })

        # Drain incoming traffic until rust-analyzer answers the initialize
        # request. Any server-initiated requests (e.g. client/registerCapability)
        # are acknowledged with a null result so the server does not stall.
        initialized = False
        while time.monotonic() < deadline:
            try:
                msg = incoming.get(timeout=deadline - time.monotonic())
            except queue.Empty:
                break
            if msg.get("id") == 1 and "result" in msg:
                initialized = True
                break
            if "method" in msg and "id" in msg:
                _send({"jsonrpc": "2.0", "id": msg["id"], "result": None})

        if not initialized:
            return "No diagnostics returned by rust-analyzer."

        _send({
            "jsonrpc": "2.0",
            "method": "initialized",
            "params": converter.unstructure(lsp_types.InitializedParams()),
        })

        did_open_params = lsp_types.DidOpenTextDocumentParams(
            text_document=lsp_types.TextDocumentItem(
                uri=our_uri,
                language_id="rust",
                version=1,
                text=rust_code,
            )
        )
        _send({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": converter.unstructure(did_open_params),
        })

        first_publish_seen = False
        first_publish_time: float | None = None
        while True:
            now = time.monotonic()
            if now >= deadline:
                break
            if first_publish_seen and first_publish_time is not None:
                settle_deadline = first_publish_time + _SETTLE_WINDOW_SECONDS
                if now >= settle_deadline:
                    break
                wait_timeout = min(deadline - now, settle_deadline - now)
            else:
                wait_timeout = deadline - now

            try:
                msg = incoming.get(timeout=wait_timeout)
            except queue.Empty:
                # Outer deadline or settle window elapsed; outer-loop check decides.
                continue

            if "method" in msg and "id" in msg:
                _send({"jsonrpc": "2.0", "id": msg["id"], "result": None})
                continue

            if msg.get("method") != "textDocument/publishDiagnostics":
                continue

            params = msg.get("params") or {}
            if params.get("uri") != our_uri:
                continue

            diagnostics = params.get("diagnostics", []) or []

            if not first_publish_seen:
                first_publish_seen = True
                first_publish_time = time.monotonic()

        if not first_publish_seen or not diagnostics:
            return "No diagnostics returned by rust-analyzer."
        return _format_diagnostics(diagnostics)

    finally:
        _send({"jsonrpc": "2.0", "id": 9999, "method": "shutdown", "params": None})
        _send({"jsonrpc": "2.0", "method": "exit", "params": None})
        try:
            proc.stdin.close()
        except (BrokenPipeError, OSError):
            pass
        try:
            proc.wait(timeout=2)
        except subprocess.TimeoutExpired:
            proc.terminate()
            try:
                proc.wait(timeout=2)
            except subprocess.TimeoutExpired:
                proc.kill()
        reader_thread.join(timeout=1)


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
