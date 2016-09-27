use v8_sys as v8;
use error;
use isolate;
use std::ptr;
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

macro_rules! drop {
    ($typ:ident, $dtor:expr) => {
        impl<'a> Drop for $typ<'a> {
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
