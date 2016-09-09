use std::mem;
use std::sync::atomic;
use v8_sys as v8;
use allocator;
use platform;

static mut INITIALIZED: atomic::AtomicBool = atomic::ATOMIC_BOOL_INIT;

pub struct Isolate(*mut v8::Isolate, allocator::Allocator);

impl Isolate {
    pub fn new() -> Isolate {
        ensure_initialized();

        let allocator = allocator::Allocator::new();
        let raw = unsafe { v8::Isolate_New(allocator.as_raw()) };

        if raw.is_null() {
            panic!("Could not create Isolate");
        }

        Isolate(raw, allocator)
    }

    pub fn as_raw(&self) -> *mut v8::Isolate {
        self.0
    }
}

impl Drop for Isolate {
    fn drop(&mut self) {
        unsafe {
            v8::Isolate_Dispose(self.0);
        }
    }
}

fn ensure_initialized() {
    unsafe {
        if !INITIALIZED.swap(true, atomic::Ordering::Relaxed) {
            v8::V8_InitializeICU();

            let platform = platform::Platform::new();
            v8::V8_InitializePlatform(platform.as_raw());
            // TODO: implement some form of cleanup
            mem::forget(platform);

            v8::V8_Initialize();
        }
    }
}
