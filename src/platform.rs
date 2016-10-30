use v8_sys as v8;
use isolate;
use std::marker;
use std::mem;
use std::os;
use std::thread;
use std::time;

pub enum ExpectedRuntime {
    ShortRunningTask,
    LongRunningTask,
}

pub trait Platform {
    fn number_of_available_background_threads(&self) -> usize;

    fn call_on_background_thread(&self, task: Task, expected_runtime: ExpectedRuntime);

    fn call_on_foreground_thread(&self, isolate: &isolate::Isolate, task: Task);

    fn call_delayed_on_foreground_thread(&self,
                                         isolate: &isolate::Isolate,
                                         task: Task,
                                         delay: time::Duration);

    fn call_idle_on_foreground_thread(&self, isolate: &isolate::Isolate, task: IdleTask);

    fn idle_tasks_enabled(&self) -> bool;

    fn monotonically_increasing_time(&self) -> time::Instant;
}

/// A simple platform implementation that uses global OS threads for
/// scheduling.
pub struct DefaultPlatform;

#[derive(Debug)]
pub struct PlatformInstance<P>(v8::PlatformPtr, marker::PhantomData<P>);

pub struct Task(v8::TaskPtr);

pub struct IdleTask(v8::IdleTaskPtr);

impl<P> PlatformInstance<P>
    where P: Platform
{
    pub fn new(platform: Box<P>) -> PlatformInstance<P> {
        let raw =
            unsafe { v8::Platform_Create(mem::transmute(platform), platform_functions::<P>()) };

        if raw.is_null() {
            panic!("Could not create Platform")
        }

        PlatformInstance(raw, marker::PhantomData)
    }

    pub fn as_raw(&self) -> v8::PlatformPtr {
        self.0
    }
}

impl<P> Drop for PlatformInstance<P> {
    fn drop(&mut self) {
        unsafe {
            v8::Platform_Destroy(self.0);
        }
    }
}

fn platform_functions<P>() -> v8::PlatformFunctions
    where P: Platform
{
    v8::PlatformFunctions {
        Destroy: Some(destroy_platform::<P>),
        NumberOfAvailableBackgroundThreads: Some(number_of_available_background_threads::<P>),
        CallOnBackgroundThread: Some(call_on_background_thread::<P>),
        CallOnForegroundThread: Some(call_on_foreground_thread::<P>),
        CallDelayedOnForegroundThread: Some(call_delayed_on_foreground_thread::<P>),
        CallIdleOnForegroundThread: Some(call_idle_on_foreground_thread::<P>),
        IdleTasksEnabled: Some(idle_tasks_enabled::<P>),
        MonotonicallyIncreasingTime: Some(monotonically_increasing_time::<P>),
    }
}

unsafe extern "C" fn destroy_platform<P>(this: *mut os::raw::c_void) {
    drop(Box::from_raw(this as *mut P))
}

unsafe extern "C" fn number_of_available_background_threads<P>(this: *mut os::raw::c_void) -> usize
    where P: Platform
{
    coerce_ref::<P>(this).number_of_available_background_threads()
}

unsafe extern "C" fn call_on_background_thread<P>(this: *mut os::raw::c_void,
                                                  task: v8::TaskPtr,
                                                  expected_runtime: v8::ExpectedRuntime)
    where P: Platform
{
    let expected_runtime = match expected_runtime {
        v8::ExpectedRuntime::SHORT_RUNNING_TASK => ExpectedRuntime::ShortRunningTask,
        v8::ExpectedRuntime::LONG_RUNNING_TASK => ExpectedRuntime::LongRunningTask,
    };

    coerce_ref::<P>(this).call_on_background_thread(Task(task), expected_runtime)
}

unsafe extern "C" fn call_on_foreground_thread<P>(this: *mut os::raw::c_void,
                                                  isolate: v8::IsolatePtr,
                                                  task: v8::TaskPtr)
    where P: Platform
{
    coerce_ref::<P>(this).call_on_foreground_thread(&isolate::Isolate::from_raw(isolate), Task(task))
}

unsafe extern "C" fn call_delayed_on_foreground_thread<P>(this: *mut os::raw::c_void,
                                                          isolate: v8::IsolatePtr,
                                                          task: v8::TaskPtr,
                                                          delay_in_seconds: f64)
    where P: Platform
{
    let delay = time::Duration::new(delay_in_seconds as u64,
                                    (delay_in_seconds.fract() * 1e9) as u32);

    coerce_ref::<P>(this).call_delayed_on_foreground_thread(&isolate::Isolate::from_raw(isolate), Task(task), delay)
}

unsafe extern "C" fn call_idle_on_foreground_thread<P>(this: *mut os::raw::c_void,
                                                       isolate: v8::IsolatePtr,
                                                       task: v8::IdleTaskPtr)
    where P: Platform
{
    coerce_ref::<P>(this).call_idle_on_foreground_thread(&isolate::Isolate::from_raw(isolate), IdleTask(task))
}

unsafe extern "C" fn idle_tasks_enabled<P>(this: *mut os::raw::c_void,
                                           isolate: v8::IsolatePtr)
                                           -> u8
    where P: Platform
{
    if coerce_ref::<P>(this).idle_tasks_enabled() {
        1
    } else {
        0
    }
}

unsafe extern "C" fn monotonically_increasing_time<P>(this: *mut os::raw::c_void) -> f64
    where P: Platform
{
    let instant = coerce_ref::<P>(this).monotonically_increasing_time();
}

fn coerce_ref<'a, A>(ptr: *mut os::raw::c_void) -> &'a A {
    (ptr as *mut A).as_ref().unwrap()
}
