//! Low-level interface to a GDB session.

use super::{
    grammar::parse_gdbmi_record,
    raw_output::{Record, ResultRecord},
    result::{BadResponse, Result},
};

/// Low level interface to GDB that communicates using literal strings.
pub trait StringGdbMiStream {
    /// Sends an MI command to GDB.
    ///
    /// The command must be valid in the
    /// [GDB/MI input syntax](https://sourceware.org/gdb/current/onlinedocs/gdb.html/GDB_002fMI-Input-Syntax.html).
    ///
    /// The returned string is the result record (starting with `^`)
    /// that responds to the passed command.
    fn send_command(&mut self, command: &str) -> impl Future<Output = std::io::Result<String>>;

    /// Shorthand for constructing a command with formatting.
    ///
    /// See [`StringGdbMiStream::send_command`] for information about how this function should be used.
    fn send_command_fmt(
        &mut self,
        args: std::fmt::Arguments<'_>,
    ) -> impl Future<Output = std::io::Result<String>> {
        async move { self.send_command(&std::fmt::format(args)).await }
    }
}

/// Low level interface to GDB that responds with parsed result records.
pub trait GdbMiStream {
    /// Sends an MI command to GDB.
    ///
    /// The command must be valid in the
    /// [GDB/MI input syntax](https://sourceware.org/gdb/current/onlinedocs/gdb.html/GDB_002fMI-Input-Syntax.html).
    ///
    /// The returned string is the result record (starting with `^`)
    /// that responds to the passed command.
    fn send_command(&mut self, command: &str) -> impl Future<Output = Result<ResultRecord>>;

    /// Shorthand for constructing a command with formatting.
    ///
    /// See [`GdbMiStream::send_command`] for information about how this function should be used.
    fn send_command_fmt(
        &mut self,
        args: std::fmt::Arguments<'_>,
    ) -> impl Future<Output = Result<ResultRecord>> {
        async move { self.send_command(&std::fmt::format(args)).await }
    }
}

impl<T: StringGdbMiStream> GdbMiStream for T {
    async fn send_command(&mut self, command: &str) -> Result<ResultRecord> {
        let output = StringGdbMiStream::send_command(self, command).await?;
        match parse_gdbmi_record(&output) {
            Ok(Record::Result(r)) => Ok(r),
            _ => Err(BadResponse::SyntaxError(output).into()),
        }
    }

    async fn send_command_fmt(&mut self, args: std::fmt::Arguments<'_>) -> Result<ResultRecord> {
        let output = StringGdbMiStream::send_command_fmt(self, args).await?;
        match parse_gdbmi_record(&output) {
            Ok(Record::Result(r)) => Ok(r),
            _ => Err(BadResponse::SyntaxError(output).into()),
        }
    }
}
