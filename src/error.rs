//! Error types and utilities.

use std::fmt;
use v8_sys as v8;
use isolate;
use util;
use value;

error_chain! {
    errors {
        Javascript(message: String, stack_trace: CapturedStackTrace) {
            description("Javascript exception")
            display("Javascript exception: {}\n{}", message, stack_trace)
        }
    }
}

#[derive(Clone, Debug)]
pub struct CapturedStackTrace {
    frames: Vec<CapturedStackFrame>,
}

#[derive(Clone, Debug)]
pub struct CapturedStackFrame {
    line: u32,
    column: u32,
    script_name: Option<String>,
    function_name: Option<String>,
    is_eval: bool,
    is_constructor: bool,
}

/// An error message.
#[derive(Debug)]
pub struct Message(isolate::Isolate, v8::MessageRef);

#[derive(Debug)]
pub struct StackTrace(isolate::Isolate, v8::StackTraceRef);

#[derive(Debug)]
pub struct StackFrame(isolate::Isolate, v8::StackFrameRef);

impl Message {
    // TODO: pub fn get_script_origin(&self)

    pub fn get(&self) -> value::String {
        unsafe {
            value::String::from_raw(&self.0,
                                    util::invoke(&self.0, |c| v8::Message_Get(c, self.1)).unwrap())
        }
    }

    pub fn get_stack_trace(&self) -> StackTrace {
        let raw =
            unsafe { util::invoke(&self.0, |c| v8::Message_GetStackTrace(c, self.1)).unwrap() };

        StackTrace(self.0.clone(), raw)
    }

    pub unsafe fn from_raw(isolate: &isolate::Isolate, raw: v8::MessageRef) -> Message {
        Message(isolate.clone(), raw)
    }
}

impl StackTrace {
    pub fn get_frames(&self) -> Vec<StackFrame> {
        let count =
            unsafe { util::invoke(&self.0, |c| v8::StackTrace_GetFrameCount(c, self.1)).unwrap() };
        let mut result = Vec::with_capacity(count as usize);

        for i in 0..count {
            let raw_frame = unsafe {
                util::invoke(&self.0, |c| v8::StackTrace_GetFrame(c, self.1, i as u32)).unwrap()
            };
            let frame = StackFrame(self.0.clone(), raw_frame);
            result.push(frame);
        }

        result
    }

    pub fn to_captured(&self) -> CapturedStackTrace {
        CapturedStackTrace {
            frames: self.get_frames()
                .iter()
                .map(StackFrame::to_captured)
                .collect(),
        }
    }
}

impl StackFrame {
    pub fn get_line_number(&self) -> u32 {
        unsafe { util::invoke(&self.0, |c| v8::StackFrame_GetLineNumber(c, self.1)).unwrap() as u32 }
    }

    pub fn get_column(&self) -> u32 {
        unsafe { util::invoke(&self.0, |c| v8::StackFrame_GetColumn(c, self.1)).unwrap() as u32 }
    }

    pub fn get_script_name(&self) -> Option<value::String> {
        unsafe {
            let raw = util::invoke(&self.0, |c| v8::StackFrame_GetScriptName(c, self.1)).unwrap();
            if raw.is_null() {
                None
            } else {
                Some(value::String::from_raw(&self.0, raw))
            }
        }
    }

    pub fn get_function_name(&self) -> value::String {
        unsafe {
            let raw = util::invoke(&self.0, |c| v8::StackFrame_GetFunctionName(c, self.1)).unwrap();
            value::String::from_raw(&self.0, raw)
        }
    }

    pub fn is_eval(&self) -> bool {
        unsafe { 0 != util::invoke(&self.0, |c| v8::StackFrame_IsEval(c, self.1)).unwrap() }
    }

    pub fn is_constructor(&self) -> bool {
        unsafe { 0 != util::invoke(&self.0, |c| v8::StackFrame_IsConstructor(c, self.1)).unwrap() }
    }

    pub fn to_captured(&self) -> CapturedStackFrame {
        let function_name = self.get_function_name().value();
        CapturedStackFrame {
            line: self.get_line_number(),
            column: self.get_column(),
            script_name: self.get_script_name().map(|ref s| s.value()),
            function_name: if function_name.is_empty() {
                None
            } else {
                Some(function_name)
            },
            is_eval: self.is_eval(),
            is_constructor: self.is_constructor(),
        }
    }
}

impl fmt::Display for CapturedStackTrace {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for frame in self.frames.iter() {
            try!(writeln!(f, "{}", frame));
        }
        Ok(())
    }
}

impl fmt::Display for CapturedStackFrame {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        try!(write!(f, "    at "));

        if self.is_constructor {
            try!(write!(f, "new "));
        }

        if let Some(ref function_name) = self.function_name {
            try!(write!(f, "{} (", function_name));

            if self.is_eval {
                try!(write!(f, "eval "));
            }

            try!(write!(f,
                        "{}:{}:{})",
                        self.script_name.as_ref().map(|n| n.as_str()).unwrap_or("<anon>"),
                        self.line,
                        self.column));
        } else {
            if self.is_eval {
                try!(write!(f, "eval "));
            }
            try!(write!(f,
                        "{}:{}:{}",
                        self.script_name.as_ref().map(|n| n.as_str()).unwrap_or("<anon>"),
                        self.line,
                        self.column));
        }

        Ok(())
    }
}

reference!(Message, v8::Message_CloneRef, v8::Message_DestroyRef);
reference!(StackTrace, v8::StackTrace_CloneRef, v8::StackTrace_DestroyRef);
reference!(StackFrame, v8::StackFrame_CloneRef, v8::StackFrame_DestroyRef);
