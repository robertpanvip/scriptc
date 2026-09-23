//! build.rs — compile the vendored scriptc runtime with the host MSVC
//! toolchain (cc crate → cl.exe) so the objects share the Rust binary's
//! UCRT instance.

use std::path::PathBuf;

fn main() {
    let root = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let vendor = root.join("../vendor/scriptc-runtime");
    let src = vendor.join("src");
    let shim = vendor.join("shim");
    let mbedtls = vendor.join("vendor-mbedtls");

    let mut build = cc::Build::new();

    // ── source selection ────────────────────────────────────────────────
    // Base executable runtime (native-toolchain.ts EXECUTABLE_RUNTIME_SOURCES
    // minus scr_musl.c which is musl-only). Plus the Win32 shims and the
    // dirent/clock MSVC shims.
    let base = [
        "scr_number.c",
        "scr_string.c",
        "scr_array.c",
        "scr_bytes.c",
        "scr_bytes_io.c",
        "scr_map.c",
        "scr_closure.c",
        "scr_ffi.c",
        "scr_object.c",
        "scr_union.c",
        "scr_exception.c",
        "scr_error.c",
        "scr_console.c",
        "scr_lib.c",
        "scr_path.c",
        "scr_url.c",
        "scr_json.c",
        "scr_async.c",
        "scr_crypto_async.c",
        "scr_child.c",
        "scr_cycle.c",
        // win32 arm
        "scr_win.c",
        // optional units we ship in the PoC (regex needs libregexp; zlib needs vendored zlib)
        "scr_regex.c",
        "scr_assert.c",
        "scr_zlib.c",
        // TLS (mbedtls) + Windows CA store units — full feature parity lane
        "scr_tls.c",
        "scr_tls_ca.c",
        // net slice (TCP servers/sockets) + the Windows readiness poller
        // that backs the event loop's listen/connect/timer activity
        "scr_net.c",
        "scr_loop_wsapoll.c",
        // dynamic-listener island (net callbacks fire through it)
        "scr_dyn_handle.c",
        // msvc-specific shims
        "scr_msvc_clock.c",
    ];
    // GCC/Clang builtin shims (popcount/clz/ctz used by scr_string.c etc.)
    build.file(shim.join("scr_msvc_builtins.c"));
    for f in base {
        if f == "scr_msvc_clock.c" {
            build.file(shim.join(f));
        } else {
            build.file(src.join(f));
        }
    }
    build.file(shim.join("dirent-shim.c"));

    // ryu: scr_number.c does `#include "../vendor/ryu/d2s.c"` relative to src/,
    // so give the compiler the ryu dir AS ../vendor/ryu relative to src. cc's
    // include search resolves the quoted relative include against the
    // includer's directory first, so src/../vendor/ryu == vendor-ryu. Add an
    // explicit include mapping via a junction-free approach: copy path isn't
    // needed because MSVC resolves `../vendor/ryu/d2s.c` against src/ →
    // vendor/scriptc-runtime/vendor/ryu/d2s.c — create that alias below.
    // Simplest: keep a real directory at the expected relative location.
    build.include(vendor.join("vendor/ryu")); // ryu headers (d2s.c's own includes; src/../vendor/ryu)
    build.include(vendor.join("vendor-quickjs")); // libregexp.h etc.
    build.include(vendor.join("vendor-zlib")); // zlib.h
    build.include(mbedtls.join("include")); // mbedtls/*.h
    build.include(mbedtls.join("library")); // mbedtls' private headers
    build.include(&shim); // scr_msvc_compat.h / dirent-shim.h
    build.include(&src);

    // libregexp.c / libunicode.c (quickjs-ng matcher, ~110KB) — the regex
    // support files, compiled like upstream's LRE_SOURCES.
    build
        .file(vendor.join("vendor-quickjs/libregexp.c"))
        .file(vendor.join("vendor-quickjs/libunicode.c"))
        .file(vendor.join("vendor-quickjs/dtoa.c"));

    // zlib core sources (scr_zlib.c calls the zlib API directly)
    for f in [
        "adler32.c",
        "crc32.c",
        "compress.c",
        "deflate.c",
        "gzclose.c",
        "gzlib.c",
        "gzread.c",
        "gzwrite.c",
        "inflate.c",
        "uncompr.c",
        "zutil.c",
    ] {
        build.file(vendor.join("vendor-zlib").join(f));
    }

    // mbedTLS 3.6.7 — all library/*.c (109 files), matching the upstream
    // runtime-pack recipe (-I include -I library, no extra defines beyond
    // what its default mbedtls_config.h needs).
    for entry in std::fs::read_dir(mbedtls.join("library")).expect("mbedtls/library") {
        let path = entry.expect("mbedtls entry").path();
        if path.extension().and_then(|e| e.to_str()) == Some("c") {
            build.file(path);
        }
    }

    // ── flags ───────────────────────────────────────────────────────────
    build
        .std("c17")
        .define("_CRT_SECURE_NO_WARNINGS", None)
        .define("NOMINMAX", None)
        .define("WIN32_LEAN_AND_MEAN", None)
        .define("_WIN32_WINNT", "0x0A00") // Windows 10 floor, matches upstream target
        .define("QUICKJS_NG_BUILD", None)
        .define("_GNU_SOURCE", None)
        .flag("/utf-8")
        .warnings(false)
        .opt_level(2);

    // ── link metadata ───────────────────────────────────────────────────
    println!("cargo:rustc-link-lib=advapi32");
    println!("cargo:rustc-link-lib=iphlpapi");
    println!("cargo:rustc-link-lib=ws2_32");
    println!("cargo:rustc-link-lib=bcrypt");
    println!("cargo:rustc-link-lib=crypt32"); // scr_tls_ca.c: Windows cert stores
    println!("cargo:rustc-link-lib=userenv");

    // rerun triggers
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={}", src.display());
    println!("cargo:rerun-if-changed={}", shim.display());
    println!("cargo:rerun-if-changed={}", vendor.join("vendor-quickjs").display());
    println!("cargo:rerun-if-changed={}", vendor.join("vendor-zlib").display());
    println!("cargo:rerun-if-changed={}", vendor.join("vendor/ryu").display());
    println!("cargo:rerun-if-changed={}", mbedtls.display());
    build.compile("scriptc-runtime");

    // ── program TU (generated by `scriptc build *.ts --backend=c`) ──────
    // The emitter marks program functions `static`, so the only externally
    // visible symbol is `main`. Rename it via /Dmain=scr_program_main so it
    // can be linked into a Rust binary (no clash with Rust's own main) and
    // invoked from Rust. Calling it runs the full generated preamble:
    // scr_init() → error-vts stamping → scr_lib_init(argc, argv) → entry.
    // Active program: netchild.c — exercises child_process.spawnSync +
    // setTimeout + net TCP loopback through the WSAPOLL event loop.
    let program = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("../runtime-lab/netchild.c");
    if program.exists() {
        let mut prog = cc::Build::new();
        prog.file(&program)
            .include(&src)
            .include(&shim)
            .std("c17")
            .define("main", "scr_program_main")
            .flag("/utf-8")
            .warnings(false)
            .opt_level(2);
        println!("cargo:rerun-if-changed={}", program.display());
        prog.compile("scriptc-program");
    }
}
// force rebuild 1790183216
