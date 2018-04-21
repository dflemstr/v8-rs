use v8_sys;
use std::fmt;
use std::hash;
use std::os;
use std::ptr;
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
pub struct Platform(ptr::Unique<v8_sys::Platform>);

pub struct Task(ptr::Unique<v8_sys::Task>);

unsafe impl Send for Task {}

pub struct IdleTask(ptr::Unique<v8_sys::IdleTask>);

impl Platform {
    pub fn new() -> Platform {
        let raw = unsafe {
            ptr::Unique::new(v8_sys::impls::CreatePlatform(
                PLATFORM_FUNCTIONS,
                ptr::null_mut(),
            ))
        }.expect("could not create Platform");

        Platform(raw)
    }

    pub fn as_ptr(&self) -> *mut v8_sys::Platform {
        self.0.as_ptr()
    }
}

impl fmt::Debug for Platform {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Platform({:?})", unsafe { self.0.as_ref() })
    }
}

impl Drop for Platform {
    fn drop(&mut self) {
        unsafe {
            v8_sys::Platform_Platform_destructor(self.0.as_ptr());
        }
    }
}

impl Task {
    pub fn run(&self) {
        unsafe {
            v8_sys::Task_Run(self.0.as_ptr() as *mut os::raw::c_void);
        }
    }

    pub fn as_ptr(&self) -> *mut v8_sys::Task {
        self.0.as_ptr()
    }

    pub unsafe fn from_ptr(ptr: *mut v8_sys::Task) -> Task {
        Task(ptr::Unique::new(ptr).unwrap())
    }
}

impl fmt::Debug for Task {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Task({:?})", unsafe { self.0.as_ref() })
    }
}

impl PartialEq for Task {
    fn eq(&self, other: &Task) -> bool {
        self.as_ptr() == other.as_ptr()
    }
}

impl Eq for Task {}

impl hash::Hash for Task {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        self.as_ptr().hash(state)
    }
}

impl Drop for Task {
    fn drop(&mut self) {
        unsafe {
            v8_sys::Task_Task_destructor(self.0.as_ptr());
        }
    }
}

impl IdleTask {
    pub fn run(&self, deadline: time::Duration) {
        unsafe {
            v8_sys::IdleTask_Run(
                self.0.as_ptr() as *mut os::raw::c_void,
                duration_to_seconds(deadline),
            );
        }
    }

    pub fn as_ptr(&self) -> *mut v8_sys::IdleTask {
        self.0.as_ptr()
    }

    pub unsafe fn from_ptr(ptr: *mut v8_sys::IdleTask) -> IdleTask {
        IdleTask(ptr::Unique::new(ptr).unwrap())
    }
}

impl fmt::Debug for IdleTask {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "IdleTask({:?})", unsafe { self.0.as_ref() })
    }
}

impl Drop for IdleTask {
    fn drop(&mut self) {
        unsafe {
            v8_sys::IdleTask_IdleTask_destructor(self.0.as_ptr());
        }
    }
}

const PLATFORM_FUNCTIONS: v8_sys::impls::PlatformFunctions = v8_sys::impls::PlatformFunctions {
    Destroy: Some(destroy_platform),
    NumberOfAvailableBackgroundThreads: Some(number_of_available_background_threads),
    CallOnBackgroundThread: Some(call_on_background_thread),
    CallOnForegroundThread: Some(call_on_foreground_thread),
    CallDelayedOnForegroundThread: Some(call_delayed_on_foreground_thread),
    CallIdleOnForegroundThread: Some(call_idle_on_foreground_thread),
    IdleTasksEnabled: Some(idle_tasks_enabled),
    MonotonicallyIncreasingTime: Some(monotonically_increasing_time),
};

extern "C" fn destroy_platform(_this: *mut os::raw::c_void) {
    // No-op
}

extern "C" fn number_of_available_background_threads(_this: *mut os::raw::c_void) -> usize {
    num_cpus::get()
}

extern "C" fn call_on_background_thread(
    _this: *mut os::raw::c_void,
    task: *mut v8_sys::Task,
    _expected_runtime: v8_sys::Platform_ExpectedRuntime,
) {
    let task = unsafe { Task::from_ptr(task) };
    thread::spawn(move || unsafe {
        v8_sys::Task_Run(task.0.as_ptr() as *mut os::raw::c_void);
    });
}

extern "C" fn call_on_foreground_thread(
    _this: *mut os::raw::c_void,
    isolate: *mut v8_sys::Isolate,
    task: *mut v8_sys::Task,
) {
    let task = unsafe { Task::from_ptr(task) };
    let isolate = unsafe { isolate::Isolate::from_ptr(isolate) };

    isolate.enqueue_task(task);
}

extern "C" fn call_delayed_on_foreground_thread(
    _this: *mut os::raw::c_void,
    isolate: *mut v8_sys::Isolate,
    task: *mut v8_sys::Task,
    delay_in_seconds: f64,
) {
    let task = unsafe { Task::from_ptr(task) };
    let isolate = unsafe { isolate::Isolate::from_ptr(isolate) };
    let duration = duration_from_seconds(delay_in_seconds);

    isolate.enqueue_delayed_task(duration, task);
}

extern "C" fn call_idle_on_foreground_thread(
    _this: *mut os::raw::c_void,
    isolate: *mut v8_sys::Isolate,
    idle_task: *mut v8_sys::IdleTask,
) {
    let idle_task = unsafe { IdleTask::from_ptr(idle_task) };
    let isolate = unsafe { isolate::Isolate::from_ptr(isolate) };

    isolate.enqueue_idle_task(idle_task);
}

extern "C" fn idle_tasks_enabled(
    _this: *mut os::raw::c_void,
    isolate: *mut v8_sys::Isolate,
) -> bool {
    let isolate = unsafe { isolate::Isolate::from_ptr(isolate) };

    isolate.supports_idle_tasks()
}

extern "C" fn monotonically_increasing_time(_this: *mut os::raw::c_void) -> f64 {
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
