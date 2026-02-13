# Writing an OS in Rust - Notes

## 1. Freestanding Binary

A macOS executable with no `std` and no C runtime — practice for bare-metal.

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

### What `-nostartfiles` Actually Does

macOS **has** a C runtime (`crt0`). The flag doesn't mean it's missing — it tells the linker **not to use it**.

```
Without -nostartfiles:  OS loads binary → crt0 init → looks for main() → error (no main)
With    -nostartfiles:  OS loads binary → jumps directly to _start → runs your code
```

### Current Stage vs. Bare-Metal

At this stage the binary is still a **macOS executable** (Mach-O format), loaded and run by the macOS kernel. The purpose is to practice removing `std` and `crt0` dependencies. True bare-metal comes in the next chapter.

---

## 2. Minimal Rust Kernel

From a macOS executable to a real bare-metal kernel running on emulated x86_64 hardware.

### Boot Process

```
Power on → BIOS firmware → find boot sector (512 bytes) → load bootloader
→ bootloader: 16-bit real mode → 32-bit protected mode → 64-bit long mode
→ set up page tables, parse kernel ELF, load into memory → jump to kernel entry
```

The bootloader does the heavy lifting so we can write kernel logic in Rust.

### Toolchain: `rust-toolchain.toml`

```toml
[toolchain]
channel = "nightly"
components = ["rust-src", "llvm-tools-preview"]
```

- **nightly** - `build-std` etc. are unstable features, not available on stable
- **rust-src** - Standard library source code. Our target (`x86_64-unknown-none`) has no precompiled `core`, so it must be rebuilt from source
- **llvm-tools-preview** - The bootloader needs `llvm-objcopy` etc. to create disk images

### Target: `.cargo/config.toml`

```toml
[build]
target = "x86_64-unknown-none"
rustflags = ["-C", "relocation-model=static"]

[unstable]
build-std = ["core", "compiler_builtins"]
build-std-features = ["compiler-builtins-mem"]
```

Chapter 1 targeted macOS (`aarch64-apple-darwin`). Now we switch to **`x86_64-unknown-none`** — a bare-metal target: no OS, no C runtime, no `std`, only `core`.

**`rustflags`**: disable PIE (position-independent executables). The built-in target defaults to PIE, which can cause bootloader issues.

**`build-std`**: rebuild `core` (basic types, Option, Result, etc.) and `compiler_builtins` (compiler intrinsics like integer division) from source for our bare-metal target.

**`compiler-builtins-mem`**: provide `memcpy`, `memset`, `memcmp`. Normally these come from libc, but bare-metal has no libc.

The macOS linker config from chapter 1 (`-e __start -static -nostartfiles`) is no longer needed.

#### Why `x86_64-unknown-none` instead of a custom JSON target spec

The blog creates a custom `x86_64-blog_os.json` to describe the bare-metal target. With modern nightly, the built-in `x86_64-unknown-none` is simpler and already includes everything:

- **`disable-redzone: true`** - The red zone is a System V ABI optimization: functions can use 128 bytes below the stack pointer without moving it. Interrupts push data there, corrupting it. Must be disabled in kernels
- **`-mmx,-sse,+soft-float`** - Disable SIMD, use software float. Saving/restoring 512-bit XMM registers on every interrupt is expensive, and kernels rarely need floats
- **`panic-strategy: abort`** - Abort on panic, no unwinding
- **`linker: rust-lld`** - Rust's bundled cross-platform linker

Using the built-in target avoids JSON compatibility issues across nightly versions (field types changing, data-layout format updates, etc.).

### Dependencies: `Cargo.toml`

```toml
[dependencies]
bootloader_api = "0.11"
```

`bootloader_api` provides the `entry_point!` macro and the `BootInfo` type (memory maps, framebuffer info, etc.).

### Entry Point

```rust
use bootloader_api::entry_point;

entry_point!(kernel_main);

fn kernel_main(_boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    // kernel code here
    loop {}
}
```

The `entry_point!` macro:

1. Generates the real `_start` function as the linker entry (replaces the hand-written `#[no_mangle] pub extern "C" fn _start` from chapter 1)
2. Type-checks `kernel_main` — ensures the signature is `fn(&'static mut BootInfo) -> !`

Safer than hand-writing `_start` because the macro catches signature errors at compile time.

### Three Ways to Display Output

#### 1. VGA Text Buffer (Blog's Original Approach)

Write characters directly to memory address `0xb8000`:

```rust
let vga_buffer = 0xb8000 as *mut u8;
unsafe {
    *vga_buffer.offset(0) = b'H';     // character
    *vga_buffer.offset(1) = 0xb;      // color: light cyan
}
```

- **`0xb8000`** - VGA text mode video memory. **Memory-mapped I/O (MMIO)** — writes change the screen
- Each character = 2 bytes: ASCII code + color attribute (`background(4 bits) | foreground(4 bits)`)
- `0x0b` = black background, light cyan text

```
Address    Content
0xb8000    'H'     ← character
0xb8001    0x0b    ← color
0xb8002    'e'
0xb8003    0x0b
...
```

**Does NOT work with bootloader 0.11** — it switches to a graphical framebuffer, VGA text mode is unavailable.

#### 2. Framebuffer (Pixel Drawing)

Bootloader 0.11 provides a graphical framebuffer (1280x720, BGR) through `BootInfo`:

```rust
if let Some(fb) = boot_info.framebuffer.as_mut() {
    let info = fb.info();
    let buffer = fb.buffer_mut();

    // Each pixel is 3 bytes (BGR)
    let offset = (y * info.stride + x) * info.bytes_per_pixel;
    buffer[offset]     = blue;
    buffer[offset + 1] = green;
    buffer[offset + 2] = red;
}
```

Displaying text requires a bitmap font (8x8 pixel grids per character). Works but verbose.

#### 3. Serial Port (Simplest — What We Use)

Write bytes to COM1 via x86 `in`/`out` instructions; QEMU redirects to your terminal:

```rust
unsafe fn x86_port_read(port: u16) -> u8 {
    let value: u8;
    core::arch::asm!("in al, dx", out("al") value, in("dx") port);
    value
}

unsafe fn x86_port_write(port: u16, value: u8) {
    core::arch::asm!("out dx, al", in("al") value, in("dx") port);
}

fn serial_print_byte(byte: u8) {
    unsafe {
        while (x86_port_read(0x3FD) & 0x20) == 0 {}  // wait for transmit buffer empty
        x86_port_write(0x3F8, byte);                   // write byte to COM1
    }
}
```

- **`0x3F8`** — COM1 data register
- **`0x3FD`** — Line Status Register; bit 5 = transmit buffer empty
- **`in` / `out`** — x86 I/O port instructions, accessing a separate address space from memory

```bash
qemu-system-x86_64 -drive format=raw,file=blog_os-bios.img -nographic
# "Hello World!" prints to your terminal. Exit: Ctrl-A then X
```

#### Comparison

```
VGA:    CPU ──mov──→ memory 0xb8000  ──→ VGA controller ──→ monitor
Frame:  CPU ──mov──→ framebuffer mem ──→ GPU ──→ monitor
Serial: CPU ──out──→ I/O port 0x3F8  ──→ UART chip ──→ serial line (QEMU → terminal)
```

| | VGA Text Buffer | Framebuffer | Serial Port |
|---|---|---|---|
| Access | Memory address (pointer) | Memory address (pointer) | I/O port (`out` instruction) |
| Address space | Memory | Memory | I/O (separate, x86-specific) |
| Needs waiting | No | No | Yes (check transmit buffer) |
| Bootloader 0.11 | Not available | Available via `BootInfo` | Always available |
| Output | Monitor (text grid) | Monitor (pixels) | Terminal (text stream) |

x86 has two separate address spaces: memory (accessed via `mov`) and I/O ports (accessed via `in`/`out`). Most modern devices use MMIO; I/O ports are an older mechanism still used by serial ports, PS/2 keyboard, etc.

### Panic Handler

```rust
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
```

Same as chapter 1. Later chapters will print error messages via serial/screen.

### ELF and the Disk Image

#### What is ELF

ELF (Executable and Linkable Format) is what `cargo build` produces:

```
target/x86_64-unknown-none/debug/blog_os    ← ELF file
```

Contains: machine code, entry point address (`_start`), section info (code/data addresses and sizes). Think of it as a labeled package telling the loader where to put each piece of code in memory.

#### How the ELF Becomes a Bootable Image

```bash
cargo build          # 1. compile kernel → ELF
cd ../os-boot        # 2. separate host-target crate
cargo run -- <ELF>   #    uses DiskImageBuilder to package ELF + bootloader → .img
```

`os-boot` receives the ELF path via `env::args().nth(1)`, then `DiskImageBuilder` creates the disk image.

#### Why `os-boot` Lives Outside the Kernel Tree

It needs `std` (file I/O, etc.), but the kernel's `.cargo/config.toml` forces `x86_64-unknown-none` (no `std`). Cargo config inherits downward and can't be cleanly overridden. So the disk image builder must be a separate crate outside the kernel directory.

#### Disk Image Layout

```
blog_os-bios.img
┌──────────────────┐  Sector 0
│  bootloader      │  ← BIOS starts here
│  (boot code)     │     16-bit → 32-bit → 64-bit, page tables...
├──────────────────┤
│  kernel ELF      │  ← bootloader parses ELF,
│  (code + data)   │     loads to correct addresses, jumps to _start
└──────────────────┘
```

QEMU loads this `.img` just like real hardware booting from a disk.

### Build & Run

```bash
./run.sh          # builds kernel, creates disk image, launches QEMU
```

Or manually:

```bash
cargo build
cd ../os-boot && cargo run -- ../os/target/x86_64-unknown-none/debug/blog_os
qemu-system-x86_64 -drive format=raw,file=target/x86_64-unknown-none/debug/blog_os-bios.img -nographic
```

### Key Changes from Chapter 1

| | Chapter 1 (Freestanding Binary) | Chapter 2 (Minimal Kernel) |
|---|---|---|
| Runtime | macOS kernel | Bare metal (QEMU) |
| Binary format | Mach-O | ELF (loaded by bootloader) |
| Entry point | Hand-written `_start` + linker args | `entry_point!` macro |
| Output | None (just `loop {}`) | Serial port → terminal |
| Linker config | `-e __start -static -nostartfiles` | Not needed (target handles it) |

### Pitfalls (Blog vs. Modern Toolchain)

The blog was written for `bootloader 0.9` + older nightly. Modern toolchains (2024+) require adaptations:

1. **Bootloader 0.9 incompatible with modern nightly** — `+soft-float` banned, `target-pointer-width` type changed (string → integer), `data-layout` format updated, bootloader's internal JSON specs outdated. Pinning old nightlies cascades into more breakage (lockfile format, attribute syntax). **Fix**: use `bootloader_api 0.11` + `bootloader 0.11`

2. **No VGA text mode** — Bootloader 0.11 switches to a graphical framebuffer even in BIOS mode. Writing to `0xb8000` does nothing. **Fix**: use framebuffer from `BootInfo`, or serial port output

3. **PIE executables** — `x86_64-unknown-none` defaults to PIE, which can cause bootloader failures. **Fix**: `rustflags = ["-C", "relocation-model=static"]`

4. **Disk image builder is now a library** — Bootloader 0.9 had `cargo bootimage` CLI. Bootloader 0.11 provides `DiskImageBuilder` API — requires a separate host-target crate

5. **Custom JSON target spec unnecessary** — The blog's `x86_64-blog_os.json` is replaced by the built-in `x86_64-unknown-none` target, avoiding JSON format compatibility issues across nightly versions
