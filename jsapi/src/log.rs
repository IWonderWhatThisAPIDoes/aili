//! Logging to Javascript-side loggers.

use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const TYPESCRIPT_INTERFACES: &str = r"
    /**
     * Accepts diagnostic messages.
     */
    interface Logger {
        /**
         * Sends a message to the logger.
         * 
         * @param severity Severity class representing the nature of the message
         * @param message Text description of the message.
         */
        log(severity: Severity, message: string): void;
    }
";

#[wasm_bindgen]
extern "C" {
    /// Accepts disgnostic messages.
    #[wasm_bindgen(typescript_type = "Logger")]
    pub type Logger;

    /// Sends a message to the logger.
    #[wasm_bindgen(method)]
    pub fn log(this: &Logger, severity: Severity, message: &str);
}

/// Severity classes of log messages that indicate
/// the nature of the message.
#[wasm_bindgen]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Severity {
    /// Something has failed.
    Error,

    /// A suspicious occurrence that does not cause anything
    /// to fail, but is likely unintentional.
    Warning,

    /// Informative message.
    Info,

    /// Verbose message for assistance with debugging.
    Debug,
}
