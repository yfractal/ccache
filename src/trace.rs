use std::os::raw::c_char;

#[repr(C)]
pub struct Event {
    pub method: [c_char; 32],
    pub event: [c_char; 16],
    pub key: [c_char; 32],
    pub trace_id: [c_char; 32],
}

impl Event {
    pub fn new(method: &str, event: &str, key: &str, trace_id: &str) -> Self {
        Self {
            method: Self::str_to_fixed(method),
            event: Self::str_to_fixed(event),
            key: Self::str_to_fixed(key),
            trace_id: Self::str_to_fixed(trace_id),
        }
    }

    pub fn as_ptr(&self) -> *const Self {
        self as *const Self
    }

    fn str_to_fixed<const N: usize>(s: &str) -> [i8; N] {
        let mut array = [0i8; N];
        let bytes = s.as_bytes();

        let len = bytes.len().min(N);
        for i in 0..len {
            array[i] = bytes[i] as i8;
        }

        array
    }
}
