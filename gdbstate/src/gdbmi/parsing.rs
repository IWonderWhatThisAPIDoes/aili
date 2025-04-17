//! Parsing [`raw_output`](super::raw_output) data as [`types`](super::types).
//!
//! This module provides extension methods for [`Value`] that allow
//! one to easily parse payloads.

use super::{raw_output::*, result::BadResponse, types::*};

/// Result type associated with parsing response payloads.
pub type Result<T> = std::result::Result<T, BadResponse>;

/// Extension methods for [`Value`] that allow parsing
/// payloads as various formats defined in [`types`](super::types).
impl Value {
    pub fn tuple(self) -> Result<ResultTuple> {
        self.into_tuple().ok_or(BadResponse::BadValueType)
    }

    pub fn list(self) -> Result<Vec<Value>> {
        self.into_list().ok_or(BadResponse::BadValueType)
    }

    pub fn string(self) -> Result<String> {
        self.into_const().ok_or(BadResponse::BadValueType)
    }

    pub fn decimal<T>(self) -> Result<T>
    where
        T: std::str::FromStr,
    {
        let str = self.string()?;
        str.parse().map_err(|_| BadResponse::BadValue(str))
    }

    pub fn hex(self) -> Result<u64> {
        let str = self.string()?;
        str.strip_prefix("0x")
            .and_then(|s| u64::from_str_radix(s, 16).ok())
            .ok_or(BadResponse::BadValue(str))
    }

    pub fn symbol_query_result(self) -> Result<Vec<SymbolFile>> {
        self.list()?.into_iter().map(Self::symbol_file).collect()
    }

    pub fn symbol_file(self) -> Result<SymbolFile> {
        self.tuple()?.symbol_file()
    }

    pub fn symbol_list(self) -> Result<Vec<Symbol>> {
        self.list()?.into_iter().map(Self::symbol).collect()
    }

    pub fn symbol(self) -> Result<Symbol> {
        self.tuple()?.symbol()
    }

    pub fn stack_trace(self) -> Result<Vec<StackFrame>> {
        self.tuple()?
            .0
            .into_iter()
            .filter(|r| r.key == "frame")
            .map(|r| r.value.stack_frame())
            .collect()
    }

    pub fn stack_frame(self) -> Result<StackFrame> {
        self.tuple()?.stack_frame()
    }

    pub fn local_variable_list(self) -> Result<Vec<LocalVariable>> {
        self.list()?.into_iter().map(Self::local_variable).collect()
    }

    pub fn local_variable(self) -> Result<LocalVariable> {
        self.tuple()?.local_variable()
    }

    pub fn zero_or_one(self) -> Result<bool> {
        let str = self.string()?;
        match str.as_str() {
            "0" => Ok(false),
            "1" => Ok(true),
            _ => Err(BadResponse::BadValue(str)),
        }
    }

    pub fn child_list_inner(self) -> Result<Vec<ChildVariableObject>> {
        self.tuple()?
            .0
            .into_iter()
            .filter(|r| r.key == "child")
            .map(|r| r.value.child_varobj())
            .collect()
    }

    pub fn child_varobj(self) -> Result<ChildVariableObject> {
        self.tuple()?.child_varobj()
    }

    pub fn varobj_changelist(self) -> Result<Vec<VariableObjectUpdate>> {
        self.list()?.into_iter().map(Self::varobj_update).collect()
    }

    pub fn varobj_update(self) -> Result<VariableObjectUpdate> {
        self.tuple()?.varobj_update()
    }

    pub fn in_scope_flag(self) -> Result<InScope> {
        let str = self.string()?;
        match str.as_str() {
            "true" => Ok(InScope::True),
            "false" => Ok(InScope::False),
            "invalid" => Ok(InScope::Invalid),
            _ => Ok(InScope::Other),
        }
    }
}

/// Extension methods for [`ResultTuple`] that allow parsing
/// payloads as various formats defined in [`types`](super::types).
impl ResultTuple {
    /// Extracts a named field from a tuple if it is present,
    /// returning an error otherwise.
    pub fn take(&mut self, key: &str) -> Result<Value> {
        self.take_optional(key)
            .ok_or_else(|| BadResponse::MissingKey(key.to_owned()))
    }

    /// Extracts a named field from a tuple if it is present.
    pub fn take_optional(&mut self, key: &str) -> Option<Value> {
        let index = self
            .0
            .iter()
            .enumerate()
            .find(|(_, r)| r.key == key)
            .map(|(i, _)| i)?;
        Some(self.0.remove(index).value)
    }

    pub fn symbol_file(mut self) -> Result<SymbolFile> {
        Ok(SymbolFile {
            filename: self.take("filename")?.string()?,
            fullname: self.take("fullname")?.string()?,
            symbols: self.take("symbols")?.symbol_list()?,
        })
    }

    pub fn symbol(mut self) -> Result<Symbol> {
        Ok(Symbol {
            line: self.take("line")?.decimal()?,
            name: self.take("name")?.string()?,
            type_name: self.take("type")?.string()?,
            description: self.take("description")?.string()?,
        })
    }

    pub fn stack_frame(mut self) -> Result<StackFrame> {
        Ok(StackFrame {
            level: self.take("level")?.decimal()?,
            addr: self.take("addr")?.hex()?,
            func: self.take("func")?.string()?,
            file: self.take("file")?.string()?,
            fullname: self.take("fullname")?.string()?,
            line: self.take("line")?.decimal()?,
            from: self.take_optional("from").map(Value::string).transpose()?,
            arch: self.take("arch")?.string()?,
        })
    }

    pub fn local_variable(mut self) -> Result<LocalVariable> {
        Ok(LocalVariable {
            name: self.take("name")?.string()?,
            arg: self
                .take_optional("arg")
                .map(Value::zero_or_one)
                .transpose()?
                .unwrap_or_default(),
            value: self.take_optional("value").map(Value::string).transpose()?,
        })
    }

    pub fn var_object(mut self) -> Result<VariableObjectData> {
        Ok(VariableObjectData {
            object: VariableObject(self.take("name")?.string()?),
            numchild: self.take("numchild")?.decimal()?,
            value: self.take_optional("value").map(Value::string).transpose()?,
            type_name: self.take("type")?.string()?,
            has_more: self
                .take_optional("has_more")
                .map(Value::zero_or_one)
                .transpose()?
                .unwrap_or_default(),
            dynamic: self
                .take_optional("dynamic")
                .map(Value::zero_or_one)
                .transpose()?
                .unwrap_or_default(),
            thread_id: self
                .take_optional("thread-id")
                .map(Value::string)
                .transpose()?,
        })
    }

    pub fn child_varobj(mut self) -> Result<ChildVariableObject> {
        Ok(ChildVariableObject {
            exp: self.take("exp")?.string()?,
            variable_object: self.var_object()?,
        })
    }

    pub fn varobj_update(mut self) -> Result<VariableObjectUpdate> {
        Ok(VariableObjectUpdate {
            object: VariableObject(self.take("name")?.string()?),
            value: self.take_optional("value").map(Value::string).transpose()?,
            in_scope: self.take("in_scope")?.in_scope_flag()?,
            new_type_name: self
                .take_optional("new_type_name")
                .map(Value::string)
                .transpose()?,
            new_num_children: self
                .take_optional("new_num_children")
                .map(Value::decimal)
                .transpose()?,
            has_more: self
                .take_optional("has_more")
                .map(Value::zero_or_one)
                .transpose()?
                .unwrap_or_default(),
            dynamic: self
                .take_optional("dynamic")
                .map(Value::zero_or_one)
                .transpose()?
                .unwrap_or_default(),
            new_children: self
                .take_optional("new_children")
                .map(Value::child_list_inner)
                .transpose()?
                .unwrap_or_default(),
        })
    }

    pub fn child_list(mut self) -> Result<ChildList> {
        Ok(ChildList {
            numchild: self.take("numchild")?.decimal()?,
            has_more: self
                .take_optional("has_more")
                .map(Value::zero_or_one)
                .transpose()?
                .unwrap_or_default(),
            children: self.take("children")?.child_list_inner()?,
        })
    }
}
