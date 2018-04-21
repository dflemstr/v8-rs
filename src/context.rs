//! Execution contexts and sandboxing.
use v8_sys;
use std::ptr;
use isolate;
use handle;
use value;

/// A sandboxed execution context with its own set of built-in objects and functions.
#[derive(Debug)]
pub struct Context(v8_sys::Context);

/// A guard that keeps a context bound while it is in scope.
#[must_use]
pub struct Scope<'c>(&'c mut Context);

impl Context {
    /// Creates a new context and returns a handle to the newly allocated context.
    pub fn new<'i, 's>(
        scope: &'s handle::Scope,
        isolate: &'i isolate::Isolate,
    ) -> handle::Local<'i, 's, Context> {
        unsafe {
            handle::Local::new(v8_sys::Context::New(
                isolate.as_ptr(),
                ptr::null_mut(),
                handle::MaybeLocal::empty().into_raw(),
                handle::MaybeLocal::empty().into_raw(),
                v8_sys::DeserializeInternalFieldsCallback {
                    callback: None,
                    data: ptr::null_mut(),
                },
            ))
        }
    }

    /// Binds the context to the current scope.
    ///
    /// Within this scope, functionality that relies on implicit contexts will work.
    pub fn scope(&mut self) -> Scope {
        unsafe {
            self.0.Enter();
        }
        Scope(self)
    }

    /// Returns the global proxy object.
    ///
    /// Global proxy object is a thin wrapper whose prototype points to actual context's global
    /// object with the properties like Object, etc. This is done that way for security reasons (for
    /// more details see https://wiki.mozilla.org/Gecko:SplitWindow).
    ///
    /// Please note that changes to global proxy object prototype most probably would break VM---v8
    /// expects only global object as a prototype of global proxy object.
    ///
    pub fn global(&self) -> handle::Local<value::Object> {
        unsafe {
            handle::Local::new(self.0.Global())
        }
    }
}

impl<'c> Scope<'c> {
    pub fn context(&self) -> &Context {
        &self.0
    }

    pub fn context_mut(&mut self) -> &mut Context {
        &mut self.0
    }
}

impl<'c> Drop for Scope<'c> {
    fn drop(&mut self) {
        unsafe { (self.0).0.Exit() }
    }
}
