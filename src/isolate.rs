use std::mem;
use std::sync;
use v8_sys as v8;
use allocator;
use platform;

static INITIALIZE: sync::Once = sync::ONCE_INIT;

/// Isolate represents an isolated instance of the V8 engine.  V8 isolates have completely separate
/// states.  Objects from one isolate must not be used in other isolates.  The embedder can create
/// multiple isolates and use them in parallel in multiple threads.  An isolate can be entered by at
/// most one thread at any given time.  The Locker/Unlocker API must be used to synchronize.
#[derive(Debug)]
pub struct Isolate(v8::IsolatePtr, allocator::Allocator);

impl Isolate {
    pub fn new() -> Isolate {
        ensure_initialized();

        let allocator = allocator::Allocator::new();
        let raw = unsafe { v8::Isolate_New(allocator.as_raw()) };

        if raw.is_null() {
            panic!("Could not create Isolate");
        }

        unsafe { v8::Isolate_SetCaptureStackTraceForUncaughtExceptions_Detailed(raw, 1, 1024) };

        Isolate(raw, allocator)
    }

    pub fn as_raw(&self) -> v8::IsolatePtr {
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
    INITIALIZE.call_once(|| {
        unsafe {
            v8::V8_InitializeICU();

            let platform = platform::Platform::new();
            v8::V8_InitializePlatform(platform.as_raw());
            // TODO: implement some form of cleanup
            mem::forget(platform);

            v8::V8_Initialize();
        }
    });
}
