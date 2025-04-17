//! Error and result types returned by operations on GDB.

use derive_more::{Display, Error, From};

/// Result type for operations that involve communication with GDB.
pub type Result<T> = std::result::Result<T, Error>;

/// Describes an error in communication with GDB.
#[derive(Debug, Display, Error, From)]
pub enum Error {
    /// IO error.
    #[display("io error: {_0}")]
    IOError(std::io::Error),

    /// GDB has responded with an error.
    #[display("error response from gdb: {_0}")]
    #[error(ignore)]
    ErrorResponse(ErrorResponse),

    /// GDB has returned a malformed or unexpected response,
    /// or no response at all.
    #[display("gdb returned unexpected response: {_0}")]
    BadResponse(BadResponse),
}

/// Describes an error in processing a response returned by GDB.
///
/// This usualy indicates an incorrect expectation set by the calling code,
/// or it could be a bug on the side of GDB.
#[derive(Debug, Display, Error)]
#[error(ignore)]
pub enum BadResponse {
    /// Response was not a valid GDB/MI output.
    #[display("failed to parse record: {_0:?}")]
    SyntaxError(String),

    /// Response should have contained a result record, but it did not.
    #[display("response did not contain a result record")]
    MissingResultRecord,

    /// Result record of the response has an unexpected result class.
    #[display("unexpected result class: {_0}")]
    UnexpectedResultClass(String),

    /// Response payload was missing an expected property.
    #[display("missing property: {_0}")]
    MissingKey(String),

    /// A property in a response payload had a different type than expected.
    #[display("property does not have the expected type")]
    BadValueType,

    /// A property in a response payload did not have the correct value.
    ///
    /// This is usualy atomic (string) values that are documented to be numeric,
    /// but could not be parsed as such.
    #[display("property has unexpected value {_0:?}")]
    BadValue(String),
}

/// Result record from GDB that indicates an error.
#[derive(Debug)]
pub struct ErrorResponse {
    /// Message that describes the error, if provided.
    pub msg: Option<String>,
}

impl std::fmt::Display for ErrorResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(msg) = &self.msg {
            write!(f, "{msg}")
        } else {
            write!(f, "(no description provided)")
        }
    }
}
