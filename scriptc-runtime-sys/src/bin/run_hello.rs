//! Runs the scriptc-generated program TU (runtime-lab/netchild.c) inside
//! this Rust binary. `scr_program_main` is the generated `main` renamed at
//! compile time (/Dmain=scr_program_main, see build.rs); it performs the
//! full generated preamble itself:
//!
//!   scr_init() → error-vts stamping → scr_lib_init(argc, argv) → TS entry
//!
//! The program exercises at runtime the units the PoC only ever compiled:
//!   - child_process.spawnSync (posix_spawnp-equivalent → CreateProcess)
//!   - setTimeout (the event loop's timer wheel)
//!   - net.createServer/connect (TCP loopback over the WSAPOLL poller)
//!
//! Note: the bin must reference `scriptc_runtime_sys` (even trivially) so
//! cargo keeps the sys crate's rlib — and with it the `links` metadata that
//! adds scriptc-runtime.lib / scriptc-program.lib to the link line.

use scriptc_runtime_sys::scr_program_main;

fn main() {
    let code = unsafe { scr_program_main(0, std::ptr::null_mut()) };
    if code != 0 {
        std::process::exit(code);
    }
}
