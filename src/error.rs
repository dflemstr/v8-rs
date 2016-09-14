use v8_sys as v8;
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

pub struct Message<'a>(&'a isolate::Isolate, v8::MessageRef);
pub struct StackTrace<'a>(&'a isolate::Isolate, v8::StackTraceRef);
pub struct StackFrame<'a>(&'a isolate::Isolate, v8::StackFrameRef);

impl<'a> Message<'a> {
    // TODO: pub fn get_script_origin(&self)

    pub fn get(&self) -> value::String {
        unsafe {
            value::String::from_raw(self.0,
                                    util::invoke(self.0, |c| v8::Message_Get(c, self.1)).unwrap())
        }
    }

    pub unsafe fn from_raw(isolate: &'a isolate::Isolate, raw: v8::MessageRef) -> Message<'a> {
        Message(isolate, raw)
    }
}

drop!(Message, v8::Message_DestroyRef);
drop!(StackTrace, v8::StackTrace_DestroyRef);
drop!(StackFrame, v8::StackFrame_DestroyRef);
