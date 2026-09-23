//! FFI smoke test — 成功判断：Rust 中可以导入、调用 FFI 不报错。
//!
//! 验证链：link scriptc-runtime.lib → 调 scr_init / scr_runtime_abi_v1 /
//! scr_str_new / scr_str_retain / scr_str_release → 观察引用计数与字节。

use scriptc_runtime_sys::*;

#[test]
fn ffi_links_and_calls() {
    // 1. ABI 版本符号存在且可调用（mismatched runtime link 在这里直接失败）
    unsafe {
        scr_runtime_abi_v1();
    }

    // 2. init（私有 stdout formatter / flush-at-exit 注册）
    unsafe {
        scr_init();
    }

    // 3. ScrStr 生命周期：new → retain → release × 2
    let s = unsafe { scr_str_new(b"hello scriptc\0".as_ptr() as *const _, 13) };
    assert!(!s.is_null(), "scr_str_new returned NULL");

    unsafe {
        // 引用计数：初始 rc == 1
        assert_eq!((*s).rc, 1, "fresh string rc must be 1");
        assert_eq!((*s).len, 13);

        let data = scr_str_data(&*s);
        assert_eq!(data, b"hello scriptc");

        // retain → rc == 2
        let t = scr_str_retain(s);
        assert_eq!((*t).rc, 2, "retain must bump rc");

        // 相等比较
        let u = scr_str_new(b"hello scriptc\0".as_ptr() as *const _, 13);
        assert!(scr_str_eq(s, u), "identical strings must be equal");

        // release 两次回到平衡
        scr_str_release(u);
        scr_str_release(t);
        scr_str_release(s); // 最后一次释放（NULL-tolerant API）
    }
}

#[test]
fn ffi_empty_and_ascii_edge() {
    unsafe {
        scr_init();

        let e = scr_str_new(b"\0".as_ptr() as *const _, 0);
        assert_eq!((*e).len, 0);
        assert_eq!(scr_str_data(&*e), b"");
        scr_str_release(e);

        // 长 ASCII 串
        let long = "x".repeat(10_000);
        let l = scr_str_new(long.as_ptr() as *const _, long.len());
        assert_eq!((*l).len, 10_000);
        scr_str_release(l);
    }
}
