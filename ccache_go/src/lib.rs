use libc::{c_char, c_int, c_void};
use std::collections::HashMap;
use std::ffi::CStr;

static mut CCACHE: Option<Ccache> = None;

extern "C" {
    fn Encode(data_ptr: *mut c_void, type_ptr: *mut c_void, length: *mut c_int) -> *const u8;
    fn Decode(bytes: *const u8, type_ptr: *mut c_void, length: c_int ) -> *const c_void;
}

#[derive(Debug)]
pub struct Ccache {
    inner: HashMap<String, *mut c_void>,
    bytes: HashMap<String, Vec<u8>>,
    length: HashMap<String, c_int>,
    types: HashMap<String, *mut c_void>,
}

impl Ccache {
    pub fn new() -> Self {
        Ccache {
            inner: HashMap::new(),
            bytes: HashMap::new(),
            length: HashMap::new(),
            types: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: String, data_ptr: *mut c_void, type_ptr: *mut c_void) {
        unsafe {
            let mut length: c_int = 0;
            let c_str: *const u8 = Encode(data_ptr, type_ptr, &mut length as *mut c_int);

            if !c_str.is_null() {
                let bytes = std::slice::from_raw_parts(c_str, length as usize).to_vec();
                self.bytes.insert(key.clone(), bytes);
                self.length.insert(key.clone(), length);
                self.types.insert(key.clone(), type_ptr);
            }
        }
       
        self.inner.insert(key, data_ptr);

    }

    pub fn get(&mut self, key: String) -> *const c_void {
        let type_ptr = *self.types.get(&key).unwrap();

        let bytes = self.bytes.get(&key).unwrap();
        let l = *self.length.get(&key).unwrap();

        unsafe {
            let rv = Decode(bytes.as_ptr(), type_ptr, l);
            rv
        }
    }
}

#[no_mangle]
pub extern "C" fn ccache_init() {
    unsafe {
        CCACHE = Some(Ccache::new());
    }
}

#[no_mangle]
pub extern "C" fn ccache_insert(key: *const c_char, data_ptr: *mut c_void, type_ptr: *mut c_void) {
    unsafe {
        let c_str = CStr::from_ptr(key);
        let str_slice = c_str.to_str().expect("Failed to convert CStr to &str");
        let ccache = CCACHE.as_mut().unwrap();
        ccache.insert(str_slice.to_owned(), data_ptr, type_ptr);
    }
}

#[no_mangle]
pub extern "C" fn ccache_get(key: *const c_char) -> *const c_void{
    unsafe {
        let c_str = CStr::from_ptr(key);
        let str_slice: &str = c_str.to_str().expect("Failed to convert CStr to &str");
        let ccache = CCACHE.as_mut().unwrap();
        ccache.get(str_slice.to_owned())
    }
}
