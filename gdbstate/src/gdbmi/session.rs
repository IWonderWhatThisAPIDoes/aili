//! High-level interface to a GDB session
//! that directly exposes commands known to GDB.

use super::{
    raw_output::*,
    result::{BadResponse, ErrorResponse, Result},
    stream::GdbMiStream,
    types::*,
};

/// Exposes relevant commands from the GDB/MI API.
///
/// Detailed documentation of the GDB/MI API can be found
/// [here](https://sourceware.org/gdb/current/onlinedocs/gdb.html/GDB_002fMI.html).
pub trait GdbMiSession {
    /// Exposes the
    /// [`-symbol-info-variables`](https://sourceware.org/gdb/current/onlinedocs/gdb.html/GDB_002fMI-Symbol-Query.html#The-_002dsymbol_002dinfo_002dvariables-Command)
    /// command.
    fn symbol_info_variables(&mut self) -> impl Future<Output = Result<Vec<SymbolFile>>>;

    /// Exposes the
    /// [`-symbol-info-functions`](https://sourceware.org/gdb/current/onlinedocs/gdb.html/GDB_002fMI-Symbol-Query.html#The-_002dsymbol_002dinfo_002dfunctions-Command)
    /// command.
    fn symbol_info_functions(&mut self) -> impl Future<Output = Result<Vec<SymbolFile>>>;

    /// Exposes the
    /// [`-stack-info-depth`](https://sourceware.org/gdb/current/onlinedocs/gdb.html/GDB_002fMI-Stack-Manipulation.html#The-_002dstack_002dinfo_002ddepth-Command)
    /// command.
    fn stack_info_depth(&mut self) -> impl Future<Output = Result<usize>>;

    /// Exposes the
    /// [`-stack-select-frame`](https://sourceware.org/gdb/current/onlinedocs/gdb.html/GDB_002fMI-Stack-Manipulation.html#The-_002dstack_002dselect_002dframe-Command)
    /// command.
    fn stack_select_frame(&mut self, target_frame: usize) -> impl Future<Output = Result<()>>;

    /// Exposes the
    /// [`-stack-list-frames`](https://sourceware.org/gdb/current/onlinedocs/gdb.html/GDB_002fMI-Stack-Manipulation.html#The-_002dstack_002dlist_002dframes-Command)
    /// command.
    fn stack_list_frames(&mut self) -> impl Future<Output = Result<Vec<StackFrame>>>;

    /// Exposes the
    /// [`-stack-list-frames`](https://sourceware.org/gdb/current/onlinedocs/gdb.html/GDB_002fMI-Stack-Manipulation.html#The-_002dstack_002dlist_002dframes-Command)
    /// command with frame index bounds.
    fn stack_list_frames_bounded(
        &mut self,
        bounds: std::ops::Range<usize>,
    ) -> impl Future<Output = Result<Vec<StackFrame>>>;

    /// Exposes the
    /// [`-stack-list-variables`](https://sourceware.org/gdb/current/onlinedocs/gdb.html/GDB_002fMI-Stack-Manipulation.html#The-_002dstack_002dlist_002dvariables-Command)
    /// command.
    fn stack_list_variables(
        &mut self,
        print_values: PrintValues,
        skip_unavailable: bool,
    ) -> impl Future<Output = Result<Vec<LocalVariable>>>;

    /// Exposes the
    /// [`-var-create`](https://sourceware.org/gdb/current/onlinedocs/gdb.html/GDB_002fMI-Variable-Objects.html#The-_002dvar_002dcreate-Command)
    /// command.
    fn var_create(
        &mut self,
        frame: VariableObjectFrameContext,
        expression: &str,
    ) -> impl Future<Output = Result<VariableObjectData>>;

    /// Exposes the
    /// [`-var-delete`](https://sourceware.org/gdb/current/onlinedocs/gdb.html/GDB_002fMI-Variable-Objects.html#The-_002dvar_002ddelete-Command)
    /// command.
    fn var_delete(&mut self, object: &VariableObject) -> impl Future<Output = Result<()>>;

    /// Exposes the
    /// [`-var-evaluate-expression`](https://sourceware.org/gdb/current/onlinedocs/gdb.html/GDB_002fMI-Variable-Objects.html#The-_002dvar_002devaluate_002dexpression-Command)
    /// command.
    fn var_evaluate_expression(
        &mut self,
        object: &VariableObject,
    ) -> impl Future<Output = Result<String>>;

    /// Exposes the
    /// [`-var-list-children`](https://sourceware.org/gdb/current/onlinedocs/gdb.html/GDB_002fMI-Variable-Objects.html#The-_002dvar_002dlist_002dchildren-Command)
    /// command.
    fn var_list_children(
        &mut self,
        object: &VariableObject,
        print_values: PrintValues,
    ) -> impl Future<Output = Result<ChildList>>;

    /// Exposes the
    /// [`-var-update`](https://sourceware.org/gdb/current/onlinedocs/gdb.html/GDB_002fMI-Variable-Objects.html#The-_002dvar_002dupdate-Command)
    /// command.
    fn var_update(
        &mut self,
        print_values: PrintValues,
    ) -> impl Future<Output = Result<Vec<VariableObjectUpdate>>>;

    /// Exposes the
    /// [`-data-evaluate-expression`](https://sourceware.org/gdb/current/onlinedocs/gdb.html/GDB_002fMI-Data-Manipulation.html#The-_002ddata_002devaluate_002dexpression-Command)
    /// command.
    fn data_evaluate_expression(
        &mut self,
        expression: &str,
    ) -> impl Future<Output = Result<String>>;
}

impl<T: GdbMiStream> GdbMiSession for T {
    async fn symbol_info_variables(&mut self) -> Result<Vec<SymbolFile>> {
        Ok(self
            .send_command("-symbol-info-variables")
            .await?
            .must_be_done_or_running()?
            .take("symbols")?
            .tuple()?
            .take("debug")?
            .symbol_query_result()?)
    }

    async fn symbol_info_functions(&mut self) -> Result<Vec<SymbolFile>> {
        Ok(self
            .send_command("-symbol-info-functions")
            .await?
            .must_be_done_or_running()?
            .take("symbols")?
            .tuple()?
            .take("debug")?
            .symbol_query_result()?)
    }

    async fn stack_info_depth(&mut self) -> Result<usize> {
        Ok(self
            .send_command("-stack-info-depth")
            .await?
            .must_be_done_or_running()?
            .take("depth")?
            .decimal()?)
    }

    async fn stack_select_frame(&mut self, target_frame: usize) -> Result<()> {
        self.send_command_fmt(format_args!("-stack-select-frame {target_frame}"))
            .await?
            .must_be_done_or_running()?;
        Ok(())
    }

    async fn stack_list_frames(&mut self) -> Result<Vec<StackFrame>> {
        Ok(self
            .send_command("-stack-list-frames")
            .await?
            .must_be_done_or_running()?
            .take("stack")?
            .stack_trace()?)
    }

    async fn stack_list_frames_bounded(
        &mut self,
        bounds: std::ops::Range<usize>,
    ) -> Result<Vec<StackFrame>> {
        Ok(self
            .send_command_fmt(format_args!(
                "-stack-list-frames {} {}",
                bounds.start, bounds.end
            ))
            .await?
            .must_be_done_or_running()?
            .take("stack")?
            .stack_trace()?)
    }

    async fn stack_list_variables(
        &mut self,
        print_values: PrintValues,
        skip_unavailable: bool,
    ) -> Result<Vec<LocalVariable>> {
        let skip_arg = if skip_unavailable {
            "--skip-unavailable"
        } else {
            ""
        };
        Ok(self
            .send_command_fmt(format_args!(
                "-stack-list-variables {skip_arg} {print_values}"
            ))
            .await?
            .must_be_done_or_running()?
            .take("variables")?
            .local_variable_list()?)
    }

    async fn var_create(
        &mut self,
        frame: VariableObjectFrameContext,
        expression: &str,
    ) -> Result<VariableObjectData> {
        Ok(self
            .send_command_fmt(format_args!("-var-create - {frame} {expression}"))
            .await?
            .must_be_done_or_running()?
            .var_object()?)
    }

    async fn var_delete(&mut self, object: &VariableObject) -> Result<()> {
        self.send_command_fmt(format_args!("-var-delete {}", object.0))
            .await?
            .must_be_done_or_running()?;
        Ok(())
    }

    async fn var_evaluate_expression(&mut self, object: &VariableObject) -> Result<String> {
        Ok(self
            .send_command_fmt(format_args!("-var-evaluate-expression {}", object.0))
            .await?
            .must_be_done_or_running()?
            .take("value")?
            .string()?)
    }

    async fn var_list_children(
        &mut self,
        object: &VariableObject,
        print_values: PrintValues,
    ) -> Result<ChildList> {
        Ok(self
            .send_command_fmt(format_args!(
                "-var-list-children {print_values} {}",
                object.0
            ))
            .await?
            .must_be_done_or_running()?
            .child_list()?)
    }

    async fn var_update(&mut self, print_values: PrintValues) -> Result<Vec<VariableObjectUpdate>> {
        Ok(self
            .send_command_fmt(format_args!("-var-update {print_values} *"))
            .await?
            .must_be_done_or_running()?
            .take("changelist")?
            .varobj_changelist()?)
    }

    async fn data_evaluate_expression(&mut self, expression: &str) -> Result<String> {
        Ok(self
            .send_command_fmt(format_args!("-data-evaluate-expression {expression}"))
            .await?
            .must_be_done_or_running()?
            .take("value")?
            .string()?)
    }
}

impl ResultRecord {
    pub fn must_be_done_or_running(mut self) -> Result<ResultTuple> {
        if self.result_class == ResultClass::Error {
            let msg = self.results.take("msg").and_then(Value::string).ok();
            return Err(ErrorResponse { msg }.into());
        }
        if self.result_class != ResultClass::Done && self.result_class != ResultClass::Running {
            return Err(BadResponse::UnexpectedResultClass(self.result_class.to_string()).into());
        }
        Ok(self.results)
    }
}
