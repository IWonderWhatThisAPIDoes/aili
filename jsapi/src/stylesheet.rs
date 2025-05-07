//! Simple interface to [`aili_style::stylesheet`].

use aili_parser::{ParseError, parse_stylesheet};
use aili_style::{cascade::CascadeStyle, stylesheet};
use aili_translate::property::PropertyKey;
use js_sys::Function;
use wasm_bindgen::prelude::*;

/// Declares a stylesheet type for a given target.
///
/// Stylesheets are not dyn-polymorphic,
/// so the bindings need to distinguish different types.
macro_rules! declare_stylesheet {
    ( $( #[ $attr:meta ] )* $name:ident ( $key:ty )) => {
        $( #[ $attr ] )*
        #[wasm_bindgen]
        pub struct $name(pub(crate) CascadeStyle<$key>);

        #[wasm_bindgen]
        impl $name {
            /// Constructs an empty stylesheet.
            pub fn empty() -> Self {
                Self(CascadeStyle::empty())
            }

            /// Parses and compiles a stylesheet source using [`aili_parser`].
            pub fn parse(source: &str, error_handler: Option<Function>) -> Result<Self, JsError> {
                let on_error = |err| {
                    if let Some(f) = &error_handler {
                        f.call1(&JsValue::NULL, &StylesheetParseError(err).into())
                            .expect("Uncaught exception thrown by callback passed to parse");
                    }
                };
                parse_stylesheet(source, on_error)
                    .map(stylesheet::Stylesheet::map_key)
                    .map(CascadeStyle::from)
                    .map(Self)
                    .map_err(JsError::from)
            }
        }
    };
}

declare_stylesheet! {
    /// Compiled visualization stylesheet.
    ///
    /// See [`CascadeStyle`].
    Stylesheet(PropertyKey)
}
#[cfg(feature = "gdbstate")]
declare_stylesheet! {
    /// Compiled stylesheet with hints to help deduce length of arrays.
    ///
    /// See [`CascadeStyle`].
    LengthHintSheet(aili_gdbstate::hints::PointerLengthHintKey)
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
