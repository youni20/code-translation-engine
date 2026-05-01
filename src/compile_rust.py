import subprocess
import tempfile
from pathlib import Path


def compile_rust(rust_code: str, timeout: int = 30) -> tuple[bool, str]:
    """
    Compile Rust source code using rustc and return success status with stderr.

    Args:
        rust_code: Complete Rust source as a string.
        timeout: Maximum compilation time in seconds.

    Returns:
        (success, stderr): success is True if compilation succeeded with no errors.
        stderr contains the compiler output (errors and warnings).
    """
    with tempfile.TemporaryDirectory() as tmpdir:
        src_path = Path(tmpdir) / "main.rs"
        out_path = Path(tmpdir) / "main"
        src_path.write_text(rust_code)

        try:
            result = subprocess.run(
                ["rustc", "--edition=2021", str(src_path), "-o", str(out_path)],
                capture_output=True,
                text=True,
                timeout=timeout,
            )
            success = result.returncode == 0
            return success, result.stderr
        except subprocess.TimeoutExpired:
            return False, f"Compilation timed out after {timeout}s"
        except FileNotFoundError:
            raise RuntimeError("rustc not found on PATH. Install Rust via rustup.")