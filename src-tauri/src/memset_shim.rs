/// Windows MinGW memset_explicit shim
/// libsodium-sys-stable expects memset_explicit which isn't available in MinGW
/// This provides a compatible implementation

#[cfg(target_os = "windows")]
#[no_mangle]
pub extern "C" fn memset_explicit(dest: *mut u8, val: i32, n: usize) {
    unsafe {
        for i in 0..n {
            *dest.add(i) = val as u8;
        }
    }
}
