from lsp_tool import get_lsp_diagnostics
import time

valid_rust = """
fn main() {
    println!("hello");
}
"""

start = time.monotonic()
result = get_lsp_diagnostics(valid_rust)
elapsed = time.monotonic() - start
print(f"Elapsed: {elapsed:.1f}s")
print(f"Result: {result!r}")