use v8_sys as v8;
use std::thread;
use std::time;

lazy_static! {
    static ref START_TIME: time::Instant = {
        time::Instant::now()
    };
}

/// A simple platform implementation that uses global OS threads for
/// scheduling.
// TODO: make this use some kind of main loop/work stealing queue
// instead.
#[derive(Debug)]
pub struct Platform(v8::PlatformPtr);

impl Platform {
    pub fn new() -> Platform {
        let raw = unsafe { v8::Platform_Create(PLATFORM_FUNCTIONS) };

        if raw.is_null() {
            panic!("Could not create Platform")
        }

        Platform(raw)
    }

    pub fn as_raw(&self) -> v8::PlatformPtr {
        self.0
    }
}

impl Drop for Platform {
    fn drop(&mut self) {
        unsafe {
            v8::Platform_Destroy(self.0);
        }
    }
}

#[derive(Debug)]
struct TaskHolder(v8::TaskPtr);

unsafe impl Send for TaskHolder {}

const PLATFORM_FUNCTIONS: v8::PlatformFunctions = v8::PlatformFunctions {
    Destroy: Some(destroy_platform),
    NumberOfAvailableBackgroundThreads: Some(number_of_available_background_threads),
    CallOnBackgroundThread: Some(call_on_background_thread),
    CallOnForegroundThread: Some(call_on_foreground_thread),
    CallDelayedOnForegroundThread: Some(call_delayed_on_foreground_thread),
    CallIdleOnForegroundThread: Some(call_idle_on_foreground_thread),
    IdleTasksEnabled: Some(idle_tasks_enabled),
    MonotonicallyIncreasingTime: Some(monotonically_increasing_time),
};

extern "C" fn destroy_platform() {
    // No-op
}

extern "C" fn number_of_available_background_threads() -> usize {
    0 // TODO: do something smart
}

extern "C" fn call_on_background_thread(task: v8::TaskPtr,
                                        _expected_runtime: v8::ExpectedRuntime) {
    let holder = TaskHolder(task);
    thread::spawn(move || {
        unsafe {
            v8::Task_Run(holder.0);
        }
    });
}

extern "C" fn call_on_foreground_thread(_isolate: v8::IsolatePtr, task: v8::TaskPtr) {
    let holder = TaskHolder(task);
    // TODO: this should actually be done on some main loop
    thread::spawn(move || {
        unsafe {
            v8::Task_Run(holder.0);
        }
    });
}

extern "C" fn call_delayed_on_foreground_thread(_isolate: v8::IsolatePtr,
                                                task: v8::TaskPtr,
                                                delay_in_seconds: f64) {
    let holder = TaskHolder(task);

    // TODO: this should actually be done on some main loop
    thread::spawn(move || {
        thread::sleep(time::Duration::new(delay_in_seconds as u64,
                                          (delay_in_seconds.fract() * 1e9) as u32));
        unsafe {
            v8::Task_Run(holder.0);
        }
    });
}

extern "C" fn call_idle_on_foreground_thread(_isolate: v8::IsolatePtr, _task: v8::IdleTaskPtr) {
    unreachable!()
}

extern "C" fn idle_tasks_enabled(_isolate: v8::IsolatePtr) -> u8 {
    0
}

extern "C" fn monotonically_increasing_time() -> f64 {
    let start = *START_TIME;
    let d = time::Instant::now().duration_since(start);
    (d.as_secs() as f64) + (d.subsec_nanos() as f64 * 1e-9)
}
