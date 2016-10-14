//! Templates for constructing functions and objects efficiently.
use v8_sys as v8;
use isolate;
use util;
use value;
use value::Data;
use context;
use std::os;
use std::ptr;
use std::marker::PhantomData;
use std::mem;
use std::ops;
use std::ffi;

/// The superclass of object and function templates.
#[derive(Debug)]
pub struct Template(isolate::Isolate, v8::TemplateRef);

/// A FunctionTemplate is used to create functions at runtime.
///
/// There can only be one function created from a FunctionTemplate in a context.  Any modification
/// of a FunctionTemplate after first instantiation will trigger a crash.  A FunctionTemplate can
/// have properties, these properties are added to the function object when it is created.
#[derive(Debug)]
pub struct FunctionTemplate<T = ()>(isolate::Isolate, v8::FunctionTemplateRef, PhantomData<T>);

/// An ObjectTemplate is used to create objects at runtime.
///
/// Properties added to an ObjectTemplate are added to each object created from the ObjectTemplate.
#[derive(Debug)]
pub struct ObjectTemplate<T = ()>(isolate::Isolate, v8::ObjectTemplateRef, PhantomData<T>);

/// A Signature specifies which receiver is valid for a function.
#[derive(Debug)]
pub struct Signature(isolate::Isolate, v8::SignatureRef);

impl Signature {
    /// Creates a new signature.
    pub fn new(isolate: &isolate::Isolate) -> Signature {
        let raw = unsafe {
            util::invoke(isolate,
                         |c| v8::Signature_New(c, isolate.as_raw(), ptr::null_mut()))
                .unwrap()
        };
        Signature(isolate.clone(), raw)
    }

    /// Creates a new signature with the specified receiver.
    pub fn new_with_receiver(isolate: &isolate::Isolate, receiver: &FunctionTemplate) -> Signature {
        let raw = unsafe {
            util::invoke(isolate,
                         |c| v8::Signature_New(c, isolate.as_raw(), receiver.1))
                .unwrap()
        };
        Signature(isolate.clone(), raw)
    }
}

impl<T> FunctionTemplate<T> {
    /// Creates a function template.
    pub fn new(isolate: &isolate::Isolate,
               context: &context::Context,
               callback: Box<Fn(value::FunctionCallbackInfo<T>) -> value::Value + 'static>)
               -> FunctionTemplate<T> {
        let raw = unsafe {
            let callback_ptr = Box::into_raw(Box::new(callback));
            let callback_ext = value::External::new::<Box<Fn(value::FunctionCallbackInfo<T>) -> value::Value + 'static>>(&isolate, callback_ptr);

            let template = ObjectTemplate::new(isolate);
            template.set_internal_field_count(1);

            let closure = template.new_instance(context);
            closure.set_internal_field(0, &callback_ext);

            util::invoke(isolate, |c| {
                    v8::FunctionTemplate_New(c,
                                             context.as_raw(),
                                             Some(util::callback::<T>),
                                             (&closure as &value::Value).as_raw(),
                                             ptr::null_mut(),
                                             0,
                                             v8::ConstructorBehavior::ConstructorBehavior_kAllow)
                })
                .unwrap()
        };
        FunctionTemplate::<T>(isolate.clone(), raw, PhantomData)
    }

    /// Returns the unique function instance in the current execution context.
    pub fn get_function(self, context: &context::Context) -> value::Function {
        unsafe {
            let raw =
                util::invoke(&self.0,
                             |c| v8::FunctionTemplate_GetFunction(c, self.1, context.as_raw()))
                    .unwrap();
            value::Function::from_raw(&self.0, raw)
        }
    }

    /// Returns the underlying raw pointer behind this function template.
    pub fn as_raw(&self) -> v8::FunctionTemplateRef {
        self.1
    }
}

impl ObjectTemplate {
    /// Creates an ObjectTemplate.
    pub fn new(isolate: &isolate::Isolate) -> ObjectTemplate {
        ObjectTemplate::<()>::with_internal(isolate)
    }

    /// Creates an ObjectTemplate<T> where T is the type of the internal rust object.
    pub fn with_internal<T>(isolate: &isolate::Isolate) -> ObjectTemplate<T> {
        let raw = unsafe {
            util::invoke(isolate,
                         |c| v8::ObjectTemplate_New(c, isolate.as_raw(), ptr::null_mut()))
                .unwrap()
        };
        ObjectTemplate(isolate.clone(), raw, PhantomData)
    }
}

impl<T> ObjectTemplate<T> {
    /// Returns the number of internal fields for objects generated from this template.
    pub fn internal_field_count(&self) -> i32 {
        unsafe { util::invoke(&self.0, |c| { v8::ObjectTemplate_InternalFieldCount(c, self.1)}).unwrap() }
    }

    /// Sets the number of internal fields for objects generated from this template.
    pub unsafe fn set_internal_field_count(&self, value: usize) {
        util::invoke(&self.0, |c| {
                v8::ObjectTemplate_SetInternalFieldCount(c, self.1, value as os::raw::c_int)
            })
            .unwrap();
    }

    pub fn set(&self, name: &str, value: &value::Data) {
        let cname = ffi::CString::new(name).unwrap();
        let template: &Template = self;
        unsafe {
            util::invoke(&self.0, |c| {
                    v8::Template_Set_Raw(c,
                                         template.1,
                                         self.0.as_raw(),
                                         cname.as_ptr(),
                                         value.as_raw())
                })
                .unwrap()
        };
    }

    /// Sets a function template as `name` on instances of this template ensuring that the callback type of the FunctionTemplate matches the object instance internal type.
    pub fn set_callback(&self, name: &str, value: &FunctionTemplate<T>) {
        let cname = ffi::CString::new(name).unwrap();
        let template: &Template = self;
        unsafe {
            util::invoke(&self.0, |c| {
                    v8::Template_Set_Raw(c,
                                         template.1,
                                         self.0.as_raw(),
                                         cname.as_ptr(),
                                         value.as_raw() as v8::DataRef)
                })
                .unwrap()
        }; 
    }

    /// Creates a new object instance based off of this template.
    pub fn new_instance(&self, context: &context::Context) -> value::Object {
        unsafe {
            let raw = util::invoke(&self.0,
                                   |c| v8::ObjectTemplate_NewInstance(c, self.1, context.as_raw()))
                .unwrap();
            value::Object::from_raw(&self.0, raw)
        }
    }

    /// Creates a new object instance based off of this template with an internal rust object accessible from callbacks.
    pub fn new_instance_with_internal(&self, context: &context::Context, internal: T) -> value::Object {
        let wrapped_ptr: *mut Box<T> = Box::into_raw(Box::new(Box::new(internal)));
        if self.internal_field_count() < 1 { 
            unsafe { self.set_internal_field_count(1) };
        }

        let object = self.new_instance(context);

        unsafe {
            let external = value::External::new(&self.0, wrapped_ptr);
            object.set_internal_field(0, &external);
        }
        object
    }

    /// Creates an object template from a set of raw pointers.
    pub unsafe fn from_raw(isolate: &isolate::Isolate,
                           raw: v8::ObjectTemplateRef)
                           -> ObjectTemplate {
        ObjectTemplate(isolate.clone(), raw, PhantomData)
    }

    /// Returns the underlying raw pointer behind this object template.
    pub fn as_raw(&self) -> v8::ObjectTemplateRef {
        self.1
    }
}

impl<T> Clone for FunctionTemplate<T> {
    fn clone(&self) -> FunctionTemplate<T> {
        let raw = unsafe { util::invoke(&self.0, |c| v8::FunctionTemplate_CloneRef(c, self.1)).unwrap() };
        FunctionTemplate(self.0.clone(), raw, PhantomData)
    }
}

impl<T> Drop for FunctionTemplate<T> {
    fn drop(&mut self) {
        unsafe {
            v8::FunctionTemplate_DestroyRef(self.1)
        }
    }
}

impl<T> Clone for ObjectTemplate<T> {
    fn clone(&self) -> ObjectTemplate<T> {
        let raw = unsafe { util::invoke(&self.0, |c| v8::ObjectTemplate_CloneRef(c, self.1)).unwrap() };
        ObjectTemplate(self.0.clone(), raw, PhantomData)
    }
}

impl<T> Drop for ObjectTemplate<T> {
    fn drop(&mut self) {
        unsafe {
            v8::ObjectTemplate_DestroyRef(self.1)
        }
    }
}

impl<T> From<ObjectTemplate<T>> for Template {
    fn from(child: ObjectTemplate<T>) -> Template {
        unsafe { mem::transmute(child) }
    }
}


impl<T> ops::Deref for ObjectTemplate<T> {
    type Target = Template;

    fn deref(&self) -> &Self::Target {
        unsafe { mem::transmute(self) }
    }
}

inherit!(Template, Data);
inherit!(FunctionTemplate, Template);
inherit!(Signature, Data);

reference!(Template, v8::Template_CloneRef, v8::Template_DestroyRef);
reference!(Signature, v8::Signature_CloneRef, v8::Signature_DestroyRef);
