//! Templates for constructing functions and objects efficiently.
use v8_sys;
use isolate;
use handle;
use util;
use value;
use value::Data;
use context;
use std::os;
use std::ptr;
use std::mem;
use std::ops;
use std::ffi;

/// The superclass of object and function templates.
#[derive(Debug)]
pub struct Template(v8_sys::Template);

/// A FunctionTemplate is used to create functions at runtime.
///
/// There can only be one function created from a FunctionTemplate in a context.  Any modification
/// of a FunctionTemplate after first instantiation will trigger a crash.  A FunctionTemplate can
/// have properties, these properties are added to the function object when it is created.
#[derive(Debug)]
pub struct FunctionTemplate(v8_sys::FunctionTemplate);

/// An ObjectTemplate is used to create objects at runtime.
///
/// Properties added to an ObjectTemplate are added to each object created from the ObjectTemplate.
#[derive(Debug)]
pub struct ObjectTemplate(v8_sys::ObjectTemplate);

/// A Signature specifies which receiver is valid for a function.
#[derive(Debug)]
pub struct Signature(v8_sys::Signature);

impl Signature {
    /// Creates a new signature.
    pub fn new(isolate: &mut isolate::Isolate) -> handle::Local<Signature> {
        unsafe { handle::Local::new(v8_sys::Signature::New(&mut *isolate, ptr::null_mut())) }
    }

    /// Creates a new signature with the specified receiver.
    pub fn new_with_receiver(isolate: &mut isolate::Isolate, receiver: handle::Local<FunctionTemplate>) -> Signature {
        unsafe { handle::Local::new(v8_sys::Signature::New(&mut *isolate, receiver.as_raw())) }
    }
}

impl FunctionTemplate {
    /// Creates a function template.
    pub fn new(isolate: &isolate::Isolate,
               context: &context::Context,
               callback: Box<value::FunctionCallback>)
               -> FunctionTemplate {
        let raw = unsafe {
            let callback_ptr = Box::into_raw(Box::new(callback));
            let callback_ext = value::External::new::<Box<value::FunctionCallback>>(&isolate, callback_ptr);

            let template = ObjectTemplate::new(isolate);
            template.set_internal_field_count(1);

            let closure = template.new_instance(context);
            closure.set_internal_field(0, &callback_ext);

            util::invoke_ctx(isolate, context, |c| {
                v8_sys::v8_FunctionTemplate_New(c,
                                                context.as_raw(),
                                                Some(util::callback),
                                                (&closure as &value::Value).as_raw(),
                                                ptr::null_mut(),
                                                0,
                                                v8_sys::ConstructorBehavior::ConstructorBehavior_kAllow)
            })
                .unwrap()
        };
        FunctionTemplate(isolate.clone(), raw)
    }

    /// Returns the unique function instance in the current execution context.
    pub fn get_function(self, context: &context::Context) -> value::Function {
        unsafe {
            let raw = util::invoke_ctx(&self.0, context, |c| {
                v8_sys::v8_FunctionTemplate_GetFunction(c, self.1, context.as_raw())
            })
                .unwrap();
            value::Function::from_raw(&self.0, raw)
        }
    }
}

impl ObjectTemplate {
    /// Creates an ObjectTemplate.
    pub fn new(isolate: &isolate::Isolate) -> ObjectTemplate {
        let raw = unsafe {
            util::invoke(isolate,
                         |c| v8_sys::v8_ObjectTemplate_New(c, isolate.as_raw(), ptr::null_mut()))
                .unwrap()
        };
        ObjectTemplate(isolate.clone(), raw)
    }

    /// Sets the number of internal fields for objects generated from this template.
    pub fn set_internal_field_count(&self, value: usize) {
        unsafe {
            util::invoke(&self.0, |c| {
                v8_sys::v8_ObjectTemplate_SetInternalFieldCount(c, self.1, value as os::raw::c_int)
            })
                .unwrap()
        };
    }

    pub fn set(&self, name: &str, value: &value::Data) {
        let cname = ffi::CString::new(name).unwrap();
        let template: &Template = self;
        unsafe {
            util::invoke(&self.0, |c| {
                v8_sys::v8_Template_Set_Raw(c,
                                            template.1,
                                            self.0.as_raw(),
                                            mem::transmute(cname.as_ptr()),
                                            value.as_raw())
            })
                .unwrap()
        };
    }

    /// Creates a new object instance based off of this template.
    pub fn new_instance(&self, context: &context::Context) -> value::Object {
        unsafe {
            let raw = util::invoke_ctx(&self.0, context, |c| {
                v8_sys::v8_ObjectTemplate_NewInstance(c, self.1, context.as_raw())
            })
                .unwrap();
            value::Object::from_raw(&self.0, raw)
        }
    }

    /// Creates an object template from a set of raw pointers.
    pub unsafe fn from_raw(isolate: &isolate::Isolate,
                           raw: v8_sys::ObjectTemplateRef)
                           -> ObjectTemplate {
        ObjectTemplate(isolate.clone(), raw)
    }

    /// Returns the underlying raw pointer behind this object template.
    pub fn as_raw(&self) -> v8_sys::ObjectTemplateRef {
        self.1
    }
}

inherit!(Template, Data);
inherit!(ObjectTemplate, Template);
inherit!(FunctionTemplate, Template);
inherit!(Signature, Data);

reference!(Template, v8_sys::v8_Template_CloneRef, v8_sys::v8_Template_DestroyRef);
reference!(FunctionTemplate,
           v8_sys::v8_FunctionTemplate_CloneRef,
           v8_sys::v8_FunctionTemplate_DestroyRef);
reference!(ObjectTemplate,
           v8_sys::v8_ObjectTemplate_CloneRef,
           v8_sys::v8_ObjectTemplate_DestroyRef);
reference!(Signature, v8_sys::v8_Signature_CloneRef, v8_sys::v8_Signature_DestroyRef);
