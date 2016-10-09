//! Allocators for array buffers.
use v8_sys as v8;

use std::os;
use std::mem;

/// A simple array buffer allocator that guarantees that all allocated
/// blocks are coercible to `Vec`s.
#[derive(Debug)]
pub struct Allocator(v8::ArrayBuffer_AllocatorPtr);

impl Allocator {
    /// Creates a new allocator.
    pub fn new() -> Allocator {
        let raw = unsafe { v8::ArrayBuffer_Allocator_Create(ALLOCATOR_FUNCTIONS) };
        if raw.is_null() {
            panic!("Could not create ArrayBuffer::Allocator");
        }

        Allocator(raw)
    }

    /// Returns the underlying raw pointer behind this allocator.
    pub fn as_raw(&self) -> v8::ArrayBuffer_AllocatorPtr {
        self.0
    }
}

impl Drop for Allocator {
    fn drop(&mut self) {
        unsafe {
            v8::ArrayBuffer_Allocator_Destroy(self.0);
        }
    }
}

const ALLOCATOR_FUNCTIONS: v8::AllocatorFunctions = v8::AllocatorFunctions {
    Allocate: Some(allocate),
    AllocateUninitialized: Some(allocate_uninitialized),
    Free: Some(free),
};

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
    // TODO: restore `cap` here?  Can this possibly leak memory?
    drop(Vec::from_raw_parts(data, length, length));
}
