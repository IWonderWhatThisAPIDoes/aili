//! Common utilities for use in integration tests.

pub mod externals;
pub mod future;
pub mod gdbmi;

use externals::compile_c;
use gdbmi::TestGdbMi;

/// Compiles a C source and starts a GDB session targeting
/// the compiled executable.
pub fn gdb_from_source(source: &str) -> TestGdbMi {
    let executable = compile_c(source).expect("Compilation failed");
    TestGdbMi::new(executable).expect("Could not start GDB")
}
