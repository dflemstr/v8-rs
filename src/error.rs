use v8_sys as v8;
use error;
use isolate;
use util;
use value;

error_chain! {
    errors {
        Javascript(message: String) {
            description("Javascript exception")
            display("Javascript exception: {}", message)
        }
    }
}

pub struct Message<'a>(&'a isolate::Isolate, *mut v8::Message);
pub struct StackTrace<'a>(&'a isolate::Isolate, *mut v8::StackTrace);
pub struct StackFrame<'a>(&'a isolate::Isolate, *mut v8::StackFrame);

impl<'a> Message<'a> {

    // TODO: pub fn get_script_origin(&self)

    pub fn get(&self) -> error::Result<value::String> {
        unsafe {
            Ok(value::String::from_raw(self.0, try!(util::invoke(self.0, |c| v8::Message_Get(c, self.1)))))
        }
    }

    pub unsafe fn from_raw(isolate: &'a isolate::Isolate, raw: *mut v8::Message) -> Message<'a> {
        Message(isolate, raw)
    }
}

drop!(Message, v8::Message_Destroy);
drop!(StackTrace, v8::StackTrace_Destroy);
drop!(StackFrame, v8::StackFrame_Destroy);
