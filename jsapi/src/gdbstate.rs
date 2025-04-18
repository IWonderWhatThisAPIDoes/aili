//! State graph implementation that uses a GDB session.

#![cfg(feature = "gdbstate")]

use aili_gdbstate::{
    gdbmi::stream::StringGdbMiStream,
    state::{GdbStateGraph as GdbStateGraphImpl, GdbStateNode, GdbStateNodeId},
};
use aili_model::state::{ProgramStateGraph, RootedProgramStateGraph};
use js_sys::Reflect;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const TYPESCRIPT_INTERFACES: &str = r"
    /**
     * Accepts GDB/MI commands.
     */
    interface GdbMiSession {
        /**
         * Executes a GDB/MI command.
         * 
         * Returns the result record corresponding to the passed command.
         * 
         * @throws The command is invalid or the session could not execute it.
         */
        sendMiCommand(command: string): Promise<string>;
    }
";

#[wasm_bindgen]
extern "C" {
    /// GDB/MI session imported from Javascript.
    #[wasm_bindgen(typescript_type = "GdbMiSession")]
    pub type GdbMi;

    /// Sends a GDB/MI command to the session.
    #[wasm_bindgen(method, js_name = "sendMiCommand", catch)]
    pub async fn send_mi_command(this: &GdbMi, command: &str) -> Result<JsValue, JsValue>;
}

// Implement the trait for reference so that we can use it mutably
// Wasm-bindgen would not give us `&mut` to a JS object
impl StringGdbMiStream for &GdbMi {
    async fn send_command(&mut self, command: &str) -> std::io::Result<String> {
        match self.send_mi_command(command).await {
            Ok(output) => output.as_string().ok_or_else(|| {
                std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Command did not return a string",
                )
            }),
            Err(err) => Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                js_error_description(&err),
            )),
        }
    }
}

/// Extracts an error description from a JS object that was thrown as an error
///
/// - If the object is a string, returns it directly
/// - If the object is an error, returns its message
/// - Otherwise, returns a default error handling message
fn js_error_description(value: &JsValue) -> String {
    value
        .as_string()
        .or_else(|| {
            Reflect::get(value, &JsValue::from_str("message"))
                .ok()
                .as_ref()
                .and_then(JsValue::as_string)
        })
        .unwrap_or_else(|| "No details provided".to_owned())
}

/// [`ProgramStateGraph`] constructed using a GDB/MI session.
#[wasm_bindgen]
pub struct GdbStateGraph(pub(crate) GdbStateGraphImpl);

#[wasm_bindgen]
impl GdbStateGraph {
    /// Constructs a new state graph from a GDB/MI session.
    #[wasm_bindgen(js_name = "fromSession")]
    pub async fn from_session(mut gdb_mi: &GdbMi) -> Result<Self, JsError> {
        aili_gdbstate::state::GdbStateGraph::new(&mut gdb_mi)
            .await
            .map(Self)
            .map_err(|e| JsError::new(&format!("{e}")))
    }

    /// Updates the state graph using the provided GDB/MI session.
    pub async fn update(&mut self, mut gdb_mi: &GdbMi) -> Result<(), JsError> {
        self.0
            .update(&mut gdb_mi)
            .await
            .map_err(|e| JsError::new(&format!("{e}")))
    }

    /// Cleans up state that was required by the state graph from the provided GDB/MI session.
    #[wasm_bindgen(js_name = "cleanUp")]
    pub async fn clean_up(&self, mut gdb_mi: &GdbMi) -> Result<(), JsError> {
        self.0
            .drop_variable_objects(&mut gdb_mi)
            .await
            .map_err(|e| JsError::new(&format!("{e}")))
    }
}

impl ProgramStateGraph for GdbStateGraph {
    type NodeId = GdbStateNodeId;
    type NodeRef<'a> = &'a GdbStateNode;
    fn get(&self, id: &Self::NodeId) -> Option<Self::NodeRef<'_>> {
        self.0.get(id)
    }
}

impl RootedProgramStateGraph for GdbStateGraph {
    fn root(&self) -> Self::NodeId {
        self.0.root()
    }
}
