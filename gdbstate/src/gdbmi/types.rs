//! High-level data structures that represents data payloads returned by GDB.

use derive_more::Display;

/// Single symbol in the response to a
/// [symbol query command](https://sourceware.org/gdb/current/onlinedocs/gdb.html/GDB_002fMI-Symbol-Query.html).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Symbol {
    /// Line number where the symbol is declared.
    pub line: u64,

    /// Name of the symbol.
    pub name: String,

    /// Name of the type of the symbol.
    pub type_name: String,

    /// Full declaration of the symbol.
    pub description: String,
}

/// Section of the response to a
/// [symbol query command](https://sourceware.org/gdb/current/onlinedocs/gdb.html/GDB_002fMI-Symbol-Query.html)
/// associated with a source file.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SymbolFile {
    /// Short name of the source file.
    pub filename: String,

    /// Full path to the source file.
    pub fullname: String,

    /// Symbols found in the file.
    pub symbols: Vec<Symbol>,
}

/// Description of a single stack frame in responses to some
/// [stack manipulation commands](https://sourceware.org/gdb/current/onlinedocs/gdb.html/GDB_002fMI-Stack-Manipulation.html).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct StackFrame {
    /// Zero-based index of the frame, topmost is zero.
    pub level: usize,

    /// Memory address of the frame.
    pub addr: u64,

    /// Name of the function that created the frame.
    pub func: String,

    /// Short name of the source file where the function lives.
    pub file: String,

    /// Full path to the source file where the function lives.
    pub fullname: String,

    /// Number of the line that the program is currently executing.
    pub line: u64,

    /// Name of the shared library where the function lives, if any.
    pub from: Option<String>,

    /// Name of architecture for which the function is compiled.
    pub arch: String,
}

/// Description of a local variable in responses to some
/// [stack manipulation commands](https://sourceware.org/gdb/current/onlinedocs/gdb.html/GDB_002fMI-Stack-Manipulation.html).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LocalVariable {
    /// Name of the variable.
    pub name: String,

    /// True if the variable is argument.
    ///
    /// Only returned by
    /// [-stack-list-variables](https://sourceware.org/gdb/current/onlinedocs/gdb.html/GDB_002fMI-Stack-Manipulation.html#The-_002dstack_002dlist_002dvariables-Command).
    pub arg: bool,

    /// Current value of the variable, if requested based on a parameter
    /// of type [`PrintValues`].
    pub value: Option<String>,
}

/// Used by some [stack manipulation commands](https://sourceware.org/gdb/current/onlinedocs/gdb.html/GDB_002fMI-Stack-Manipulation.html),
/// specifies which [`LocalVariable`] entries should include a [`LocalVariable::value`].
#[derive(Display)]
pub enum PrintValues {
    /// No values should be returned.
    #[display("0")]
    NoValues,

    /// Values should be returned for all variables.
    #[display("1")]
    AllValues,

    /// Values should be returned for elementary variables,
    /// not arrays or structures.
    #[display("2")]
    SimpleValues,
}

/// Full description of a [`VariableObject`].
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct VariableObjectData {
    /// Handle to the variable object.
    pub object: VariableObject,

    /// Current value of the associated variable, if requested based
    /// on a parameter of type [`PrintValues`].
    pub value: Option<String>,

    /// Name of the type of the associated variable.
    pub type_name: String,

    /// How many children the object is known to have.
    ///
    /// This number is only reliable if [`VariableObjectData::dynamic`] is false.
    pub numchild: usize,

    /// True if the variable object is dynamic.
    ///
    /// [`VariableObjectData::numchild`] of a dynamic variable object
    /// may not include all of its children, see [`VariableObjectData::has_more`].
    ///
    /// GDB does not return dynamic variable objects by default
    /// and they have to be explicitly enabled.
    pub dynamic: bool,

    /// True if the variable object is dynamic and it may have more
    /// children than indicated by [`VariableObjectData::numchild`].
    pub has_more: bool,

    /// ID of the thread that the associated variable belongs to, if any.
    pub thread_id: Option<String>,
}

/// Update payload for a single [`VariableObject`] in response to
/// [-var-update](https://sourceware.org/gdb/current/onlinedocs/gdb.html/GDB_002fMI-Variable-Objects.html#The-_002dvar_002dupdate-Command).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct VariableObjectUpdate {
    /// Handle to the variable object.
    pub object: VariableObject,

    /// Current value of the associated variable, if requested based
    /// on a parameter of type [`PrintValues`].
    pub value: Option<String>,

    /// Whether the associated variable remains in scope.
    pub in_scope: InScope,

    /// New type name of the variable. Only present if the type has changed.
    pub new_type_name: Option<String>,

    /// New number of children. Only relevant for dynamic variable objects
    /// or if type of the variable can change.
    pub new_num_children: Option<usize>,

    /// True if the variable object is dynamic and it may have more
    /// children than indicated by [`VariableObjectData::numchild`].
    pub has_more: bool,

    /// True if the variable object is dynamic.
    ///
    /// [`VariableObjectData::numchild`] of a dynamic variable object
    /// may not include all of its children, see [`VariableObjectData::has_more`].
    ///
    /// GDB does not return dynamic variable objects by default
    /// and they have to be explicitly enabled.
    pub dynamic: bool,

    /// Newly created variable objects that correspond to newly added children
    /// of a dynamic variable object.
    pub new_children: Vec<ChildVariableObject>,
}

/// Scope status of a [`VariableObject`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum InScope {
    /// The associated variable remains in scope.
    True,

    /// The associated variable has gone out of scope, but remains valid.
    False,

    /// The associated variable is no longer valid, usualy because
    /// the debuggee has changed.
    Invalid,

    /// A different value than any of the above.
    ///
    /// Meaning of this value is unknown. It is included to keep compatibility
    /// as the [documentation](https://sourceware.org/gdb/current/onlinedocs/gdb.html/GDB_002fMI-Development-and-Front-Ends.html)
    /// warns that new values may be added.
    Other,
}

/// Handle to a [GDB/MI variable object](https://sourceware.org/gdb/current/onlinedocs/gdb.html/GDB_002fMI-Variable-Objects.html).
///
/// Internally, this is the name of the variable object.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub struct VariableObject(pub String);

/// Payload returned by [`-var-list-children`](https://sourceware.org/gdb/current/onlinedocs/gdb.html/GDB_002fMI-Variable-Objects.html#The-_002dvar_002dlist_002dchildren-Command)
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ChildList {
    /// Number of known children that are returned.
    pub numchild: usize,

    /// True if the variable object is dynamic and it may have more
    /// children than indicated by [`VariableObjectData::numchild`].
    pub has_more: bool,

    /// Data of the child variable objects.
    pub children: Vec<ChildVariableObject>,
}

/// Description of a single child of a [`VariableObject`].
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ChildVariableObject {
    /// General description of the variable object.
    pub variable_object: VariableObjectData,

    /// Short displayable expression of the variable object.
    ///
    /// This is intended to be a displayable name of the child,
    /// typically field name or array index.
    pub exp: String,
}

/// Specification of the stack frame where a variable object should live.
#[derive(Display)]
pub enum VariableObjectFrameContext {
    /// Stack frame specified by zero-based index, zero is topmost.
    #[display("{_0}")]
    Frame(usize),

    /// The stack frame that is currently selected.
    #[display("*")]
    CurrentFrame,

    /// Floating variable object.
    ///
    /// The object will be resolved and evaluated
    /// in context of the current frame every time
    /// it is invoked.
    #[display("@")]
    Floating,
}
