//! Error types and utilities.

use std::fmt;
use std::ptr;
use v8_sys;
use context;
use handle;
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
pub struct Message(v8_sys::Message);

/// A stack trace, that is bound to an isolate.
#[derive(Debug)]
pub struct StackTrace(v8_sys::StackTrace);

/// A stack frame, that is bound to an isolate.
#[derive(Debug)]
pub struct StackFrame(v8_sys::StackFrame);

impl Message {
    // TODO: pub fn get_script_origin(&self)

    /// The error message string.
    pub fn get(&self) -> handle::Local<value::String> {
        unsafe { handle::Local::new(self.0.Get()) }
    }

    /// The stack trace to the point where the error was generated.
    pub fn get_stack_trace(&self) -> handle::Local<StackTrace> {
        unsafe { handle::Local::new(self.0.GetStackTrace()) }
    }

    pub unsafe fn from_raw(raw: v8_sys::Message) -> Message {
        Message(raw)
    }
}

impl StackTrace {
    /// The stack frames that this stack trace consists of.
    pub fn get_frames(&self) -> Vec<handle::Local<StackFrame>> {
        let count = unsafe { self.0.GetFrameCount() };
        let mut result = Vec::with_capacity(count as usize);

        for i in 0..count {
            result.push(unsafe { self.0.GetFrame(i) });
        }

        unsafe { result }
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
            self.0.GetLineNumber() as u32
        }
    }

    /// The column number at which this stack frame was pushed.
    pub fn get_column(&self) -> u32 {
        unsafe { self.0.GetColumn() as u32 }
    }

    /// The script file name in which this stack frame was pushed.
    pub fn get_script_name(&self) -> Option<handle::Local<value::String>> {
        unsafe {
            let raw = self.0.GetScriptName();
            if raw.is_null() {
                None
            } else {
                Some(handle::Local::new(raw))
            }
        }
    }

    /// The function name in which this stack frame was pushed.
    pub fn get_function_name(&self) -> handle::Local<value::String> {
        unsafe {
            handle::Local::new(self.0.GetFunctionName())
        }
    }

    /// Whether this stack frame is part of an eval call.
    pub fn is_eval(&self) -> bool {
        unsafe { self.0.IsEval() }
    }

    /// Whether this stack frame is part of a constructor call.
    pub fn is_constructor(&self) -> bool {
        unsafe { self.0.IsConstructor() }
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
            writeln!(f, "{}", frame)?;
        }
        Ok(())
    }
}

impl fmt::Display for CapturedStackFrame {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "    at ")?;

        if self.is_constructor {
            write!(f, "new ")?;
        }

        if let Some(ref function_name) = self.function_name {
            write!(f, "{} (", function_name)?;

            if self.is_eval {
                write!(f, "eval ")?;
            }

            write!(f,
                   "{}:{}:{})",
                   self.script_name.as_ref().map(|n| n.as_str()).unwrap_or("<anon>"),
                   self.line,
                   self.column)?;
        } else {
            if self.is_eval {
                write!(f, "eval ")?;
            }
            write!(f,
                   "{}:{}:{}",
                   self.script_name.as_ref().map(|n| n.as_str()).unwrap_or("<anon>"),
                   self.line,
                   self.column)?;
        }

        Ok(())
    }
}
