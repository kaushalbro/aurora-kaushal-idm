# Contributing to AURORA Download Manager

Thank you for contributing to the AURORA project. This guide outlines development practices, workspace conventions, testing expectations, and contribution guidelines.

---

## 1. Development Principles

1. **Pure Rust**: Maintain 100% pure Rust implementations across all subsystems. Avoid C/C++ dependencies or webview wrappers.
2. **Platform Separation**: Never assume Unix-only or Windows-only behavior. All OS-specific operations must reside in separate files:
   - `crates/<crate>/src/platform/linux.rs`
   - `crates/<crate>/src/platform/windows.rs`
   - `crates/<crate>/src/platform/common.rs`
3. **Bounded Memory**: Never allocate unbounded buffers. All chunk streaming must route through bounded channels with backpressure propagation.
4. **Zero Unverified Assertions**: Never claim algorithmic superiority without reproducible benchmarks from `aurora-benchmark`.

---

## 2. Setting Up the Development Environment

Ensure you are using Rust stable (1.80+ recommended):

```bash
git clone https://github.com/aurora-idm/aurora.git
cd aurora

# Verify all crates compile cleanly
cargo check --workspace

# Run entire test suite
cargo test --workspace
```

---

## 3. Running Schedulers & Benchmarks Locally

```bash
# Run simulator
cargo run --bin aurora -- simulate --workers 100,50,20 --file-size-mb 512

# Run Mock Server on background port
cargo run --bin aurora -- test-server --size-mb 50

# Run GUI
cargo run --bin aurora-gui
```

---

## 4. Submitting Pull Requests

- Ensure `cargo test --workspace` passes with 0 failures and 0 compiler warnings.
- Format all code with `cargo fmt`.
- Add unit tests for any new scheduling heuristic, range arithmetic, or storage interaction.
