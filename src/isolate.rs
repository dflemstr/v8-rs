//! Heap and execution isolation.
//!
//! # Usage
//!
//! Construct a new isolate with default settings by doing `Isolate::new()`.  You can customize the
//! isolate settings by using `Isolate::builder()`.
//!
//! # Foreground tasks
//!
//! Javascript can produce "deferred" or "time-outed" tasks that need to run on the main thread.
//! Additionally, V8 has a bunch of internal tasks it wants to perform regularly (for example GC).
//! The user should therefore call `isolate.run_enqueued_tasks()` regularly to allow these tasks to
//! run.
//!
//! # Background tasks
//!
//! Javascript and V8 can trigger various background tasks to run.  These will be run as simple
//! background OS threads, while trying to keep the number of running background tasks less than the
//! number of available CPUs.
//!
//! # Idle tasks
//!
//! V8 can perform various maintenance tasks if the application has nothing better to do.  If the
//! user wants to allow this to happen, an isolate should be constructed with
//! `Isolate::builder().supports_idle_tasks(true).build()`.  The user should then regularly call
//! `isolate.run_idle_tasks(deadline)` to run any pending idle tasks.

use std::cmp;
use std::collections;
use std::mem;
use std::os;
use std::sync;
use std::time;
use v8_sys as v8;
use allocator;
use context;
use platform;

static INITIALIZE: sync::Once = sync::ONCE_INIT;

/// Isolate represents an isolated instance of the V8 engine.
///
/// V8 isolates have completely separate states.  Objects from one isolate must not be used in other
/// isolates.  The embedder can create multiple isolates and use them in parallel in multiple
/// threads.  An isolate can be entered by at most one thread at any given time.  The
/// Locker/Unlocker API must be used to synchronize.
#[derive(Debug)]
pub struct Isolate(v8::IsolatePtr);

/// A builder for isolates.  Can be converted into an isolate with the `build` method.
pub struct Builder {
    supports_idle_tasks: bool,
}

#[derive(Debug)]
struct Data {
    count: usize,
    _allocator: allocator::Allocator,
    task_queue: collections::BinaryHeap<ScheduledTask>,
    idle_task_queue: Option<collections::VecDeque<platform::IdleTask>>,
}

#[derive(Debug, Eq, PartialEq)]
struct ScheduledTask(time::Instant, platform::Task);

const DATA_PTR_SLOT: u32 = 0;

impl Isolate {
    /// Creates a new isolate.
    pub fn new() -> Isolate {
        Isolate::builder().build()
    }

    /// Creates a new isolate builder.
    pub fn builder() -> Builder {
        Builder { supports_idle_tasks: false }
    }

    /// Creates a data from a set of raw pointers.
    ///
    /// This isolate must at some point have been created by `Isolate::new`, since this library
    /// expects isolates to be configured a certain way and contain embedder information.
    pub unsafe fn from_raw(raw: v8::IsolatePtr) -> Isolate {
        let result = Isolate(raw);
        result.get_data().count += 1;
        result
    }

    /// Returns the underlying raw pointer behind this isolate.
    pub fn as_raw(&self) -> v8::IsolatePtr {
        self.0
    }

    /// Returns the context bound to the current thread for this isolate.
    ///
    /// A context will be bound by for example `Context::make_current`, or while inside of a
    /// function callback.
    pub fn current_context(&self) -> Option<context::Context> {
        unsafe {
            let raw = v8::Isolate_GetCurrentContext(self.as_raw()).as_mut();
            raw.map(|r| context::Context::from_raw(self, r))
        }
    }

    /// Runs all enqueued tasks until there are no more tasks available.
    pub fn run_enqueued_tasks(&self) {
        while self.run_enqueued_task() {}
    }

    /// Runs a single enqueued task, if there is one.  Returns `true` if a task was executed, and
    /// `false` if there are no pending tasks to run.
    pub fn run_enqueued_task(&self) -> bool {
        let data = unsafe { self.get_data() };
        let now = time::Instant::now();

        if data.task_queue.peek().map(|t| t.0 > now).unwrap_or(false) {
            let task = data.task_queue.pop().unwrap().1;
            task.run();
            true
        } else {
            false
        }
    }

    /// Runs as many idle tasks as possible within the specified deadline.  It is not guaranteed
    /// that the execution of the tasks will take less time than the specified deadline.
    pub fn run_idle_tasks(&self, deadline: time::Duration) {
        let deadline = time::Instant::now() + deadline;

        loop {
            let now = time::Instant::now();

            if now > deadline {
                break;
            }

            self.run_idle_task(deadline - now);
        }
    }

    /// Runs a single idle task within the specified deadline.  It is not guaranteed that the
    /// execution of the task will take less time than the specified deadline.  Returns `true` if a
    /// task was executed, and `false` if there are no pending tasks to run.
    pub fn run_idle_task(&self, deadline: time::Duration) -> bool {
        let data = unsafe { self.get_data() };

        if let Some(idle_task) = data.idle_task_queue
            .as_mut()
            .map(|q| q.pop_front())
            .unwrap_or(None) {
            idle_task.run(deadline);
            true
        } else {
            false
        }
    }

    /// Enqueues the specified task to run as soon as possible.
    pub fn enqueue_task(&self, task: platform::Task) {
        let scheduled_task = ScheduledTask(time::Instant::now(), task);
        unsafe { self.get_data() }.task_queue.push(scheduled_task);
    }

    /// Enqueues the specified task to run after the specified delay has passed.
    pub fn enqueue_delayed_task(&self, delay: time::Duration, task: platform::Task) {
        let scheduled_task = ScheduledTask(time::Instant::now() + delay, task);
        unsafe { self.get_data() }.task_queue.push(scheduled_task);
    }

    /// Enqueues a task to be run when the isolate is considered to be "idle."
    pub fn enqueue_idle_task(&self, idle_task: platform::IdleTask) {
        unsafe { self.get_data() }.idle_task_queue.as_mut().unwrap().push_back(idle_task);
    }

    /// Whether this isolate was configured to support idle tasks.
    pub fn supports_idle_tasks(&self) -> bool {
        unsafe { self.get_data() }.idle_task_queue.is_some()
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

impl Builder {
    /// Whether the isolate should support idle tasks; i.e. whether the user will call
    /// `run_idle_tasks` regularly.
    pub fn supports_idle_tasks(mut self, value: bool) -> Builder {
        self.supports_idle_tasks = value;
        self
    }

    /// Constructs a new `Isolate` based on this builder.
    pub fn build(self) -> Isolate {
        ensure_initialized();

        let allocator = allocator::Allocator::new();

        let raw = unsafe { v8::Isolate_New(allocator.as_raw()) };
        if raw.is_null() {
            panic!("Could not create Isolate");
        }

        unsafe {
            assert!(v8::Isolate_GetNumberOfDataSlots(raw) > 0);
        }

        let idle_task_queue = if self.supports_idle_tasks {
            Some(collections::VecDeque::new())
        } else {
            None
        };

        let data = Data {
            count: 1,
            _allocator: allocator,
            task_queue: collections::BinaryHeap::new(),
            idle_task_queue: idle_task_queue,
        };
        let data_ptr: *mut Data = Box::into_raw(Box::new(data));

        unsafe {
            v8::Isolate_SetData(raw, DATA_PTR_SLOT, data_ptr as *mut os::raw::c_void);
            v8::Isolate_SetCaptureStackTraceForUncaughtExceptions_Detailed(raw, 1, 1024);
        }

        Isolate(raw)
    }
}

impl PartialOrd for ScheduledTask {
    fn partial_cmp(&self, other: &ScheduledTask) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ScheduledTask {
    fn cmp(&self, other: &ScheduledTask) -> cmp::Ordering {
        self.0.cmp(&other.0).reverse()
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
