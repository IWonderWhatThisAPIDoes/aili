//! Testing implementation of [`GdbMiStream`] that communicates
//! with the debugger synchronously.

use super::externals::gdb_path;
use aili_gdbstate::gdbmi::{
    grammar::parse_gdbmi_record,
    raw_output::{Record, ResultRecord},
    result::{BadResponse, Result},
    stream::GdbMiStream,
};
use std::{
    io::{BufRead, BufReader, Write},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
};

pub struct TestGdbMi {
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl TestGdbMi {
    pub fn new(executable_path: impl AsRef<std::ffi::OsStr>) -> Result<Self> {
        let mut instance = Self::construct_new(executable_path)?;
        instance.read_output_section()?; // GDB prints a banner first
        instance.send_command("-exec-run --start")?;
        instance
            .read_output_section_with_result()?
            .must_be_done_or_running()?;
        instance.read_output_section()?; // Wait for it to pause
        Ok(instance)
    }

    fn construct_new(executable_path: impl AsRef<std::ffi::OsStr>) -> Result<Self> {
        let mut gdb = Self::spawn_gdb(executable_path)?;
        let stdin = gdb
            .stdin
            .take()
            .expect("GDB was spawned with piped IO, so file descriptors should exist");
        let stdout = gdb
            .stdout
            .take()
            .expect("GDB was spawned with piped IO, so file descriptors should exist");
        Ok(Self {
            stdin,
            stdout: BufReader::new(stdout),
        })
    }

    fn spawn_gdb(executable_path: impl AsRef<std::ffi::OsStr>) -> std::io::Result<Child> {
        Command::new(gdb_path())
            .arg(executable_path)
            .arg("--interpreter=mi")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
    }

    fn send_command(&mut self, command: &str) -> std::io::Result<()> {
        self.send_command_fmt(format_args!("{command}"))
    }

    fn send_command_fmt(&mut self, args: std::fmt::Arguments) -> std::io::Result<()> {
        self.stdin.write_fmt(args)?;
        self.stdin.write_all(b"\r\n")?;
        self.stdin.flush()?;
        Ok(())
    }

    fn read_output_line(&mut self) -> std::io::Result<String> {
        let mut line = String::new();
        self.stdout.read_line(&mut line)?;
        Ok(line)
    }

    fn read_output_section(&mut self) -> std::io::Result<Option<String>> {
        let mut result_record = None;
        loop {
            let line = self.read_output_line()?;
            if line.trim() == Self::OUTPUT_SECTION_END {
                break;
            } else if has_result_record_prefix(&line) {
                result_record = Some(line);
            }
        }
        Ok(result_record)
    }

    fn read_output_section_with_result(&mut self) -> Result<ResultRecord> {
        let Some(result_record_line) = self.read_output_section()? else {
            return Err(BadResponse::MissingResultRecord.into());
        };
        let Ok(Record::Result(result_record)) = parse_gdbmi_record(&result_record_line) else {
            return Err(BadResponse::SyntaxError(result_record_line).into());
        };
        Ok(result_record)
    }

    pub fn run_to_line(&mut self, line: usize) -> Result<()> {
        self.send_command_fmt(format_args!("-break-insert -t {line}"))?;
        self.read_output_section_with_result()?
            .must_be_done_or_running()?;
        self.send_command("-exec-continue")?;
        self.read_output_section_with_result()?
            .must_be_done_or_running()?; // GDB will tell us it ran
        self.read_output_section()?; // This output should be generated when it stops
        Ok(())
    }

    const OUTPUT_SECTION_END: &str = "(gdb)";
}

impl GdbMiStream for TestGdbMi {
    async fn send_command(&mut self, command: &str) -> Result<ResultRecord> {
        TestGdbMi::send_command(self, command)?;
        self.read_output_section_with_result()
    }
    async fn send_command_fmt(&mut self, args: std::fmt::Arguments<'_>) -> Result<ResultRecord> {
        TestGdbMi::send_command_fmt(self, args)?;
        self.read_output_section_with_result()
    }
}

impl Drop for TestGdbMi {
    fn drop(&mut self) {
        let _ = self.send_command("-gdb-exit");
    }
}

fn has_result_record_prefix(line: &str) -> bool {
    line.chars().find(|c| !c.is_ascii_digit()) == Some('^')
}
