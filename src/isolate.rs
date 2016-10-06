use std::mem;
use std::sync;
use v8_sys as v8;
use allocator;
use context;
use platform;

static INITIALIZE: sync::Once = sync::ONCE_INIT;

/// Isolate represents an isolated instance of the V8 engine.  V8 isolates have completely separate
/// states.  Objects from one isolate must not be used in other isolates.  The embedder can create
/// multiple isolates and use them in parallel in multiple threads.  An isolate can be entered by at
/// most one thread at any given time.  The Locker/Unlocker API must be used to synchronize.
#[derive(Debug)]
pub struct Isolate(IsolateHolder);

#[derive(Debug)]
enum IsolateHolder {
    Owned(v8::IsolatePtr, allocator::Allocator),
    Borrowed(v8::IsolatePtr),
}

impl Isolate {
    pub fn new() -> Isolate {
        ensure_initialized();

        let allocator = allocator::Allocator::new();
        let raw = unsafe { v8::Isolate_New(allocator.as_raw()) };

        if raw.is_null() {
            panic!("Could not create Isolate");
        }

        unsafe { v8::Isolate_SetCaptureStackTraceForUncaughtExceptions_Detailed(raw, 1, 1024) };

        Isolate(IsolateHolder::Owned(raw, allocator))
    }

    pub unsafe fn from_raw(raw: v8::IsolatePtr) -> Isolate {
        Isolate(IsolateHolder::Borrowed(raw))
    }

    pub fn as_raw(&self) -> v8::IsolatePtr {
        match self.0 {
            IsolateHolder::Owned(p, _) => p,
            IsolateHolder::Borrowed(p) => p,
        }
    }

    pub fn current_context(&self) -> Option<context::Context> {
        unsafe { 
            let raw = v8::Isolate_GetCurrentContext(self.as_raw()).as_mut();
            raw.map(|r| context::Context::from_raw(self, r))
        }
    }
}

impl Drop for Isolate {
    fn drop(&mut self) {
        unsafe {
            match self.0 {
                IsolateHolder::Owned(p, _) => v8::Isolate_Dispose(p),
                _ => (),
            }
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
