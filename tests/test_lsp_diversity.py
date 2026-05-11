"""Verify LSP diagnostics work across diverse Rust error categories."""
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent.parent / "src"))

from lsp_tool import get_lsp_diagnostics
import time

snippets = {
    "valid": "fn main() { println!(\"hello\"); }",
    "type_mismatch": "fn main() { let x: i32 = \"hello\"; }",
    "borrow_check": "fn main() { let mut v = vec![1]; let r = &v; v.push(2); println!(\"{:?}\", r); }",
    "missing_import": "fn main() { let m = HashMap::new(); m.insert(1, 2); }",
    "lifetime": "fn longest<'a>(x: &'a str, y: &str) -> &'a str { y }",
    "trait_unsatisfied": "fn print_it<T: std::fmt::Display>(x: T) { println!(\"{}\", x); } fn main() { print_it(vec![1, 2, 3]); }",
    "undefined_function": "fn main() { undefined_function(); }",
    "syntax_error": "fn main() { let x = ; }",
}

for name, code in snippets.items():
    print(f"\n=== {name} ===")
    start = time.monotonic()
    result = get_lsp_diagnostics(code)
    elapsed = time.monotonic() - start
    print(f"({elapsed:.1f}s)")
    print(result)