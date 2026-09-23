//! Raw FFI bindings to the scriptc runtime, MSVC/UCRT build.
//!
//! Success criterion for this crate: it links and the FFI surface can be
//! called from Rust without errors.

#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals, dead_code)]

use std::os::raw::c_char;

/// ScrStr — refcounted UTF-8 string (layout per scr_runtime.h).
#[repr(C)]
pub struct ScrStr {
    pub rc: usize,
    pub len: usize,
    pub cap: usize,
    /// flexible array member — access via scr_str_data()
    pub _data: u8,
}

/// ScrBytes — refcounted byte buffer. Layout mirrors ScrStr's header
/// (rc/len/cap + flexible data).
#[repr(C)]
pub struct ScrBytes {
    pub rc: usize,
    pub len: usize,
    pub cap: usize,
    pub _data: u8,
}

extern "C" {
    // ── init / lifecycle ────────────────────────────────────────────
    /// called once at the top of main (console formatter, flush-at-exit)
    pub fn scr_init();
    /// versioned ABI marker — mismatched runtime links fail here
    pub fn scr_runtime_abi_v1();

    // ── program TU entry ────────────────────────────────────────────
    /// The scriptc-generated program TU (`scriptc build *.ts --backend=c`)
    /// is compiled with `/Dmain=scr_program_main` (see build.rs), renaming
    /// its generated `main` so it can be invoked from Rust. Calling it runs
    /// the full generated preamble: scr_init() → error-vts stamping →
    /// scr_lib_init(argc, argv) → the mangled TS entry. Provided the TS
    /// program's top-level/main path terminates, this returns its exit code.
    /// Provided by the `scriptc-program` static lib (runtime-lab/hello.c).
    pub fn scr_program_main(argc: std::os::raw::c_int, argv: *mut *mut c_char) -> std::os::raw::c_int;

    // ── strings ─────────────────────────────────────────────────────
    pub fn scr_str_new(bytes: *const c_char, len: usize) -> *mut ScrStr; /* +1 */
    pub fn scr_str_release(s: *mut ScrStr); /* NULL-tolerant */
    pub fn scr_str_eq(a: *mut ScrStr, b: *mut ScrStr) -> bool;
    pub fn scr_str_length(s: *const ScrStr) -> usize;
}

/// scr_str_retain is `static inline` in scr_runtime.h — re-export it as a
/// Rust-side inline that performs the same RC bump (non-embedded immortal
/// rc == SIZE_MAX stays untouched).
#[inline]
pub unsafe fn scr_str_retain(s: *mut ScrStr) -> *mut ScrStr {
    if (*s).rc != usize::MAX {
        (*s).rc += 1;
    }
    s
}

/// Borrow the string's bytes (data follows the header, NUL-terminated).
#[inline]
pub fn scr_str_data(s: &ScrStr) -> &[u8] {
    let ptr = std::ptr::addr_of!(s._data);
    unsafe { std::slice::from_raw_parts(ptr, s.len) }
}
