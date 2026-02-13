# 用 Rust 写操作系统 - 笔记

## 1. 独立二进制文件

去掉标准库和 C 运行时的 macOS 可执行文件——为裸机开发做准备。

### 关键属性

- `#![no_std]` - 不链接 Rust 标准库（没有 OS 抽象可用）
- `#![no_main]` - 不使用默认入口点 `main`，手动定义 `_start`

### 入口点：`_start`

```rust
#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! { loop {} }
```

- `#[unsafe(no_mangle)]` - 保留函数名不被修饰。没有它 Rust 会把名字改成类似 `_ZN7blog_os6_start17h...` 的形式，链接器就找不到入口点了
- `extern "C"` - 使用 C 调用约定，链接器才能识别
- `-> !` - 发散函数（永不返回），因为没有东西可以返回到

验证：`nm target/debug/blog_os | grep start` → `T __start`

### Panic 处理

- 默认策略：**栈展开 (unwinding)** - 逐帧回溯调用栈，调用析构函数 (`drop`) 做清理
- 栈展开依赖操作系统支持（`eh_personality` 语言项、libunwind 等）
- 裸机没有 OS，所以在 `Cargo.toml` 中设置 `panic = "abort"` 直接终止

### Cargo.toml：`test = false` / `bench = false`

```toml
[[bin]]
name = "blog_os"
test = false
bench = false
```

- 默认为 `true`，Cargo 会构建依赖 `std` 的**测试框架**
- `std` 自带 `panic_impl` → 和你的 `#[panic_handler]` 冲突（重复 `lang item` 错误）
- `--all-targets` 或 `cargo test` 会触发这个问题
- 后续教程会用 `#![feature(custom_test_frameworks)]` 替代

### macOS 链接器配置

`.cargo/config.toml`：

```toml
[target.'cfg(target_os = "macos")']
rustflags = ["-C", "link-args=-e __start -static -nostartfiles"]
```

- `-e __start` - macOS 给所有 C 符号加 `_` 前缀，所以 `_start` 在符号表中变成 `__start`
- `-static` - 不链接 `libSystem.dylib`（macOS 官方不支持静态二进制，但我们需要避免 OS 依赖）
- `-nostartfiles` - 跳过 `crt0`（C 运行时启动代码）；否则链接器会找 `main` 函数，而我们没有（`#![no_main]`）

Linux 只需要 `-nostartfiles`。

### `-nostartfiles` 的实际作用

macOS **有** C 运行时 (`crt0`)。这个标志不是说它不存在，而是告诉链接器**不要使用它**。

```
不加 -nostartfiles：OS 加载二进制 → crt0 初始化 → 找 main() → 报错（没有 main）
加了 -nostartfiles：OS 加载二进制 → 直接跳转到 _start → 执行你的代码
```

### 当前阶段 vs. 裸机

此时二进制文件仍然是 **macOS 可执行文件**（Mach-O 格式），由 macOS 内核加载运行。目的是练习去掉 `std` 和 `crt0` 依赖。真正的裸机在下一章。

---

## 2. 最小 Rust 内核

从 macOS 可执行文件到真正运行在模拟 x86_64 硬件上的裸机内核。

### 启动流程

```
开机 → BIOS 固件 → 找到引导扇区（512 字节）→ 加载 bootloader
→ bootloader：16 位实模式 → 32 位保护模式 → 64 位长模式
→ 设置页表，解析内核 ELF，加载到内存 → 跳转到内核入口
```

bootloader 完成所有底层工作，我们可以直接用 Rust 写内核逻辑。

### 工具链：`rust-toolchain.toml`

```toml
[toolchain]
channel = "nightly"
components = ["rust-src", "llvm-tools-preview"]
```

- **nightly** - `build-std` 等是不稳定功能，stable 版本不可用
- **rust-src** - 标准库源码。我们的目标平台 (`x86_64-unknown-none`) 没有预编译的 `core`，必须从源码重新编译
- **llvm-tools-preview** - bootloader 创建磁盘映像时需要 `llvm-objcopy` 等工具

### 编译目标：`.cargo/config.toml`

```toml
[build]
target = "x86_64-unknown-none"
rustflags = ["-C", "relocation-model=static"]

[unstable]
build-std = ["core", "compiler_builtins"]
build-std-features = ["compiler-builtins-mem"]
```

第一章的目标是 macOS (`aarch64-apple-darwin`)。现在切换到 **`x86_64-unknown-none`** —— 裸机目标：没有 OS，没有 C 运行时，没有 `std`，只有 `core`。

**`rustflags`**：禁用 PIE（位置无关可执行文件）。内置目标默认生成 PIE，会导致 bootloader 出问题。

**`build-std`**：从源码重新编译 `core`（基本类型、Option、Result 等）和 `compiler_builtins`（编译器内建函数，如整数除法）。

**`compiler-builtins-mem`**：提供 `memcpy`、`memset`、`memcmp` 的实现。正常情况下来自 libc，但裸机没有 libc。

第一章的 macOS 链接器配置（`-e __start -static -nostartfiles`）不再需要。

#### 为什么用 `x86_64-unknown-none` 而不是自定义 JSON 目标

博客创建了自定义的 `x86_64-blog_os.json` 来描述裸机目标。用现代 nightly，内置的 `x86_64-unknown-none` 更简单，且已包含所有设置：

- **`disable-redzone: true`** - 红区是 System V ABI 的优化：函数可以使用栈指针下方 128 字节而不移动栈指针。中断会往那里压数据，导致损坏。内核中必须禁用
- **`-mmx,-sse,+soft-float`** - 禁用 SIMD，使用软件浮点。每次中断保存/恢复 512 位 XMM 寄存器开销很大，内核几乎不需要浮点运算
- **`panic-strategy: abort`** - panic 时直接终止，不做栈展开
- **`linker: rust-lld`** - Rust 自带的跨平台链接器

使用内置目标可以避免 JSON 格式在不同 nightly 版本间的兼容性问题（字段类型变化、data-layout 格式更新等）。

### 依赖：`Cargo.toml`

```toml
[dependencies]
bootloader_api = "0.11"
```

`bootloader_api` 提供 `entry_point!` 宏和 `BootInfo` 类型（内存映射、framebuffer 信息等）。

### 入口点

```rust
use bootloader_api::entry_point;

entry_point!(kernel_main);

fn kernel_main(_boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    // 内核代码
    loop {}
}
```

`entry_point!` 宏做两件事：

1. 生成真正的 `_start` 函数作为链接器入口（替代第一章手写的 `#[no_mangle] pub extern "C" fn _start`）
2. 对 `kernel_main` 做类型检查——确保签名是 `fn(&'static mut BootInfo) -> !`

比手写 `_start` 更安全，因为宏会在编译期捕获签名错误。

### 三种输出方式

#### 1. VGA 文本缓冲区（博客原版方案）

直接往内存地址 `0xb8000` 写字符：

```rust
let vga_buffer = 0xb8000 as *mut u8;
unsafe {
    *vga_buffer.offset(0) = b'H';     // 字符
    *vga_buffer.offset(1) = 0xb;      // 颜色：浅青色
}
```

- **`0xb8000`** - VGA 文本模式显存地址。**内存映射 I/O (MMIO)** —— 写入直接改变屏幕显示
- 每个字符 = 2 字节：ASCII 码 + 颜色属性（`背景色(4位) | 前景色(4位)`）
- `0x0b` = 黑底浅青字

```
地址        内容
0xb8000    'H'     ← 字符
0xb8001    0x0b    ← 颜色
0xb8002    'e'
0xb8003    0x0b
...
```

**bootloader 0.11 下不可用** —— 它会切换到图形 framebuffer，VGA 文本模式不再存在。

#### 2. Framebuffer（像素绘制）

bootloader 0.11 通过 `BootInfo` 提供图形 framebuffer（1280x720，BGR 格式）：

```rust
if let Some(fb) = boot_info.framebuffer.as_mut() {
    let info = fb.info();
    let buffer = fb.buffer_mut();

    // 每个像素 3 字节（BGR）
    let offset = (y * info.stride + x) * info.bytes_per_pixel;
    buffer[offset]     = blue;
    buffer[offset + 1] = green;
    buffer[offset + 2] = red;
}
```

显示文字需要位图字体（每个字符是 8x8 像素的网格）。能用但代码较多。

#### 3. 串口（最简单——我们使用的方案）

通过 x86 的 `in`/`out` 指令往 COM1 写字节，QEMU 重定向到终端：

```rust
/// 从 x86 I/O 端口读一个字节
unsafe fn x86_port_read(port: u16) -> u8 {
    let value: u8;
    core::arch::asm!("in al, dx", out("al") value, in("dx") port);
    value
}

/// 往 x86 I/O 端口写一个字节
unsafe fn x86_port_write(port: u16, value: u8) {
    core::arch::asm!("out dx, al", in("al") value, in("dx") port);
}

fn serial_print_byte(byte: u8) {
    unsafe {
        while (x86_port_read(0x3FD) & 0x20) == 0 {}  // 等发送缓冲区空
        x86_port_write(0x3F8, byte);                   // 写字节到 COM1
    }
}
```

- **`0x3F8`** — COM1 数据寄存器
- **`0x3FD`** — 线路状态寄存器 (LSR)；bit 5 = 发送缓冲区空
- **`in` / `out`** — x86 I/O 端口指令，访问的是独立于内存的 I/O 地址空间

```bash
qemu-system-x86_64 -drive format=raw,file=blog_os-bios.img -nographic
# "Hello World!" 直接打印到终端。退出：Ctrl-A 然后 X
```

#### 对比

```
VGA:         CPU ──mov──→ 内存 0xb8000   ──→ VGA 控制器 ──→ 显示器
Framebuffer: CPU ──mov──→ 帧缓冲内存     ──→ GPU        ──→ 显示器
串口:        CPU ──out──→ I/O 端口 0x3F8 ──→ UART 芯片  ──→ 串口线（QEMU → 终端）
```

| | VGA 文本缓冲区 | Framebuffer | 串口 |
|---|---|---|---|
| 访问方式 | 内存地址（指针） | 内存地址（指针） | I/O 端口（`out` 指令） |
| 地址空间 | 内存 | 内存 | I/O（独立，x86 特有） |
| 是否需要等待 | 不需要 | 不需要 | 需要（检查发送缓冲区） |
| Bootloader 0.11 | 不可用 | 通过 `BootInfo` 获取 | 始终可用 |
| 输出目标 | 显示器（文字网格） | 显示器（像素） | 终端（文字流） |

x86 有两套独立的地址空间：内存地址空间（用 `mov` 访问）和 I/O 地址空间（用 `in`/`out` 访问）。大多数现代设备已转向 MMIO；I/O 端口是较老的机制，串口、PS/2 键盘等传统设备仍在使用。

### Panic 处理

```rust
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
```

和第一章一样。后续章节会改成通过串口/屏幕打印错误信息。

### ELF 与磁盘映像

#### 什么是 ELF

ELF（Executable and Linkable Format，可执行和可链接格式）是 `cargo build` 生成的二进制格式：

```
target/x86_64-unknown-none/debug/blog_os    ← ELF 文件
```

包含：机器码（CPU 指令）、入口点地址（`_start` 在哪）、段信息（代码段/数据段的地址和大小）。可以把它理解为一个带目录的包裹，告诉加载者"把这段代码放到内存的这个位置，然后从这个地址开始执行"。

#### ELF 如何变成可启动映像

```bash
cargo build          # 1. 编译内核 → 生成 ELF
cd ../os-boot        # 2. 独立的宿主机 crate
cargo run -- <ELF>   #    用 DiskImageBuilder 把 ELF + bootloader 打包 → .img
```

`os-boot` 通过 `env::args().nth(1)` 接收 ELF 路径，然后 `DiskImageBuilder` 创建磁盘映像。

#### 为什么 `os-boot` 放在内核目录外面

它需要 `std`（文件 I/O 等），但内核的 `.cargo/config.toml` 强制所有东西都编译到 `x86_64-unknown-none`（没有 `std`）。Cargo 配置会向下继承，子目录无法干净地覆盖。所以磁盘映像构建器必须是内核目录外面的独立 crate。

#### 磁盘映像布局

```
blog_os-bios.img
┌──────────────────┐  扇区 0
│  bootloader      │  ← BIOS 从这里开始执行
│  (引导代码)       │     16 位 → 32 位 → 64 位，设置页表...
├──────────────────┤
│  内核 ELF        │  ← bootloader 解析 ELF，
│  (代码 + 数据)    │     加载到正确的内存地址，跳转到 _start
└──────────────────┘
```

QEMU 加载这个 `.img` 就像真实硬件从硬盘启动一样。

### 完整执行流程

从 `./run.sh` 到终端显示 "Hello World!" 的全链路：

#### 第一步：`cargo build`（编译内核）

```
src/main.rs (Rust 源码)
    │
    ▼  rustc 编译器（nightly，目标 x86_64-unknown-none）
    │
    │  1. 从源码编译 core 和 compiler_builtins（因为 build-std）
    │  2. 编译 bootloader_api（entry_point! 宏展开，生成 _start）
    │  3. 编译 blog_os（内核代码）
    │  4. rust-lld 链接所有目标文件
    │
    ▼
target/x86_64-unknown-none/debug/blog_os (ELF 文件)
```

ELF 文件内容：

```
┌─────────────────────────────┐
│ ELF Header                  │
│   entry point: 0x201450     │  ← _start 函数的地址
│                             │
│ .text (代码段)               │
│   _start → kernel_main      │
│   serial_print_byte          │
│   x86_port_read              │
│   x86_port_write             │
│                             │
│ .rodata (只读数据段)          │
│   b"Hello World!\n"         │
└─────────────────────────────┘
```

#### 第二步：`cd ../os-boot && cargo run -- "$KERNEL"`（创建磁盘映像）

```
os-boot/src/main.rs 被编译运行（宿主机 aarch64-apple-darwin，有 std）

命令行参数：
  argv[0] = "os-boot"
  argv[1] = ".../os/target/x86_64-unknown-none/debug/blog_os"  ← ELF 路径
                │
                ▼  env::args().nth(1)
          kernel_path = PathBuf::from(argv[1])
                │
                ▼  DiskImageBuilder::new(kernel_path)
                │
                │  DiskImageBuilder 内部：
                │  1. 读取内核 ELF 文件
                │  2. 读取 bootloader 自带的引导代码：
                │     - boot sector（512 字节，BIOS 第一个加载的）
                │     - second stage（16→32 位切换）
                │     - stage 3（32→64 位切换）
                │     - stage 4（解析 ELF，设页表，跳转内核）
                │  3. 拼接成一个磁盘映像
                │
                ▼  builder.create_bios_image(&bios_image)
                │
                ▼
target/x86_64-unknown-none/debug/blog_os-bios.img
```

磁盘映像内容：

```
blog_os-bios.img
┌────────────────────────┐  字节 0
│  Boot Sector (512B)    │  ← BIOS 固件加载这里到内存 0x7C00
│  跳转到 second stage    │
├────────────────────────┤
│  Second Stage          │  ← 16 位实模式 → 32 位保护模式
├────────────────────────┤
│  Stage 3               │  ← 32 位 → 64 位长模式
├────────────────────────┤
│  Stage 4               │  ← 解析 ELF，设置页表，映射 framebuffer
├────────────────────────┤
│  内核 ELF              │  ← blog_os 二进制（原封不动嵌入）
└────────────────────────┘
```

#### 第三步：`qemu-system-x86_64 ... -nographic`（启动虚拟机）

```
QEMU 参数：
  -drive format=raw,file=blog_os-bios.img → "这个文件是一块硬盘"
  -nographic → 不开窗口，串口 COM1 重定向到终端 stdout

QEMU 模拟 x86_64 计算机启动：

1. BIOS (SeaBIOS)
   │  QEMU 内置的 BIOS 固件
   │  初始化硬件，扫描硬盘
   │  找到 boot sector（磁盘第一个 512 字节）
   │  加载到内存地址 0x7C00
   │  CPU 跳转到 0x7C00（16 位实模式）
   │
   ▼
2. Boot Sector
   │  运行在 16 位实模式
   │  从磁盘读取 second stage 到内存
   │  跳转过去
   │
   ▼
3. Second Stage
   │  设置 GDT（全局描述符表）
   │  开启保护模式
   │  CPU：16 位 → 32 位
   │  从磁盘加载 stage 3 和 stage 4 到内存
   │
   ▼
4. Stage 3
   │  设置 64 位页表
   │  开启长模式
   │  CPU：32 位 → 64 位
   │
   ▼
5. Stage 4
   │  从磁盘读取内核 ELF 到内存 0x01000000
   │  解析 ELF header：
   │    - .text 段 → 加载到虚拟地址 0x201330
   │    - .rodata 段 → 加载到虚拟地址 0x200000
   │    - entry point → 0x201450
   │  设置 VESA framebuffer（1280x720）
   │  创建 BootInfo 结构体（内存映射、framebuffer 信息等）
   │  把 &mut BootInfo 作为参数
   │  跳转到 0x201450
   │
   ▼
6. _start（entry_point! 宏生成的）
   │  地址：0x201450
   │  接收 &mut BootInfo 参数
   │  调用 kernel_main(boot_info)
   │
   ▼
7. kernel_main
   │  调用 serial_print(b"Hello World!\n")
   │
   ▼
8. serial_print → serial_print_byte（逐字节发送）

   以 'H' (0x48) 为例：

   ┌─ x86_port_read(0x3FD) ─────────────────┐
   │  asm!("in al, dx")                      │
   │  CPU 执行 in 指令                        │
   │  从 I/O 端口 0x3FD 读取 LSR 寄存器       │
   │  检查 bit 5（发送缓冲区空？）             │
   │  0 → 继续循环等待                        │
   │  1 → 缓冲区空了，往下走                   │
   └──────────────────────────────────────────┘

   ┌─ x86_port_write(0x3F8, 0x48) ──────────┐
   │  asm!("out dx, al")                     │
   │  CPU 执行 out 指令                       │
   │  往 I/O 端口 0x3F8 写入 0x48 ('H')       │
   │       │                                 │
   │       ▼                                 │
   │  QEMU 截获这次 I/O 操作                   │
   │  QEMU 的虚拟 UART 设备收到 0x48          │
   │       │                                 │
   │       ▼  （-nographic 模式）              │
   │  QEMU 把 0x48 写到 stdout               │
   │       │                                 │
   │       ▼                                 │
   │  终端显示 'H'                            │
   └──────────────────────────────────────────┘

   重复：'e' 'l' 'l' 'o' ' ' 'W' 'o' 'r' 'l' 'd' '!' '\r' '\n'

   ▼
9. loop {}
   CPU 无限空转，内核"结束"
```

#### 数据流总结

```
源码                编译器              磁盘映像构建器           QEMU
─────              ──────             ──────────            ────

main.rs ──rustc──→ blog_os (ELF) ──DiskImageBuilder──→ blog_os-bios.img
                                                            │
                   argv[1] 传递 ELF 路径                      │
                                                            │
                                              -drive 参数传递映像路径
                                                            │
                                                            ▼
                                                    BIOS → bootloader
                                                            │
                                              bootloader 解析 ELF，
                                              把代码加载到内存，
                                              把 &mut BootInfo 作为参数
                                              跳转到 entry point
                                                            │
                                                            ▼
                                                      kernel_main
                                                            │
                                                   out 指令 → 端口 0x3F8
                                                            │
                                                   QEMU 截获 → stdout
                                                            │
                                                            ▼
                                                    终端显示 Hello World!
```

### 编译与运行

```bash
./run.sh          # 编译内核，创建磁盘映像，启动 QEMU
```

或者手动执行：

```bash
cargo build
cd ../os-boot && cargo run -- ../os/target/x86_64-unknown-none/debug/blog_os
qemu-system-x86_64 -drive format=raw,file=target/x86_64-unknown-none/debug/blog_os-bios.img -nographic
```

### 与第一章的关键变化

| | 第 1 章（独立二进制） | 第 2 章（最小内核） |
|---|---|---|
| 运行环境 | macOS 内核 | 裸机（QEMU 模拟） |
| 二进制格式 | Mach-O | ELF（由 bootloader 加载） |
| 入口点 | 手写 `_start` + 链接器参数 | `entry_point!` 宏 |
| 输出 | 无（只是 `loop {}`） | 串口 → 终端 |
| 链接器配置 | `-e __start -static -nostartfiles` | 不需要（目标自带） |

### 踩坑记录（博客 vs. 现代工具链）

博客基于 `bootloader 0.9` + 旧版 nightly 编写。现代工具链（2024+）需要以下适配：

1. **Bootloader 0.9 与现代 nightly 不兼容** — `+soft-float` 被禁止、`target-pointer-width` 类型从字符串变为整数、`data-layout` 格式更新、bootloader 内部 JSON 目标描述过时。钉死旧 nightly 会连锁引发更多问题（lockfile 格式、属性语法等）。**解决**：使用 `bootloader_api 0.11` + `bootloader 0.11`

2. **没有 VGA 文本模式** — Bootloader 0.11 即使在 BIOS 模式下也会切换到图形 framebuffer。往 `0xb8000` 写数据没有效果。**解决**：使用 `BootInfo` 中的 framebuffer，或串口输出

3. **PIE 可执行文件** — `x86_64-unknown-none` 默认生成 PIE，可能导致 bootloader 加载失败。**解决**：`rustflags = ["-C", "relocation-model=static"]`

4. **磁盘映像构建器变成了库** — Bootloader 0.9 有配套的 `cargo bootimage` CLI 工具。Bootloader 0.11 提供 `DiskImageBuilder` API——需要单独的宿主机 crate

5. **自定义 JSON 目标描述不再必要** — 博客的 `x86_64-blog_os.json` 被内置的 `x86_64-unknown-none` 目标替代，避免了 JSON 格式在不同 nightly 版本间的兼容性问题
