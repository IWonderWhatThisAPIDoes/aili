//! Use of external processes in tests.

use derive_more::{Display, Error, From};
use std::{
    hash::{DefaultHasher, Hash, Hasher},
    io::Write,
    path::PathBuf,
    process::{Command, ExitStatus, Stdio},
    sync::LazyLock,
};

/// Exposes a value that is loaded from an environment variable
/// or a default value is used.
macro_rules! lazy_env_or_default {
    ( $( #[ $attr:meta ] )* $vis:vis $get:ident = $env:literal | $default:expr $(;)? ) => {
        $( #[ $attr ] )*
        $vis fn $get() -> &'static str {
            fn construct() -> String {
                std::env::var($env).unwrap_or_else(|_| $default.to_string())
            }
            static LAZY_VAL: LazyLock<String> = LazyLock::new(construct);
            &LAZY_VAL
        }
    };
}

lazy_env_or_default! {
    /// Path to the GDB executable.
    pub gdb_path = "GDB_PATH" | "gdb";
}
lazy_env_or_default! {
    /// Path to a C compiler executable.
    cc_path = "CC_PATH" | "gcc";
}

/// Builds a hex string from the hash of a value.
fn hex_hash<T: Hash + ?Sized>(x: &T) -> String {
    let mut hasher = DefaultHasher::new();
    Hash::hash(x, &mut hasher);
    let hash = hasher.finish();
    format!("{hash:x}")
}

/// Gets the path to a temorary directory for storing test artifacts
/// and creates the directory if needed.
fn temporary_directory() -> Result<PathBuf, std::io::Error> {
    let tmp = std::env::temp_dir();
    let pid = std::process::id();
    let my_tmp_dir = tmp.join(pid.to_string());
    if let Err(err) = std::fs::create_dir(&my_tmp_dir) {
        // If the path already exists and is a directory, then it is actually fine
        // Otherwise we fail with the returned error
        if !my_tmp_dir.is_dir() {
            return Err(err);
        }
    }
    Ok(my_tmp_dir)
}

/// Compiles a C source to an executable in a temporary directory
/// and returns the path to the executable.
pub fn compile_c(source: &str) -> Result<PathBuf, CompileError> {
    let my_tmp_binary = temporary_directory()?.join(hex_hash(&source));
    let mut cc = Command::new(cc_path())
        .arg("-o")
        .arg(&my_tmp_binary)
        .arg("-ggdb")
        .arg("-x")
        .arg("c")
        .arg("-")
        .stdin(Stdio::piped())
        .spawn()?;
    let mut stdin = cc
        .stdin
        .take()
        .expect("Process was created with piped io, so file descriptors should be available");
    stdin.write_all(source.as_bytes())?;
    drop(stdin); // Close the file descriptor
    let status = cc.wait()?;
    if status.success() {
        Ok(my_tmp_binary)
    } else {
        Err(status.into())
    }
}

/// Indicates a failure while compiling a C source.
#[derive(From, Debug, Display, Error)]
pub enum CompileError {
    /// Failed due to an IO error.
    IOError(std::io::Error),

    /// C compiler rejected the source.
    #[display("C compiler exited with status {_0}")]
    #[error(ignore)]
    ErrorStatus(ExitStatus),
}
