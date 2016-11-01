use v8_sys as v8;
use std::thread;
use std::time;
use num_cpus;
use isolate;

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

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Task(v8::TaskPtr);

unsafe impl Send for Task {}

#[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct IdleTask(v8::IdleTaskPtr);

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

impl Task {
    pub fn run(&self) {
        unsafe {
            v8::Task_Run(self.0);
        }
    }
}

impl Drop for Task {
    fn drop(&mut self) {
        unsafe {
            v8::Task_Destroy(self.0);
        }
    }
}

impl IdleTask {
    pub fn run(&self, deadline: time::Duration) {
        unsafe {
            v8::IdleTask_Run(self.0, duration_to_seconds(deadline));
        }
    }
}

impl Drop for IdleTask {
    fn drop(&mut self) {
        unsafe {
            v8::IdleTask_Destroy(self.0);
        }
    }
}

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
    num_cpus::get()
}

extern "C" fn call_on_background_thread(task: v8::TaskPtr,
                                        _expected_runtime: v8::ExpectedRuntime) {
    let task = Task(task);
    thread::spawn(move || {
        unsafe {
            v8::Task_Run(task.0);
        }
    });
}

extern "C" fn call_on_foreground_thread(isolate: v8::IsolatePtr, task: v8::TaskPtr) {
    let task = Task(task);
    let isolate = unsafe { isolate::Isolate::from_raw(isolate) };

    isolate.enqueue_task(task);
}

extern "C" fn call_delayed_on_foreground_thread(isolate: v8::IsolatePtr,
                                                task: v8::TaskPtr,
                                                delay_in_seconds: f64) {
    let task = Task(task);
    let isolate = unsafe { isolate::Isolate::from_raw(isolate) };
    let duration = duration_from_seconds(delay_in_seconds);

    isolate.enqueue_delayed_task(duration, task);
}

extern "C" fn call_idle_on_foreground_thread(isolate: v8::IsolatePtr, idle_task: v8::IdleTaskPtr) {
    let idle_task = IdleTask(idle_task);
    let isolate = unsafe { isolate::Isolate::from_raw(isolate) };

    isolate.enqueue_idle_task(idle_task);
}

extern "C" fn idle_tasks_enabled(isolate: v8::IsolatePtr) -> u8 {
    let isolate = unsafe { isolate::Isolate::from_raw(isolate) };

    if isolate.supports_idle_tasks() { 1 } else { 0 }
}

extern "C" fn monotonically_increasing_time() -> f64 {
    let start = *START_TIME;
    let d = time::Instant::now().duration_since(start);
    duration_to_seconds(d)
}

fn duration_to_seconds(duration: time::Duration) -> f64 {
    (duration.subsec_nanos() as f64).mul_add(1e-9, duration.as_secs() as f64)
}

fn duration_from_seconds(seconds: f64) -> time::Duration {
    time::Duration::new(seconds as u64, (seconds.fract() * 1e9) as u32)
}
