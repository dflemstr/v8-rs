//! Error types and utilities.

use std::fmt;
use v8_sys as v8;
use context;
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

/// A captured stack trace, that is separated from its underlying isolate.
#[derive(Clone, Debug)]
pub struct CapturedStackTrace {
    pub frames: Vec<CapturedStackFrame>,
}

/// A captured stack frame, that is separated from its underlying isolate.
#[derive(Clone, Debug)]
pub struct CapturedStackFrame {
    pub line: u32,
    pub column: u32,
    pub script_name: Option<String>,
    pub function_name: Option<String>,
    pub is_eval: bool,
    pub is_constructor: bool,
}

/// An error message.
#[derive(Debug)]
pub struct Message(isolate::Isolate, v8::MessageRef);

/// A stack trace, that is bound to an isolate.
#[derive(Debug)]
pub struct StackTrace(isolate::Isolate, v8::StackTraceRef);

/// A stack frame, that is bound to an isolate.
#[derive(Debug)]
pub struct StackFrame(isolate::Isolate, v8::StackFrameRef);

impl Message {
    // TODO: pub fn get_script_origin(&self)

    /// The error message string.
    pub fn get(&self, context: &context::Context) -> value::String {
        let _g = context.make_current();
        unsafe {
            value::String::from_raw(&self.0,
                                    util::invoke(&self.0, |c| v8::Message_Get(c, self.1)).unwrap())
        }
    }

    /// The stack trace to the point where the error was generated.
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
    /// The stack frames that this stack trace consists of.
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

    /// Creates a captured version of this stack trace, that doesn't retain a reference to its
    /// isolate.
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
    /// The line number at which this stack frame was pushed.
    pub fn get_line_number(&self) -> u32 {
        unsafe {
            util::invoke(&self.0, |c| v8::StackFrame_GetLineNumber(c, self.1)).unwrap() as u32
        }
    }

    /// The column number at which this stack frame was pushed.
    pub fn get_column(&self) -> u32 {
        unsafe { util::invoke(&self.0, |c| v8::StackFrame_GetColumn(c, self.1)).unwrap() as u32 }
    }

    /// The script file name in which this stack frame was pushed.
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

    /// The function name in which this stack frame was pushed.
    pub fn get_function_name(&self) -> value::String {
        unsafe {
            let raw = util::invoke(&self.0, |c| v8::StackFrame_GetFunctionName(c, self.1)).unwrap();
            value::String::from_raw(&self.0, raw)
        }
    }

    /// Whether this stack frame is part of an eval call.
    pub fn is_eval(&self) -> bool {
        unsafe { 0 != util::invoke(&self.0, |c| v8::StackFrame_IsEval(c, self.1)).unwrap() }
    }

    /// Whether this stack frame is part of a constructor call.
    pub fn is_constructor(&self) -> bool {
        unsafe { 0 != util::invoke(&self.0, |c| v8::StackFrame_IsConstructor(c, self.1)).unwrap() }
    }

    /// Creates a captured version of this stack frame, that doesn't retain a reference to its
    /// isolate.
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
reference!(StackTrace,
           v8::StackTrace_CloneRef,
           v8::StackTrace_DestroyRef);
reference!(StackFrame,
           v8::StackFrame_CloneRef,
           v8::StackFrame_DestroyRef);
