//! Interoperability with the
//! [GDB Machine Interface](https://sourceware.org/gdb/current/onlinedocs/gdb.html/GDB_002fMI.html)

pub mod grammar;
mod parsing;
pub mod raw_output;
pub mod result;
pub mod session;
pub mod stream;
pub mod types;
