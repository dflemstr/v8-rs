use v8_sys as v8;
use context;
use error;
use isolate;
use std::any;
use std::mem;
use std::panic;
use std::ptr;
use value;

pub fn invoke<F, B>(isolate: &isolate::Isolate, func: F) -> error::Result<B>
    where F: FnOnce(v8::RustContext) -> B
{
    invoke_inner(isolate, None, func)
}

pub fn invoke_ctx<F, B>(isolate: &isolate::Isolate,
                        context: &context::Context,
                        func: F)
                        -> error::Result<B>
    where F: FnOnce(v8::RustContext) -> B
{
    invoke_inner(isolate, Some(context), func)
}

fn invoke_inner<F, B>(isolate: &isolate::Isolate,
                      context: Option<&context::Context>,
                      func: F)
                      -> error::Result<B>
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
        let exception = unsafe { value::Value::from_raw(isolate, exception) };
        let message = unsafe { error::Message::from_raw(isolate, message) };
        let context = context.map(|c| c.clone())
            .or_else(|| isolate.current_context())
            .unwrap_or_else(|| context::Context::new(&isolate));

        if exception.is_object() {
            let exception = exception.into_object().unwrap();
            let panic_info_key = value::String::from_str(isolate, "panicInfo");

            if exception.has(&context, &panic_info_key) {
                match exception.get(&context, &panic_info_key).into_external() {
                    Some(panic_info) => {
                        let panic_info =
                            unsafe {
                                Box::from_raw(panic_info.value() as *mut Box<any::Any + Send + 'static>)
                            };
                        panic::resume_unwind(panic_info);
                    }
                    None => {
                        // Somebody is playing tricks on us, creating random panicInfo properties...
                    }
                }
            }
        }

        let message_str = message.get(&context).value();
        let stack_trace = message.get_stack_trace().to_captured();
        Err(error::ErrorKind::Javascript(message_str, stack_trace).into())
    }
}

pub extern "C" fn callback(callback_info: v8::FunctionCallbackInfoPtr_Value) {
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

        let result = panic::catch_unwind(|| {
            let callback_ext = data.get_internal_field(0).into_external().unwrap();
            let callback_ptr: *mut Box<value::FunctionCallback> = callback_ext.value();
            let callback = callback_ptr.as_ref().unwrap();
            callback(info)
        });

        match result {
            Ok(value) => {
                let result = value.unwrap_or_else(|exception| throw_exception(&isolate, &exception));
                callback_info.ReturnValue = result.as_raw();
                mem::forget(result);
            }
            Err(panic) => {
                let error = create_panic_error(&isolate, panic);
                callback_info.ThrownValue = error.as_raw();
                mem::forget(error);
            }
        }
    }
}

fn throw_exception(isolate: &isolate::Isolate, exception: &value::Value) -> value::Value {
    unsafe {
        let raw = v8::Isolate_ThrowException(isolate.as_raw(), exception.as_raw()).as_mut().unwrap();
        ::value::Value::from_raw(isolate, raw)
    }
}

fn create_panic_error(isolate: &isolate::Isolate,
                      panic: Box<any::Any + Send + 'static>)
                      -> value::Value {
    let context = isolate.current_context()
        .unwrap_or_else(|| context::Context::new(&isolate));
    let message = if let Some(s) = panic.downcast_ref::<String>() {
        value::String::from_str(&isolate, &format!("Rust panic: {}", s))
    } else {
        value::String::from_str(&isolate, "Rust panic")
    };

    let exception = value::Exception::error(&isolate, &message).into_object().unwrap();

    let panic_info_key = value::String::from_str(isolate, "panicInfo");
    let panic_info = unsafe { value::External::new(&isolate, Box::into_raw(Box::new(panic))) };
    exception.set(&context, &panic_info_key, &panic_info);

    exception.into()
}

macro_rules! reference {
    ($typ:ident, $clone:expr, $dtor:expr) => {
        impl Clone for $typ {
            fn clone(&self) -> $typ {
                let raw = unsafe { $crate::util::invoke(&self.0, |c| $clone(c, self.1)).unwrap() };
                $typ(self.0.clone(), raw)
            }
        }

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
