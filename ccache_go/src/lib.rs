use libc::{c_void, c_char, c_int};
use std::ffi::CStr;

extern "C" {
    fn CallEncode(data_ptr: *mut c_void, type_ptr: *mut c_void, size: *mut c_int) -> *const c_char;
}

#[no_mangle]
pub extern "C" fn call_greet(data_ptr: *mut c_void, type_ptr: *mut c_void) {
    println!("data_ptr {:?}", data_ptr);
    println!("type_ptr {:?}", type_ptr);

    unsafe {
        let mut size: c_int = 0;
        let c_str: *const c_char = CallEncode(data_ptr, type_ptr, &mut size as *mut c_int);

        if !c_str.is_null() {
            let c_str = CStr::from_ptr(c_str);
            let bytes = c_str.to_bytes();
    
            // Print the bytes
            println!("Bytes: {:?}", bytes);
        } else {
            println!("Received a null pointer from the C function");
        }
    }
}
