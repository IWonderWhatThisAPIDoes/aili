//! Deserialized raw output data from GDB.
//!
//! Based on the following specification:
//! - [General syntax](https://sourceware.org/gdb/current/onlinedocs/gdb.html/GDB_002fMI-Output-Syntax.html)
//! - [Output records](https://sourceware.org/gdb/current/onlinedocs/gdb.html/GDB_002fMI-Output-Records.html)

use derive_more::{Display, From};

/// Class of a [result record](https://sourceware.org/gdb/current/onlinedocs/gdb.html/GDB_002fMI-Result-Records.html).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Display)]
pub enum ResultClass {
    /// Requested action was completed.
    #[display("done")]
    Done,

    /// Legacy alias for [`ResultClass::Done`].
    #[display("running")]
    Running,

    /// Debugger has connected to a remote target.
    #[display("connected")]
    Connected,

    /// Error response.
    #[display("error")]
    Error,

    /// The debugger has exited.
    #[display("exit")]
    Exit,
}

/// Class of an [async record](https://sourceware.org/gdb/current/onlinedocs/gdb.html/GDB_002fMI-Async-Records.html).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Display)]
pub enum AsyncExecClass {
    /// The debuggee has started executing.
    #[display("running")]
    Running,

    /// The debuggee has stopped executing.
    #[display("stopped")]
    Stopped,
}

/// A record in the output of GDB.
///
/// Currently, only [`ResultRecord`] and [`AsyncExecRecord`] are supported.
#[derive(Clone, PartialEq, Eq, Debug, From)]
pub enum Record {
    /// Result record.
    Result(ResultRecord),

    /// Async record.
    AsyncExec(AsyncExecRecord),
}

/// Full [async record](https://sourceware.org/gdb/current/onlinedocs/gdb.html/GDB_002fMI-Async-Records.html).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AsyncExecRecord {
    /// Class of the record.
    pub async_exec_class: AsyncExecClass,

    /// Payload data.
    pub results: ResultTuple,
}

/// Full [result record](https://sourceware.org/gdb/current/onlinedocs/gdb.html/GDB_002fMI-Result-Records.html).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ResultRecord {
    /// Token that was passed to the associated command, if any.
    pub token: Option<String>,

    /// Class of the record.
    pub result_class: ResultClass,

    /// Payload data.
    pub results: ResultTuple,
}

/// Data payload that contains named fields.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct ResultTuple(pub Vec<ResultEntry>);

/// Single entry of a [`ResultTuple`].
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ResultEntry {
    pub key: String,
    pub value: Value,
}

/// Any value in a GDB/MI payload.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Value {
    /// String literal.
    Const(String),

    /// Structured data with named fields.
    Tuple(ResultTuple),

    /// List with named items.
    ///
    /// Names of the items are usualy all the same,
    /// so they may be ignored.
    TupleList(ResultTuple),

    /// List that contains a sequence of values.
    List(Vec<Value>),
}

impl Value {
    /// Extracts a [`Value::Const`] from the value, if it is present.
    pub fn into_const(self) -> Option<String> {
        match self {
            Self::Const(s) => Some(s),
            _ => None,
        }
    }

    /// Extracts a [`Value::Tuple`] or [`Value::TupleList`] from the value, if it is present.
    pub fn into_tuple(self) -> Option<ResultTuple> {
        match self {
            Self::Tuple(t) => Some(t),
            Self::TupleList(t) => Some(t),
            _ => None,
        }
    }

    /// Extracts a [`Value::List`] from the value, if it is present.
    pub fn into_list(self) -> Option<Vec<Value>> {
        match self {
            Self::List(l) => Some(l),
            _ => None,
        }
    }
}
