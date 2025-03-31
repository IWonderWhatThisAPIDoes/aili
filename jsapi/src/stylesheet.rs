//! Simple interface to [`aili_translate::stylesheet`].

use aili_parser::{ParseError, parse_stylesheet};
use aili_translate::cascade::CascadeStyle;
use js_sys::Function;
use wasm_bindgen::prelude::*;

/// Compiled visualization stylesheet.
///
/// See [`aili_translate::cascade::CascadeStyle`]
#[wasm_bindgen]
pub struct Stylesheet(pub(crate) CascadeStyle);

#[wasm_bindgen]
impl Stylesheet {
    /// Parses and compiles a stylesheet source using [`aili_parser`].
    pub fn parse(source: &str, error_handler: Option<Function>) -> Result<Self, JsError> {
        let on_error = |err| {
            if let Some(f) = &error_handler {
                f.call1(&JsValue::NULL, &StylesheetParseError(err).into())
                    .expect("Uncaught exception thrown by callback passed to parse");
            }
        };
        parse_stylesheet(source, on_error)
            .map(CascadeStyle::from)
            .map(Stylesheet)
            .map_err(JsError::from)
    }
}

/// Type of error message emited when the stylesheet parser
/// encounters a recoverable syntax error.
///
/// See [`aili_parser::ParseError`].
#[wasm_bindgen]
pub struct StylesheetParseError(ParseError);

#[wasm_bindgen]
impl StylesheetParseError {
    /// Message that describes the error.
    #[wasm_bindgen(getter)]
    pub fn message(&self) -> String {
        self.0.to_string()
    }
}
