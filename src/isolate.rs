use std::mem;
use std::os;
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
pub struct Isolate(v8::IsolatePtr);

struct Data {
    count: usize,
    _allocator: allocator::Allocator,
}

const DATA_PTR_SLOT: u32 = 0;

impl Isolate {
    pub fn new() -> Isolate {
        ensure_initialized();

        let allocator = allocator::Allocator::new();

        let raw = unsafe { v8::Isolate_New(allocator.as_raw()) };
        if raw.is_null() {
            panic!("Could not create Isolate");
        }

        unsafe {
            assert!(v8::Isolate_GetNumberOfDataSlots(raw) > 0);
        }

        let data = Data {
            count: 1,
            _allocator: allocator,
        };
        let data_ptr: *mut Data = Box::into_raw(Box::new(data));

        unsafe {
            v8::Isolate_SetData(raw, DATA_PTR_SLOT, data_ptr as *mut os::raw::c_void);
            v8::Isolate_SetCaptureStackTraceForUncaughtExceptions_Detailed(raw, 1, 1024);
        }

        Isolate(raw)
    }

    pub unsafe fn from_raw(raw: v8::IsolatePtr) -> Isolate {
        let result = Isolate(raw);
        result.get_data().count += 1;
        result
    }

    pub fn as_raw(&self) -> v8::IsolatePtr {
        self.0
    }

    pub fn current_context(&self) -> Option<context::Context> {
        unsafe {
            let raw = v8::Isolate_GetCurrentContext(self.as_raw()).as_mut();
            raw.map(|r| context::Context::from_raw(self, r))
        }
    }

    unsafe fn get_data_ptr(&self) -> *mut Data {
        v8::Isolate_GetData(self.0, DATA_PTR_SLOT) as *mut Data
    }

    unsafe fn get_data(&self) -> &mut Data {
        self.get_data_ptr().as_mut().unwrap()
    }
}

impl Clone for Isolate {
    fn clone(&self) -> Isolate {
        unsafe {
            self.get_data().count += 1;
        }
        Isolate(self.0)
    }
}

impl Drop for Isolate {
    fn drop(&mut self) {
        unsafe {
            let ref mut count = self.get_data().count;
            *count -= 1;

            if *count == 0 {
                drop(Box::from_raw(self.get_data_ptr()));
                v8::Isolate_Dispose(self.0);
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
