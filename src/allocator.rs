//! Allocators for array buffers.
use v8_sys;

use std::fmt;
use std::os;
use std::mem;
use std::ptr;

/// A simple array buffer allocator that guarantees that all allocated
/// blocks are coercible to `Vec`s of `u8`.
pub struct Allocator(ptr::Shared<v8_sys::ArrayBuffer_Allocator>);

impl Allocator {
    /// Creates a new allocator.
    pub fn new() -> Allocator {
        let raw = unsafe {
            ptr::Shared::new(v8_sys::impls::CreateArrayBufferAllocator(
                ALLOCATOR_FUNCTIONS,
                ptr::null_mut(),
            ))
        }.expect("could not create ArrayBuffer::Allocator");

        Allocator(raw)
    }

    /// Returns the underlying raw pointer behind this allocator.
    pub fn as_ptr(&self) -> *mut v8_sys::ArrayBuffer_Allocator {
        self.0.as_ptr()
    }
}

impl fmt::Debug for Allocator {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Allocator({:?})", unsafe { self.0.as_ref() })
    }
}

impl Drop for Allocator {
    fn drop(&mut self) {
        unsafe {
            v8_sys::ArrayBuffer_Allocator_Allocator_destructor(self.0.as_ptr());
        }
    }
}

const ALLOCATOR_FUNCTIONS: v8_sys::impls::ArrayBufferAllocatorFunctions =
    v8_sys::impls::ArrayBufferAllocatorFunctions {
        Destroy: None,
        Allocate: Some(allocate),
        AllocateUninitialized: Some(allocate_uninitialized),
        Reserve: None,
        Free: Some(free),
        FreeMode: Some(free_mode),
        SetProtection: None,
    };

unsafe extern "C" fn allocate(
    _this: *mut os::raw::c_void,
    _fallback_fn: Option<unsafe extern "C" fn(*mut os::raw::c_void, usize) -> *mut os::raw::c_void>,
    _fallback_arg: *mut os::raw::c_void,
    length: usize,
) -> *mut os::raw::c_void {
    let mut data = Vec::with_capacity(length);
    data.resize(length, 0u8);
    let ptr = data.as_mut_ptr();
    mem::forget(data);

    ptr as *mut os::raw::c_void
}

unsafe extern "C" fn allocate_uninitialized(
    _this: *mut os::raw::c_void,
    _fallback_fn: Option<unsafe extern "C" fn(*mut os::raw::c_void, usize) -> *mut os::raw::c_void>,
    _fallback_arg: *mut os::raw::c_void,
    length: usize,
) -> *mut os::raw::c_void {
    let mut data = Vec::with_capacity(length);
    data.set_len(length);

    let ptr = data.as_mut_ptr();
    mem::forget(data);

    ptr as *mut os::raw::c_void
}

unsafe extern "C" fn free(
    _this: *mut os::raw::c_void,
    _fallback_fn: Option<unsafe extern "C" fn(*mut os::raw::c_void, *mut os::raw::c_void, usize)>,
    _fallback_arg: *mut os::raw::c_void,
    data: *mut os::raw::c_void,
    length: usize,
) {
    // TODO: restore `cap` here?  Can this possibly leak memory?
    drop(Vec::from_raw_parts(data, length, length));
}

unsafe extern "C" fn free_mode(
    _this: *mut os::raw::c_void,
    fallback_fn: Option<
        unsafe extern "C" fn(*mut os::raw::c_void,
                             *mut os::raw::c_void,
                             usize,
                             v8_sys::ArrayBuffer_Allocator_AllocationMode),
    >,
    fallback_arg: *mut os::raw::c_void,
    data: *mut os::raw::c_void,
    length: usize,
    mode: v8_sys::ArrayBuffer_Allocator_AllocationMode,
) {
    if mode == v8_sys::ArrayBuffer_Allocator_AllocationMode_kNormal {
        // TODO: restore `cap` here?  Can this possibly leak memory?
        drop(Vec::from_raw_parts(data, length, length));
    } else {
        fallback_fn.unwrap()(fallback_arg, data, length, mode);
    }
}
