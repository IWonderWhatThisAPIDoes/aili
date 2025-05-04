//! Construction of a [`GdbStateGraph`] using a [`GdbMiSession`].

use crate::{
    gdbmi::{result::Result, session::GdbMiSession, types::*},
    state::*,
};
use aili_model::state::*;
use derive_more::Debug;
use regex::Regex;
use std::{
    collections::{BTreeMap, HashMap},
    sync::LazyLock,
};

impl GdbStateGraph {
    /// Constructs a state graph that only consists of the root node
    pub fn empty() -> Self {
        Self {
            root_node: GdbStateNode::new(NodeTypeClass::Root),
            stack_trace: Vec::new(),
            variables: HashMap::new(),
            length_nodes: HashMap::new(),
            address_mapping: BTreeMap::new(),
        }
    }

    /// Constructs a new state graph using a provided GDB session.
    ///
    /// This function sends commands to GDB and awaits responses
    /// asynchronously.
    pub async fn new(gdb: &mut impl GdbMiSession) -> Result<Self> {
        let mut graph = Self::empty();
        graph.update_stack_trace(gdb).await?;
        Ok(graph)
    }

    /// Updates an existing state graph using a provided GDB session.
    ///
    /// It is assumed that it is the same session that was passed
    /// to [`GdbStateGraph::new`] in order to recude the number
    /// of commands that need to be invoked. Modifying the session
    /// in between calls can yield unexpected results.
    pub async fn update(&mut self, gdb: &mut impl GdbMiSession) -> Result<()> {
        self.update_variable_objects(gdb).await?;
        self.update_stack_trace(gdb).await?;
        Ok(())
    }

    /// Erases all variable objects associated with this state graph
    /// from the provided GDB session.
    pub async fn drop_variable_objects(&self, gdb: &mut impl GdbMiSession) -> Result<()> {
        for (variable_handle, node) in &self.variables {
            // Only top level nodes need to be deleted,
            // the rest will be cleaned up by GDB recursively
            if node.is_top_level() {
                // TODO: Better error handling; only some errors may be ignored
                let _ = gdb.var_delete(variable_handle).await;
            }
        }
        Ok(())
    }

    async fn update_variable_objects(&mut self, gdb: &mut impl GdbMiSession) -> Result<()> {
        let changelist = gdb.var_update(PrintValues::SimpleValues).await?;
        for change in &changelist {
            self.update_variable_object(change, gdb).await?;
        }
        Ok(())
    }

    async fn update_variable_object(
        &mut self,
        var_object: &VariableObjectUpdate,
        gdb: &mut impl GdbMiSession,
    ) -> Result<()> {
        if var_object.dynamic {
            // TODO: Warn
            // Dynamic variable objects should never be returned by GDB unless explicitly enabled
        }
        if var_object.new_type_name.is_some() {
            // TODO: Warn
        }
        if var_object.in_scope != InScope::True {
            self.variable_object_out_of_scope(&var_object.object, gdb)
                .await?;
        } else if let Some(variable) = self.variables.get_mut(&var_object.object) {
            // Otherwise, the value must have changed, so reevaluate it
            variable.value = var_object.value.as_deref().and_then(Self::parse_node_value);
            // If the variable is a pointer, update its dereference
            if variable.type_class == NodeTypeClass::Ref {
                variable.remove_successor(&EdgeLabel::Deref);
                if let (Some(NodeValue::Uint(address)), Some(type_name)) =
                    (variable.value, variable.type_name.clone())
                {
                    if gdb
                        .data_evaluate_expression(&format!("*(char*){address}"))
                        .await
                        .is_ok()
                    {
                        let deref_var_object = Box::pin(
                            self.get_or_create_dereference_variable_node(gdb, address, &type_name),
                        )
                        .await?;
                        self.variables
                            .get_mut(&var_object.object)
                            .expect("The node was just created")
                            .successors
                            .push((
                                EdgeLabel::Deref,
                                GdbStateNodeId::VarObject(deref_var_object),
                            ));
                    }
                }
            }
        }
        // If we do not know about the object, someone else must have
        // created it in the session, so we ignore it
        Ok(())
    }

    async fn variable_object_out_of_scope(
        &mut self,
        var_object: &VariableObject,
        gdb: &mut impl GdbMiSession,
    ) -> Result<()> {
        // The variable has gone out of scope, so we destroy it
        let parent_node = self.remove_variables_recursive(var_object);
        // Remove the reference to it from its parent frame
        if let Some(GdbStateNodeId::Frame(frame_index)) = parent_node {
            if let Some(frame) = self.stack_trace.get_mut(frame_index) {
                frame.remove_successor_by_id(&GdbStateNodeId::VarObject(var_object.clone()));
            }
        } else {
            // Only local variables can go out of scope
            // TODO: warn
        }
        gdb.var_delete(var_object).await?;
        Ok(())
    }

    fn remove_variables_recursive(&mut self, handle: &VariableObject) -> Option<GdbStateNodeId> {
        if let Some(node) = self.variables.remove(handle) {
            for (edge_label, next_object) in node.node.successors {
                match edge_label {
                    // These edges are what one would reasonably expect here
                    EdgeLabel::Named(_, _) | EdgeLabel::Index(_) | EdgeLabel::Length => {}
                    // Leave dereference edges be, they go by different rules
                    // (most notably, they are not a tree, and we do not want
                    // a dangling reference here)
                    EdgeLabel::Deref => continue,
                    // These edges cannot go from a variable node,
                    // so we emit a warning if it ever happens
                    EdgeLabel::Main | EdgeLabel::Next | EdgeLabel::Result => {
                        // TODO: Warn
                        continue;
                    }
                }
                match next_object {
                    GdbStateNodeId::Root | GdbStateNodeId::Frame(_) => {
                        // TODO: Warn
                    }
                    GdbStateNodeId::VarObject(v) => {
                        self.remove_variables_recursive(&v);
                    }
                    GdbStateNodeId::Length(v) => {
                        self.length_nodes.remove(&v);
                    }
                }
            }
            node.parent
        } else {
            None
        }
    }

    async fn update_stack_trace(&mut self, gdb: &mut impl GdbMiSession) -> Result<()> {
        let stack_trace = gdb.stack_list_frames().await?;
        // There is no way to tell if the top stack frame has
        // returned and then the same function was called
        // (as opposed to still being in that function),
        // so this update is done on a best-effort basis.
        //
        // Traverse the stack from the bottom up and update
        // everything after the first frame that does not match
        // the cached state
        let update_index = self
            .stack_trace
            .iter()
            // Reverse the trace from GDB, it lists frames starting from the top
            .zip(stack_trace.iter().rev())
            .enumerate()
            // Find the first function that does not have the same name
            // Unwrap is safe here because all stack frame nodes have a name
            .find(|(_, (cached, new))| cached.type_name.as_deref().unwrap() != new.func)
            .map(|(i, _)| i)
            // If all available frames match, at least the frames that are only
            // cached but no longer reported by GDB (or vice versa) must be updated
            .unwrap_or(self.stack_trace.len().min(stack_trace.len()));
        // Drop all cached frames starting at the first different frame
        self.drop_stack_frames_after(update_index);
        // New variables may have come into scope at the topmost unchanged frame
        if update_index > 0 {
            gdb.stack_select_frame(stack_trace[stack_trace.len() - update_index].level)
                .await?;
            self.update_local_variables(update_index - 1, gdb).await?;
        }
        // Create new frames starting at the first different frame
        let frames_to_push = stack_trace.into_iter().rev().skip(update_index);
        self.push_stack_frames(frames_to_push, gdb).await?;
        Ok(())
    }

    async fn update_local_variables(
        &mut self,
        frame_index: usize,
        gdb: &mut impl GdbMiSession,
    ) -> Result<()> {
        let mut locals = gdb
            .stack_list_variables(PrintValues::NoValues, false)
            .await?;
        // Sort the output by name so that variables of the same name end up together
        locals.sort_by(|a, b| a.name.cmp(&b.name));
        let mut locals = locals.into_iter().peekable();
        // Go through all local variables
        while let Some(local) = locals.next() {
            let name = local.name;
            // How many variables of the same name are currently visible
            let mut overloads = 0;
            while locals.peek().is_some_and(|v| v.name == name) {
                overloads += 1;
                // Eat the other variables with this name, they do not have any useful information
                locals.next();
            }
            // We can only get one variable value, assume it is the one
            // with largest discriminator (the most recently declared one)
            let edge_id = EdgeLabel::Named(name.clone(), overloads);
            // Check that the parent (the stack frame node) knows about the variable
            let has_the_variable = self.stack_trace[frame_index]
                .successors
                .iter()
                .any(|(e, _)| *e == edge_id);
            // If the stack frame does not know about the variable, create it now
            if !has_the_variable {
                self.create_local_variable(gdb, frame_index, &name, edge_id)
                    .await?;
            }
            // TODO: Check that the stack knows about all shadowed variables as well,
            // and warn if it does not (they are not reachable from our current point)
        }
        Ok(())
    }

    async fn create_local_variable(
        &mut self,
        gdb: &mut impl GdbMiSession,
        frame_index: usize,
        name: &str,
        edge_label: EdgeLabel,
    ) -> Result<()> {
        let var_object = gdb
            .var_create(VariableObjectFrameContext::CurrentFrame, name)
            .await?;
        let id = self
            .create_variable_tree(gdb, var_object, Some(GdbStateNodeId::Frame(frame_index)))
            .await?;
        self.stack_trace[frame_index]
            .successors
            .push((edge_label, id));
        Ok(())
    }

    fn drop_stack_frames_after(&mut self, update_index: usize) {
        // Drop frames until there is the requested amount
        while self.stack_trace.len() > update_index {
            self.pop_stack_frame();
        }
        // Unlink the reference in the preceding node
        if update_index == 0 {
            self.root_node.remove_successor(&EdgeLabel::Main);
        } else {
            self.stack_trace[update_index - 1].remove_successor(&EdgeLabel::Next);
        }
    }

    /// Panics if the stack is empty
    fn pop_stack_frame(&mut self) {
        // Variable objects should be invalidated by GDB,
        // so we do not remove those manually
        self.stack_trace.pop().unwrap();
    }

    async fn push_stack_frames(
        &mut self,
        new_frames: impl IntoIterator<Item = StackFrame>,
        gdb: &mut impl GdbMiSession,
    ) -> Result<()> {
        for frame in new_frames {
            self.push_stack_frame(frame, gdb).await?;
        }
        Ok(())
    }

    async fn push_stack_frame(
        &mut self,
        frame: StackFrame,
        gdb: &mut impl GdbMiSession,
    ) -> Result<()> {
        // Get the esteemed index of the frame
        let frame_index = self.stack_trace.len();
        // Create the node and add it to the trace
        let mut frame_node = GdbStateNode::new(NodeTypeClass::Frame);
        frame_node.type_name = Some(frame.func);
        self.stack_trace.push(frame_node);
        // Link the frame to the previous one or to the root node
        if frame_index == 0 {
            self.root_node
                .successors
                .push((EdgeLabel::Main, GdbStateNodeId::Frame(0)));
        } else {
            self.stack_trace[frame_index - 1]
                .successors
                .push((EdgeLabel::Next, GdbStateNodeId::Frame(frame_index)));
        }
        // Populate all local variables
        gdb.stack_select_frame(frame.level).await?;
        self.update_local_variables(frame_index, gdb).await?;
        Ok(())
    }

    #[expect(unused)]
    async fn populate_global_variables(&mut self, gdb: &mut impl GdbMiSession) -> Result<()> {
        // Get all global variables across all files
        let query_result = gdb.symbol_info_variables().await?;
        for file in query_result {
            for symbol in file.symbols {
                self.create_global_variable(symbol, gdb).await?;
            }
        }
        Ok(())
    }

    async fn create_global_variable(
        &mut self,
        variable_symbol: Symbol,
        gdb: &mut impl GdbMiSession,
    ) -> Result<()> {
        let edge_name = variable_symbol.name.clone();
        // Create the node
        let id = self.read_global_variable_node(variable_symbol, gdb).await?;
        // Insert the node into root
        self.root_node.add_named_successor(edge_name, id);
        Ok(())
    }

    async fn read_global_variable_node(
        &mut self,
        variable_symbol: Symbol,
        gdb: &mut impl GdbMiSession,
    ) -> Result<GdbStateNodeId> {
        let var_object = gdb
            .var_create(
                VariableObjectFrameContext::CurrentFrame,
                &format!("::{}", variable_symbol.name),
            )
            .await?;
        self.create_variable_tree(gdb, var_object, Some(GdbStateNodeId::Root))
            .await
    }

    async fn create_variable_tree(
        &mut self,
        gdb: &mut impl GdbMiSession,
        var_object: VariableObjectData,
        parent: Option<GdbStateNodeId>,
    ) -> Result<GdbStateNodeId> {
        if var_object.dynamic {
            // TODO: Warn
            // Dynamic variable objects should never be returned by GDB unless explicitly enabled
        }
        let has_children = var_object.numchild > 0;
        let var_object_handle = var_object.object.clone();
        self.create_variable_node(var_object, parent);
        if has_children {
            // If there are children, now is the time to resolve them
            self.after_create_non_atom_variable_node(gdb, &var_object_handle)
                .await?;
        }
        Ok(GdbStateNodeId::VarObject(var_object_handle))
    }

    async fn after_create_non_atom_variable_node(
        &mut self,
        gdb: &mut impl GdbMiSession,
        var_object: &VariableObject,
    ) -> Result<()> {
        let node = self
            .variables
            .get_mut(var_object)
            .expect("The variable object must be mapped to a node");
        if node.value.is_some() {
            // If the value of the node parsed as an elementary value,
            // the node is a pointer
            node.type_class = NodeTypeClass::Ref;
            // Get the pointer's type name so we can cast properly
            let pointer_type_name = node.type_name.clone();
            // GDB will report a child no matter what, but if it's a null pointer,
            // it should not appear in the state graph
            let Some(NodeValue::Uint(address)) = node.value else {
                return Ok(());
            };
            if address == 0 {
                return Ok(());
            }
            let can_access_target_address = gdb
                .data_evaluate_expression(&format!("*(char*){address}"))
                .await
                .is_ok();
            if !can_access_target_address {
                return Ok(());
            }
            let Some(type_name) = pointer_type_name else {
                return Ok(());
            };
            let deref_var_object =
                Box::pin(self.get_or_create_dereference_variable_node(gdb, address, &type_name))
                    .await?;
            self.variables
                .get_mut(var_object)
                .expect("The node was just created")
                .successors
                .push((
                    EdgeLabel::Deref,
                    GdbStateNodeId::VarObject(deref_var_object),
                ));
        } else {
            let children = gdb
                .var_list_children(var_object, PrintValues::SimpleValues)
                .await?;
            let container_kind = ContainerKind::deduce_from_children(&children.children)
                .expect("We have just verified that the node has children; type must be deducible");
            let node = self
                .variables
                .get_mut(var_object)
                .expect("The node was just created");
            node.type_class = container_kind.into();
            match container_kind {
                ContainerKind::Struct => {
                    for child in children.children {
                        // Construct the full tree recursively
                        let child_node_id = Box::pin(self.create_variable_tree(
                            gdb,
                            child.variable_object,
                            Some(GdbStateNodeId::VarObject(var_object.clone())),
                        ))
                        .await?;
                        // Insert child into parent
                        self.variables
                            .get_mut(var_object)
                            .expect("The node was just created")
                            .add_named_successor(child.exp, child_node_id);
                    }
                }
                ContainerKind::Array => {
                    // Remove the node's type if it was given one, array nodes do not have types
                    node.type_name = None;
                    // Cache the full length of the array so we can insert is as a node later
                    let mut length = 0;
                    for child in children.children {
                        // Construct the full tree recursively
                        let child_node_id = Box::pin(self.create_variable_tree(
                            gdb,
                            child.variable_object,
                            Some(GdbStateNodeId::VarObject(var_object.clone())),
                        ))
                        .await?;
                        // Parse the variable's index
                        let Ok(index) = child.exp.parse() else {
                            // `ContainerKind::deduce_from_children` ensures that all
                            // children have numeric names, but the name may be too long
                            // to store in our variables
                            // TODO: warn
                            continue;
                        };
                        length = length.max(index + 1);
                        // Insert child into parent
                        self.variables
                            .get_mut(var_object)
                            .expect("The node was just created")
                            .successors
                            .push((EdgeLabel::Index(index), child_node_id));
                    }
                    // Insert the length node
                    let mut length_node = GdbStateNode::new(NodeTypeClass::Atom);
                    length_node.value = Some(NodeValue::Uint(length as u64));
                    self.length_nodes.insert(var_object.clone(), length_node);
                    self.variables
                        .get_mut(var_object)
                        .expect("The node was just created")
                        .successors
                        .push((
                            EdgeLabel::Length,
                            GdbStateNodeId::Length(var_object.clone()),
                        ));
                }
                ContainerKind::Pointer => unreachable!(),
            }
        }
        Ok(())
    }

    async fn get_or_create_dereference_variable_node(
        &mut self,
        gdb: &mut impl GdbMiSession,
        address: u64,
        pointer_type_name: &str,
    ) -> Result<VariableObject> {
        // If the node already exists, return it right away
        if let Some(var_object) = self.address_mapping.get(&address) {
            return Ok(var_object.clone());
        }
        let deref_var_object = gdb
            .var_create(
                VariableObjectFrameContext::CurrentFrame,
                &format!("*({pointer_type_name}){address}"),
            )
            .await?;
        let var_object = deref_var_object.object.clone();
        self.create_variable_tree(gdb, deref_var_object, None)
            .await?;
        self.address_mapping.insert(address, var_object.clone());
        Ok(var_object)
    }

    fn create_variable_node(
        &mut self,
        var_object: VariableObjectData,
        parent: Option<GdbStateNodeId>,
    ) {
        let node = self.new_variable_node(var_object.object, NodeTypeClass::Atom, parent);
        node.type_name = Some(Self::preprocess_type_name(var_object.type_name));
        node.value = var_object.value.as_deref().and_then(Self::parse_node_value);
    }

    fn new_variable_node(
        &mut self,
        id: VariableObject,
        type_class: NodeTypeClass,
        parent: Option<GdbStateNodeId>,
    ) -> &mut GdbStateNode {
        self.variables
            .entry(id)
            .insert_entry(GdbStateNodeForVariable::new(
                GdbStateNode::new(type_class),
                parent,
            ))
            .into_mut()
    }

    fn parse_node_value(mut s: &str) -> Option<NodeValue> {
        // GDB includes both numeric and character representation of chars
        // and char pointers, so we need to strip the character string
        static CHAR_VALUE_REGEX: LazyLock<Regex> = LazyLock::new(|| {
            Regex::new(r#"^([+\-]?(?:0[xX])?[\da-fA-F]+)\s*(?:'.*'|".*")$"#).unwrap()
        });
        if let Some(caps) = CHAR_VALUE_REGEX.captures(s) {
            s = caps.get(1).unwrap().as_str()
        }
        if let Ok(u) = s.parse() {
            // Parse it as unsigned decimal
            Some(NodeValue::Uint(u))
        } else if let Ok(i) = s.parse() {
            // Parse it as signed decimal
            Some(NodeValue::Int(i))
        } else if let Some(h) = s.strip_prefix("0x") {
            // Parse it as hexadecimal
            u64::from_str_radix(h, 16).ok().map(NodeValue::Uint)
        } else {
            // It's probably a struct or array, so do not include a value
            None
        }
    }

    fn preprocess_type_name(name: String) -> String {
        // Const keyword should not be apart of the type name
        let name = name
            .strip_prefix("const ")
            .map(str::to_owned)
            .unwrap_or(name);
        // This is C, so struct type names may include the struct keyword
        // We do not want that to be included, so we drop it if possible
        let name = name
            .strip_prefix("struct ")
            .map(str::to_owned)
            .unwrap_or(name);
        name
    }
}

impl GdbStateNode {
    fn new(type_class: NodeTypeClass) -> Self {
        Self {
            type_class,
            type_name: None,
            successors: Vec::new(),
            value: None,
        }
    }

    fn add_named_successor(&mut self, name: String, successor: GdbStateNodeId) {
        let existing_nodes_with_same_name = self
            .successors
            .iter()
            .filter(|(edge, _)| {
                if let EdgeLabel::Named(existing_name, _) = edge {
                    *existing_name == name
                } else {
                    false
                }
            })
            .count();
        let new_edge_label = EdgeLabel::Named(name, existing_nodes_with_same_name);
        self.successors.push((new_edge_label, successor));
    }

    fn remove_successor(&mut self, id: &EdgeLabel) -> Option<GdbStateNodeId> {
        let index = self
            .successors
            .iter()
            .enumerate()
            .find(|(_, (e, _))| e == id)?
            .0;
        Some(self.successors.swap_remove(index).1)
    }

    fn remove_successor_by_id(&mut self, id: &GdbStateNodeId) -> Option<EdgeLabel> {
        let index = self
            .successors
            .iter()
            .enumerate()
            .find(|(_, (_, v))| v == id)?
            .0;
        Some(self.successors.swap_remove(index).0)
    }
}

/// Enumerates categories of types that GDB reports as having child variables
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ContainerKind {
    /// Structure type, corresponds to [`NodeTypeClass::Struct`].
    Struct,

    /// Plain array type, corresponds to [`NodeTypeClass::Array`].
    Array,

    /// Raw pointer type, corresponds to [`NodeTypeClass::Ref`].
    Pointer,
}

impl ContainerKind {
    /// Deduces what kind of container a node is based on the names
    /// that GDB assigned to its children.
    /// - If there are no children, the type cannot be deduced.
    /// - If there is exactly one child and its name indicates dereference,
    ///   the parent is a [`ContainerKind::Pointer`].
    /// - If all children's names are decimal numbers,
    ///   the parent is a [`ContainerKind::Array`].
    /// - Otherwise, the parent is a [`ContainerKind::Struct`].
    fn deduce_from_children<'a>(
        children: impl IntoIterator<Item = &'a ChildVariableObject>,
    ) -> Option<Self> {
        let mut children = children.into_iter();
        let Some(first_child) = children.next() else {
            // If there are no children, refuse to deduce type
            return None;
        };
        let is_decimal =
            |child: &ChildVariableObject| child.exp.chars().all(|c| c.is_ascii_digit());
        if first_child.exp.starts_with('*') && children.next().is_none() {
            // Child's name is indicative of a dereference,
            // so it is a pointer
            Some(Self::Pointer)
        } else if is_decimal(first_child) && children.all(is_decimal) {
            // All children's names are decimal numbers,
            // so it is an indexed array
            Some(Self::Array)
        } else {
            // Nothing else fits, assume it is a struct
            Some(Self::Struct)
        }
    }
}

impl From<ContainerKind> for NodeTypeClass {
    fn from(value: ContainerKind) -> Self {
        match value {
            ContainerKind::Struct => Self::Struct,
            ContainerKind::Array => Self::Array,
            ContainerKind::Pointer => Self::Ref,
        }
    }
}
