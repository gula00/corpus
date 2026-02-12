# Writing an OS in Rust - Notes

## 1. Freestanding Binary

### Key Attributes

- `#![no_std]` - No Rust standard library (no OS abstractions available)
- `#![no_main]` - No default entry point (`main`), define `_start` manually

### Entry Point: `_start`

```rust
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! { loop {} }
```

- `#[unsafe(no_mangle)]` - Preserves the symbol name as `_start` in the binary; without it Rust mangles the name (e.g. `_ZN7blog_os6_start17h...`) and the linker can't find the entry point
- `extern "C"` - Uses the C calling convention so the linker recognizes it
- `-> !` - Diverging function (never returns), required because there's nothing to return to

Verify with: `nm target/debug/blog_os | grep start` → `T __start`

### Panic Handling

- Default strategy: **unwinding** - walks the call stack frame by frame, calling destructors (`drop`) for cleanup
- Unwinding requires OS-level support (`eh_personality` language item, libunwind, etc.)
- Bare-metal has no OS, so set `panic = "abort"` in `Cargo.toml` to terminate immediately instead

### Cargo.toml: `test = false` / `bench = false`

```toml
[[bin]]
name = "blog_os"
test = false
bench = false
```

- Default is `true`, which makes Cargo build a **test harness** that depends on `std`
- `std` provides its own `panic_impl` → conflicts with your `#[panic_handler]` (duplicate `lang item` error)
- `--all-targets` or `cargo test` will trigger this if not disabled
- Later the tutorial replaces this with `#![feature(custom_test_frameworks)]`

### macOS-Specific Linker Config

`.cargo/config.toml`:

```toml
[target.'cfg(target_os = "macos")']
rustflags = ["-C", "link-args=-e __start -static -nostartfiles"]
```

- `-e __start` - macOS prefixes all C symbols with `_`, so `_start` becomes `__start` in the symbol table
- `-static` - Prevent linking against `libSystem.dylib` (macOS does not officially support static binaries, but we need this to avoid OS dependencies)
- `-nostartfiles` - Skip `crt0` (C runtime startup); without this the linker expects a `main` function which we don't have (`#![no_main]`)

Linux only needs `-nostartfiles`.

Equivalent one-off command: `cargo rustc -- -C link-args="-e __start -static -nostartfiles"`

### What `-nostartfiles` Actually Does

macOS **has** a C runtime (`crt0`). The flag doesn't mean it's missing — it tells the linker **not to use it**.

```
Without -nostartfiles:  OS loads binary → crt0 init → looks for main() → error (no main)
With    -nostartfiles:  OS loads binary → jumps directly to _start → runs your code
```

### Current Stage vs. Bare-Metal

At this stage the binary is still a **macOS executable** (Mach-O format), loaded and run by the macOS kernel. The purpose is to practice removing `std` and `crt0` dependencies. True bare-metal comes in the next step.

## 2. Minimal Rust Kernel (next step)

- Target switches from host OS to bare-metal `x86_64-unknown-none`
- Uses `bootloader` crate + `bootimage` tool to produce a bootable image
- Requires QEMU: `brew install qemu && cargo install bootimage`
- Run: `qemu-system-x86_64 -drive format=raw,file=target/.../bootimage-blog_os.bin`
- At this point the binary has **nothing to do with macOS** — it runs on emulated hardware
