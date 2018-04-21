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

use std::cell;
use std::collections;
use std::fmt;
use std::mem;
use std::os;
use std::ptr;
use std::rc;
use std::sync;
use std::time;
use v8_sys;
use allocator;
use platform;
use priority_queue;

static INITIALIZE: sync::Once = sync::ONCE_INIT;

/// Isolate represents an isolated instance of the V8 engine.
///
/// V8 isolates have completely separate states.  Objects from one isolate must not be used in other
/// isolates.  The embedder can create multiple isolates and use them in parallel in multiple
/// threads.  An isolate can be entered by at most one thread at any given time.  The
/// Locker/Unlocker API must be used to synchronize.
pub struct Isolate(ptr::Shared<v8_sys::Isolate>);

/// A builder for isolates.  Can be converted into an isolate with the `build` method.
pub struct Builder {
    supports_idle_tasks: bool,
}

#[must_use]
pub struct Scope<'i>(&'i mut Isolate);

#[derive(Debug)]
struct Data {
    count: cell::Cell<usize>,
    _allocator: allocator::Allocator,
    task_queue: rc::Rc<cell::RefCell<priority_queue::PriorityQueue<platform::Task, time::Instant>>>,
    idle_task_queue: Option<rc::Rc<cell::RefCell<collections::VecDeque<platform::IdleTask>>>>,
}

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

    /// Creates a data from a raw pointer.
    ///
    /// This isolate must at some point have been created by `Isolate::new`, since this library
    /// expects isolates to be configured a certain way and contain embedder information.
    pub unsafe fn from_ptr(raw: *mut v8_sys::Isolate) -> Isolate {
        let mut result = Isolate(ptr::Shared::new(raw).unwrap());
        *result.data_mut().count.get_mut() += 1;
        result
    }

    /// Returns the underlying raw pointer behind this isolate.
    pub fn as_ptr(&self) -> *mut v8_sys::Isolate {
        self.0.as_ptr()
    }

    pub fn scope(&mut self) -> Scope {
        unsafe { self.0.as_mut().Enter() };
        Scope(self)
    }

    /*
    /// Returns the context bound to the current thread for this isolate.
    ///
    /// A context will be bound by for example `Context::make_current`, or while inside of a
    /// function callback.
    pub fn current_context(&self) -> Option<context::Context> {
        unsafe {
            let raw = self.isolate(self.as_raw()).as_mut();
            raw.map(|r| context::Context::from_raw(self, r))
        }
    }
    */

    /// Runs all enqueued tasks until there are no more tasks available.
    pub fn run_enqueued_tasks(&self) {
        while self.run_enqueued_task() {}
    }

    /// Runs a single enqueued task, if there is one.  Returns `true` if a task was executed, and
    /// `false` if there are no pending tasks to run.
    pub fn run_enqueued_task(&self) -> bool {
        let data = self.data();
        let now = time::Instant::now();

        if data.task_queue
            .borrow()
            .peek()
            .map(|(_, p)| *p > now)
            .unwrap_or(false)
        {
            let task = data.task_queue.borrow_mut().pop().unwrap().0;
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
        let data = self.data();

        if let Some(idle_task) = data.idle_task_queue
            .as_ref()
            .map(|q| q.borrow_mut().pop_front())
            .unwrap_or(None)
        {
            idle_task.run(deadline);
            true
        } else {
            false
        }
    }

    /// Enqueues the specified task to run as soon as possible.
    pub fn enqueue_task(&self, task: platform::Task) {
        self.data().task_queue.borrow_mut().push(
            task,
            time::Instant::now(),
        );
    }

    /// Enqueues the specified task to run after the specified delay has passed.
    pub fn enqueue_delayed_task(&self, delay: time::Duration, task: platform::Task) {
        self.data().task_queue.borrow_mut().push(
            task,
            time::Instant::now() + delay,
        );
    }

    /// Enqueues a task to be run when the isolate is considered to be "idle."
    pub fn enqueue_idle_task(&self, idle_task: platform::IdleTask) {
        self.data()
            .idle_task_queue
            .as_ref()
            .unwrap()
            .borrow_mut()
            .push_back(idle_task);
    }

    /// Whether this isolate was configured to support idle tasks.
    pub fn supports_idle_tasks(&self) -> bool {
        self.data().idle_task_queue.is_some()
    }

    fn data_ptr(&self) -> *mut Data {
        unsafe { (*self.0.as_ptr()).GetData(DATA_PTR_SLOT) as *mut Data }
    }

    fn data(&self) -> &Data {
        unsafe { self.data_ptr().as_ref().unwrap() }
    }

    fn data_mut(&mut self) -> &mut Data {
        unsafe { self.data_ptr().as_mut().unwrap() }
    }
}

impl Clone for Isolate {
    fn clone(&self) -> Isolate {
        let data = self.data();
        let new_count = data.count.get() + 1;
        data.count.set(new_count);
        Isolate(self.0)
    }
}

impl fmt::Debug for Isolate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Isolate({:?}, {:?})",
            unsafe { self.0.as_ref() },
            self.data()
        )
    }
}

impl Drop for Isolate {
    fn drop(&mut self) {
        let new_count = {
            let data = self.data();
            let new_count = data.count.get() - 1;
            data.count.set(new_count);
            new_count
        };

        unsafe {
            if new_count == 0 {
                drop(Box::from_raw(self.data_ptr()));
                self.0.as_mut().Dispose();
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

        let mut raw = unsafe {
            let mut params: v8_sys::Isolate_CreateParams = mem::zeroed();
            params.allow_atomics_wait = true;
            params.array_buffer_allocator = allocator.as_ptr();
            ptr::Shared::new(v8_sys::Isolate::New(&params)).expect("Could not create Isolate")
        };

        unsafe {
            assert!(v8_sys::Isolate::GetNumberOfDataSlots() > 0);
        }

        let idle_task_queue = if self.supports_idle_tasks {
            Some(rc::Rc::new(
                cell::RefCell::new(collections::VecDeque::new()),
            ))
        } else {
            None
        };

        let data = Data {
            count: cell::Cell::new(1),
            _allocator: allocator,
            task_queue: rc::Rc::new(cell::RefCell::new(priority_queue::PriorityQueue::new())),
            idle_task_queue: idle_task_queue,
        };
        let data_ptr: *mut Data = Box::into_raw(Box::new(data));

        unsafe {
            raw.as_mut().SetData(
                DATA_PTR_SLOT,
                data_ptr as *mut os::raw::c_void,
            );
            raw.as_mut().SetCaptureStackTraceForUncaughtExceptions(
                true,
                1024,
                v8_sys::StackTrace_StackTraceOptions_kDetailed,
            );
        }

        Isolate(raw)
    }
}

impl<'i> Scope<'i> {
    pub fn isolate(&self) -> &Isolate {
        &self.0
    }

    pub fn isolate_mut(&mut self) -> &mut Isolate {
        &mut self.0
    }
}

impl<'i> Drop for Scope<'i> {
    fn drop(&mut self) {
        unsafe { (self.0).0.as_mut().Exit() }
    }
}

fn ensure_initialized() {
    INITIALIZE.call_once(|| {
        unsafe {
            v8_sys::V8_InitializeICU(ptr::null());

            let platform = platform::Platform::new();
            v8_sys::V8_InitializePlatform(platform.as_ptr());
            // TODO: implement some form of cleanup
            mem::forget(platform);

            v8_sys::V8_Initialize();
        }
    });
}
