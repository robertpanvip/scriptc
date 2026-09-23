# scriptc Runtime MSVC 移植可行性报告

**日期**: 2026-09-24 · **上游**: vercel-labs/scriptc @ main (c8b4277) · **许可**: Apache-2.0（vendor 合法，保留 LICENSE）

## 目标

把 scriptc 的 C runtime（TS→native 编译器的执行时）从 "zig cc + MinGW sysroot" 脱钩为原生 MSVC/UCRT 工具链，最终以 `build.rs + cc` crate 形式编进用户的 Rust (x86_64-pc-windows-msvc) 二进制——与 rquickjs 嵌 QuickJS 同构的集成形态。

## 试编译结果（VS2022 MSVC 14.44 + Windows SDK 10.0.26100，/std:c17 /utf-8）

| 结果 | 数量 | 说明 |
|---|---|---|
| **PASS** | **43** | 直接编译通过，零改动 |
| **FAIL** | **16** | 全部为浅层问题，见分类 |
| SKIP | 4 | epoll/kqueue/musl/curl——平台专属，本就不在 win 构建面 |

**结论先行：43/63 文件零改动即编译通过。没有任何架构级障碍。**

## 16 个失败的分类与处置

### 类别 1 — 缺三方源（非代码问题，拉 vendor 即可）
| 文件 | 缺什么 | 处置 |
|---|---|---|
| scr_number.c | `../vendor/ryu/d2s.c` | 拉 ryu（MSVC-clean） |
| scr_zlib.c / scr_fetch.c | `zlib.h` | 拉 zlib（官方支持 MSVC） |
| scr_regex.c | `libregexp.h` | QuickJS 的 regex 引擎，随 runtime 包 vendor 拉齐 |

### 类别 2 — unistd.h（8 文件，最浅）
scr_path / scr_url / scr_watch / scr_dgram / scr_net / scr_events / scr_tls / scr_lib
- 实际只调 `getcwd`、`isatty` 等少量函数。
- 处置：compat shim 头里 `#define getcwd _getcwd`、`#define isatty _isatty`（UCRT 自带），每文件 1-2 行改动。

### 类别 3 — dirent.h（1 处）
scr_lib.c 的 `opendir/readdir`（rmSync 递归删除等）。
- MSVC 无 dirent；scr_lib.c 的 `_WIN32` arm 大部分已用 Win32 原生面，剩这一处 POSIX 残留。
- 处置：20 行 FindFirstFileW shim，或改写该函数用已有的 Win32 arm。

### 类别 4 — MinGW 专属残留（2 文件）
- scr_win.c: `clock_gettime32/nanosleep32` 用了 MinGW pthread 头里的 `clockid_t`、`struct _timespec32`。这些是给 MinGW 兼容面补的符号；MSVC arm 里应改为内部静态实现 + 导出统一的 `scr_clock_gettime`。
- scr_file_handle.c:262: `mode_t` 强转 + `O_BINARY`（MinGW 有，MSVC 用 `_O_BINARY` + `_open` 的 mode 参数语义不同）。

### 类别 5 — CLOCK_* 宏缺失（2 文件）
scr_bytes_io.c:253 / scr_async.c:442 调 `clock_gettime(CLOCK_REALTIME/CLOCK_MONOTONIC, &ts)`。
- scr_win.c 已有完整的高精度实现（GetSystemTimePreciseAsFileTime / QueryPerformanceCounter），但是 `static` 没导出。
- 处置：把 shim 提为可链接的 `clock_gettime/nanosleep`（签名用 runtime 自己的 timespec 类型），或 compat 头定义 CLOCK_* 宏 + 中转。约 30 行。

## 有利证据（降低风险）

1. **上游已有 MSVC 意识**：scr_runtime.h 里 `#if defined(_WIN32) && defined(_MSC_VER)` 分支定义了 `ssize_t`，且注释明确留了"MSVC 路线"的口子。
2. **异步/进程的 _WIN32 arm 完整且高质量**：scr_async.c 用 SRWLOCK/CONDITION_VARIABLE，scr_child.c 用 CreateProcessW 并逐条对照 libuv/Node-on-Windows 语义（含 PATH 搜索、引号转义算法）。event loop 有专门的 scr_loop_wsapoll.c 后端。TLS CA 走 Windows 证书库。
3. **纯 C、无汇编、无 longjmp 魔法依赖特定 CRT**（异常面用自家 trap 机制），ABI 就是普通 COFF C 调用约定。
4. **CRT 一致性目标可达**：全部用 UCRT 重编后，与 Rust MSVC 工具链共享同一 UCRT 实例——这正是本次脱钩的动机。

## 工作量估算

| 项 | 量级 |
|---|---|
| compat shim 头（unistd/dirent/CLOCK/mode_t） | ~1 天 |
| 三方 vendor 拉齐 + 编译脚本 | ~0.5 天 |
| build.rs + cc 集成 + 链接 advapi32/ws2_32/iphlpapi/bcrypt/crypt32 | ~0.5 天 |
| 冒烟验证（编译一段 TS→obj 链进 Rust 跑 hello world + 定时器） | ~1 天 |
| **合计** | **约 3 个工作日**（不含完整语义测试） |

风险项：仅 zlib/mbedtls 交叉依赖需确认版本对齐（mbedtls 在 crypto TLS 路径才需要，PoC 可先跳过 scr_crypto_async/scr_tls）。

## 与 fork 方案的对比

| 方案 | 改动面 | 可维护性 |
|---|---|---|
| ~~fork 编译器改 targets.ts/native-toolchain.ts~~ | 大（链接计划、缓存 key、CI） | 长期背 fork |
| **vendor runtime C + cc crate（本方案）** | 仅 runtime 层 ~10-15 处浅改 | 上游更新时重新 vendor + 重跑 shim，diff 面小 |

## 产物位置

- Runtime 源码快照: `E:\AI-workspace\scriptc\vendor\scriptc-runtime\src\`（69 文件 + LICENSE + UPSTREAM-package.json）
- 试编译脚手架: `E:\AI-workspace\scriptc\msvc-probe\compile-msvc-probe.sh`
- 全量错误日志: `E:\AI-workspace\scriptc\msvc-probe\<file>.log`
