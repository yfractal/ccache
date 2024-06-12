use libc::{c_char, c_int, c_void};
use std::collections::HashMap;
use std::ffi::CStr;
use std::ptr;

static mut GLOBAL_PTR: *mut Ccache = ptr::null_mut();

extern "C" {
    fn Encode(data_ptr: *mut c_void, type_ptr: *mut c_void, size: *mut c_int) -> *const u8;
}

#[derive(Debug)]
pub struct Ccache {
    inner: HashMap<String, *mut c_void>,
}

impl Ccache {
    pub fn init() -> *mut c_void {
        let data = Ccache::new();

        unsafe {
            GLOBAL_PTR = Box::into_raw(Box::new(data));
            GLOBAL_PTR as *mut c_void
        }
    }

    pub fn new() -> Self {
        Ccache {
            inner: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: String, data_ptr: *mut c_void, type_ptr: *mut c_void) {
        unsafe {
            let mut size: c_int = 0;
            let c_str: *const u8 = Encode(data_ptr, type_ptr, &mut size as *mut c_int);

            if !c_str.is_null() {
                let bytes = std::slice::from_raw_parts(c_str, size as usize).to_vec();
                println!("Bytes in insert: {:?}", bytes);
                for byte in bytes {
                    print!("{:02x} ", byte);
                }
            }
        }

        self.inner.insert(key, data_ptr);
    }
}

#[no_mangle]
pub extern "C" fn ccache_init() {
    Ccache::init();
}

#[no_mangle]
pub extern "C" fn ccache_insert(key: *const c_char, data_ptr: *mut c_void, type_ptr: *mut c_void) {
    unsafe {
        let boxed_data = Box::from_raw(GLOBAL_PTR);
        let mut data = *boxed_data;
        let c_str = CStr::from_ptr(key);
        let str_slice = c_str.to_str().expect("Failed to convert CStr to &str");
        data.insert(str_slice.to_owned(), data_ptr, type_ptr);
    }
}
