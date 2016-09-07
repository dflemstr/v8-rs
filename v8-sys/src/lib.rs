#[cfg(test)]
#[macro_use]
extern crate lazy_static;

include!(concat!(env!("OUT_DIR"), "/ffi.rs"));

pub use ffi::*;

#[cfg(test)]
mod tests {
    use super::*;

    use std::ffi;
    use std::mem;
    use std::os;
    use std::thread;
    use std::time;

    lazy_static! {
        static ref START_TIME: time::Instant = {
            time::Instant::now()
        };
    }

    #[test]
    fn e2e() {
        unsafe {
            let platform_functions = PlatformFunctions {
                Destroy: Some(destroy_platform),
                NumberOfAvailableBackgroundThreads: Some(number_of_available_background_threads),
                CallOnBackgroundThread: Some(call_on_background_thread),
                CallOnForegroundThread: Some(call_on_foreground_thread),
                CallDelayedOnForegroundThread: Some(call_delayed_on_foreground_thread),
                CallIdleOnForegroundThread: Some(call_idle_on_foreground_thread),
                IdleTasksEnabled: Some(idle_tasks_enabled),
                MonotonicallyIncreasingTime: Some(monotonically_increasing_time),
            };

            let allocator_functions = AllocatorFunctions {
                Allocate: Some(allocate),
                AllocateUninitialized: Some(allocate_uninitialized),
                Free: Some(free),
            };

            V8_InitializeICU();
            let platform = Platform_Create(platform_functions);
            assert!(!platform.is_null());
            V8_InitializePlatform(platform);
            V8_Initialize();

            let allocator = ArrayBuffer_Allocator_Create(allocator_functions);
            assert!(!allocator.is_null());
            let isolate = Isolate_New(allocator);
            assert!(!isolate.is_null());
            let context = Context_New(isolate);
            assert!(!context.is_null());

            Context_Enter(isolate, context);

            let source_str = "'Hello' + ', World!'";
            let source_cstr = ffi::CString::new(source_str).unwrap();
            let source = String_NewFromUtf8_Normal(isolate, source_cstr.as_ptr() as *const i8, -1);
            assert!(!source.is_null());
            let script = Script_Compile(isolate, context, source);
            assert!(!script.is_null());

            let result = Script_Run(isolate, script, context);
            assert!(!result.is_null());
            assert!(Value_IsString(isolate, result) == 1);

            let result_string = Value_ToString(isolate, result, context);
            assert!(!result_string.is_null());

            let len = String_Utf8Length(isolate, result_string) as usize;
            let mut buf = vec![0u8; len];
            String_WriteUtf8(isolate,
                             result_string,
                             buf.as_mut_ptr() as *mut i8,
                             len as i32);
            let rust_string = ::std::string::String::from_utf8_unchecked(buf);
            assert_eq!("Hello, World!", rust_string);

            Context_Exit(isolate, context);

            String_Destroy(isolate, result_string);
            Value_Destroy(isolate, result);
            Script_Destroy(isolate, script);
            String_Destroy(isolate, source);
            Context_Destroy(isolate, context);
            Isolate_Dispose(isolate);
            ArrayBuffer_Allocator_Destroy(allocator);
            V8_Dispose();
            V8_ShutdownPlatform();
            Platform_Destroy(platform);
        }
    }

    struct TaskHolder(*mut Task);

    unsafe impl Send for TaskHolder {}

    extern "C" fn allocate(length: usize) -> *mut os::raw::c_void {
        let mut data = Vec::with_capacity(length);
        data.resize(length, 0u8);
        let ptr = data.as_mut_ptr();
        mem::forget(data);

        ptr as *mut os::raw::c_void
    }

    extern "C" fn allocate_uninitialized(length: usize) -> *mut os::raw::c_void {
        let mut data = Vec::with_capacity(length);

        unsafe {
            data.set_len(length);
        }

        let ptr = data.as_mut_ptr();
        mem::forget(data);

        ptr as *mut os::raw::c_void
    }

    unsafe extern "C" fn free(data: *mut os::raw::c_void, length: usize) {
        drop(Vec::from_raw_parts(data, length, length));
    }

    extern "C" fn destroy_platform() {
        // No-op
    }

    extern "C" fn number_of_available_background_threads() -> usize {
        0 // TODO: do something smart
    }

    unsafe extern "C" fn call_on_background_thread(task: *mut Task,
                                                   _expected_runtime: ExpectedRuntime) {
        let holder = TaskHolder(task);
        thread::spawn(move || {
            Task_Run(holder.0);
        });
    }

    unsafe extern "C" fn call_on_foreground_thread(_isolate: *mut Isolate, task: *mut Task) {
        let holder = TaskHolder(task);
        thread::spawn(move || {
            Task_Run(holder.0);
        });
    }

    unsafe extern "C" fn call_delayed_on_foreground_thread(_isolate: *mut Isolate,
                                                           task: *mut Task,
                                                           delay_in_seconds: f64) {
        let holder = TaskHolder(task);

        thread::spawn(move || {
            thread::sleep(time::Duration::new(delay_in_seconds as u64,
                                              (delay_in_seconds.fract() * 1e9) as u32));
            Task_Run(holder.0);
        });
    }

    unsafe extern "C" fn call_idle_on_foreground_thread(_isolate: *mut Isolate,
                                                        _task: *mut IdleTask) {
    }

    unsafe extern "C" fn idle_tasks_enabled(_isolate: *mut Isolate) -> u8 {
        0
    }

    extern "C" fn monotonically_increasing_time() -> f64 {
        let start = *START_TIME;
        let d = time::Instant::now().duration_since(start);
        (d.as_secs() as f64) + (d.subsec_nanos() as f64 * 1e-9)
    }
}
