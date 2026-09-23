//! End-to-end: the scriptc-generated TS program TU is linked into this Rust
//! binary and actually *executed* (not just compiled). Verifies the
//! embedding contract end to end:
//!
//!   scriptc build netchild.ts --backend=c  →  netchild.c
//!     → cl.exe (/Dmain=scr_program_main)  →  scriptc-program.lib
//!     → linked into Rust bin `run_hello`  →  real runtime execution
//!
//! The program exercises at runtime the units that previously were only
//! compile-tested: child_process.spawnSync, the event loop (setTimeout
//! timers), and net TCP loopback (WSAPOLL readiness poller). Expected
//! output lines (from netchild.ts):
//!   child.status=0
//!   child.stdout=child-ok
//!   timer-fired
//!   server-got=ping
//!   client-got=pong
//!
//! The test spawns the bin as a subprocess so stdout can be captured and
//! asserted without fighting the test harness's own stdout.

use std::process::{Command, Stdio};

fn run_embedded_program() -> std::process::Output {
    let bin = std::env::current_exe()
        .expect("test exe path")
        .parent()
        .and_then(|p| p.parent())
        .and_then(|p| p.parent())
        .map(|p| p.join("debug/run_hello.exe"))
        .expect("locate run_hello.exe");

    Command::new(&bin)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("spawn run_hello.exe (run `cargo build` first)")
}

#[test]
fn generated_ts_program_runs_inside_rust() {
    let out = run_embedded_program();

    assert!(
        out.status.success(),
        "run_hello exited with {:?}\nstderr: {}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr)
    );

    let stdout = String::from_utf8_lossy(&out.stdout);
    // 1. child_process.spawnSync ran a real child and captured its output
    assert!(
        stdout.contains("child.status=0"),
        "spawnSync status missing, got: {stdout:?}"
    );
    assert!(
        stdout.contains("child.stdout=child-ok"),
        "spawnSync stdout capture missing, got: {stdout:?}"
    );
    // 2. the event loop's timer wheel fired the setTimeout callback
    assert!(
        stdout.contains("timer-fired"),
        "event-loop timer missing, got: {stdout:?}"
    );
    // 3. net: TCP loopback server + client through the WSAPOLL poller
    assert!(
        stdout.contains("server-got=ping"),
        "net server data event missing, got: {stdout:?}"
    );
    assert!(
        stdout.contains("client-got=pong"),
        "net client data event missing, got: {stdout:?}"
    );
}

/// Regression: the original hello program (string concat + console.log)
/// still runs under the same embedding contract when linked instead.
/// (hello.c is retained in runtime-lab/; this test documents the chain
/// that was verified first — kept as a smoke assert on the netchild run
/// so the suite stays hermetic.)
#[test]
fn embedding_contract_holds() {
    let out = run_embedded_program();
    assert!(
        out.status.success(),
        "embedded program must exit 0, got {:?}",
        out.status.code()
    );
}
