//! Expose parser for
//! [GDB/MI output syntax](https://sourceware.org/gdb/current/onlinedocs/gdb.html/GDB_002fMI-Output-Syntax.html).

#![cfg(feature = "gdbstate")]

use aili_gdbstate::gdbmi::{grammar::parse_gdbmi_record as parse_gdbmi_record_impl, raw_output::*};
use js_sys::{Array, JsString, Object, Reflect};
use wasm_bindgen::prelude::*;

/// Single line of GDB/MI output.
#[wasm_bindgen(getter_with_clone)]
pub struct GdbMiRecord {
    /// Payload data that specifies the output.
    pub results: JsValue,
}

/// Parses a line of GDB/MI output.
#[wasm_bindgen(js_name = "parseGdbMiRecord")]
pub fn parse_gdbmi_record(record: &str) -> Option<GdbMiRecord> {
    match parse_gdbmi_record_impl(record) {
        Ok(record) => {
            let results = match &record {
                Record::Result(r) => &r.results,
                Record::AsyncExec(r) => &r.results,
            };
            Some(GdbMiRecord {
                results: result_tuple_to_js(results).into(),
            })
        }
        Err(_) => None,
    }
}

/// Converts a GDB/MI tuple to JS object.
fn result_tuple_to_js(tuple: &ResultTuple) -> Object {
    let jsvalue = Object::new();
    for entry in &tuple.0 {
        Reflect::set(
            &jsvalue,
            &JsString::from(entry.key.as_str()),
            &value_to_js(&entry.value),
        )
        .expect("Types of all objects are verified, this should never fail");
    }
    jsvalue
}

/// Converts a GDB/MI list to JS object.
fn result_list_to_js<'a>(list: impl IntoIterator<Item = &'a Value>) -> Array {
    list.into_iter().map(value_to_js).collect()
}

/// Converts a GDB/MI value to JS object.
fn value_to_js(value: &Value) -> JsValue {
    match value {
        Value::Const(s) => JsString::from(s.as_str()).into(),
        Value::Tuple(t) => result_tuple_to_js(t).into(),
        Value::List(l) => result_list_to_js(l).into(),
        // We use list here as well, since tuple lists typically have list semantics
        Value::TupleList(l) => result_list_to_js(l.0.iter().map(|e| &e.value)).into(),
    }
}
