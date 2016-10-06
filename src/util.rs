use v8_sys as v8;
use error;
use isolate;
use std::ptr;
use std::mem;
use value;

pub fn invoke<F, B>(isolate: &isolate::Isolate, func: F) -> error::Result<B>
    where F: FnOnce(v8::RustContext) -> B
{
    let mut exception = ptr::null_mut();
    let mut message = ptr::null_mut();
    let rust_ctx = v8::RustContext {
        isolate: isolate.as_raw(),
        exception: &mut exception,
        message: &mut message,
    };

    let result = func(rust_ctx);

    if exception.is_null() {
        assert!(message.is_null());
        Ok(result)
    } else {
        assert!(!message.is_null());
        // TODO: maybe use exception value for something; the lifetime
        // is annoying though.
        drop(unsafe { value::Value::from_raw(isolate, exception) });
        let message = unsafe { error::Message::from_raw(isolate, message) };
        let message_str = message.get().to_string();

        let stack_trace = message.get_stack_trace().to_captured();

        Err(error::ErrorKind::Javascript(message_str, stack_trace).into())
    }
}

pub extern "C" fn callback<F>(callback_info: v8::FunctionCallbackInfoPtr_Value) where F: Fn(value::FunctionCallbackInfo) -> value::Value {
    unsafe {
        let callback_info = callback_info.as_mut().unwrap();
        let isolate = isolate::Isolate::from_raw(callback_info.GetIsolate);
        let data = value::Object::from_raw(&isolate, callback_info.Data as v8::ObjectRef);

        let length = callback_info.Length as isize;
        let args = (0..length)
            .map(|i| value::Value::from_raw(&isolate, *callback_info.Args.offset(i)))
            .collect();
        let info = value::FunctionCallbackInfo {
            isolate: isolate.clone(),
            length: length,
            args: args,
            this: value::Object::from_raw(&isolate, callback_info.This),
            holder: value::Object::from_raw(&isolate, callback_info.Holder),
            new_target: value::Value::from_raw(&isolate, callback_info.NewTarget),
            is_construct_call: 0 != callback_info.IsConstructCall,
        };

        let callback_ptr: *mut Box<F> = data.get_aligned_pointer_from_internal_field(0);
        let callback = Box::from_raw(callback_ptr);

        let r = callback(info);

        mem::forget(callback);

        callback_info.ReturnValue = r.as_raw();
        mem::forget(r);
    }
}

macro_rules! drop {
    ($typ:ident, $dtor:expr) => {
        impl Drop for $typ {
            fn drop(&mut self) {
                // SAFETY: This is unsafe because it calls a native method with a void pointer.
                // It's safe because the macro is only used with a type and its corresponding
                // destructor.
                unsafe {
                    $dtor(self.1)
                }
            }
        }
    }
}

macro_rules! subtype {
    ($child:ident, $parent:ident) => {
        impl From<$child> for $parent {
            fn from(child: $child) -> $parent {
                unsafe { mem::transmute(child) }
            }
        }
    }
}

macro_rules! inherit {
    ($child:ident, $parent:ident) => {
        subtype!($child, $parent);

        impl ops::Deref for $child {
            type Target = $parent;

            fn deref(&self) -> &Self::Target {
                unsafe { mem::transmute(self) }
            }
        }
    }
}
