from src.compile_rust import compile_rust

# Test 1: Valid code compiles
ok, err = compile_rust('fn main() { println!("hi"); }')
print(f"Test 1 (valid): success={ok}")
assert ok, f"Expected success, got: {err}"

# Test 2: Type error fails with informative stderr
ok, err = compile_rust('fn main() { let x: i32 = "hello"; }')
print(f"Test 2 (type error): success={ok}")
print(f"  stderr snippet: {err[:200]}")
assert not ok
assert "mismatched types" in err

# Test 3: Syntax error fails
ok, err = compile_rust("fn main( { }")
print(f"Test 3 (syntax error): success={ok}")
assert not ok

print("\nAll three checks passed.")