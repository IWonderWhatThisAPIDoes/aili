//! Adapter for filtering lexer and syntax error reports.

/// Adapter for a parser error handler that filters messages
/// so as to not send a whole cascade of errors that are
/// usualy consequences of just one typo.
///
/// This adapter implements the same strategy as
/// [Pomelo](https://docs.rs/pomelo/latest/pomelo/#error-processing),
/// which is in turn inspired by Yacc.
/// Whenever an error is reported, the filter switches
/// to cooldown state, and discards any further errors
/// until [`FilteredErrorHandler::REPORT_COOLDOWN`] consecutive
/// tokens have been accepted without any errors.
///
/// Although Pomelo implements this on its own,
/// this adapter is needed to provide the same filter
/// for lexer errors as well.
pub struct FilteredErrorHandler<T: FnMut(E), E> {
    /// The error handler callback.
    error_handler: T,
    /// How many tokens need to be accepted before
    /// another error can be reported.
    cooldown: u8,
    /// Marker to provide context for the unused type parameter.
    _marker: std::marker::PhantomData<fn(E)>,
}

impl<T: FnMut(E), E> FilteredErrorHandler<T, E> {
    /// How many tokens must be successfully accepted
    /// before another error can be reported.
    pub const REPORT_COOLDOWN: u8 = 3;

    /// Wraps an error handler in the adapter.
    pub fn new(error_handler: T) -> Self {
        Self {
            error_handler,
            cooldown: 0,
            _marker: std::marker::PhantomData,
        }
    }

    /// Sends an error through the adapter.
    ///
    /// The error will be forwarded to the contained
    /// error handler unless another error has been
    /// reported recently.
    pub fn handle_error(&mut self, error: E) {
        if self.cooldown == 0 {
            (self.error_handler)(error);
        }
        // Add a +1 here, because REPORT_COOLDOWN counts
        // successful token parses, while token_parsed
        // is called for all token parses
        self.cooldown = Self::REPORT_COOLDOWN + 1;
    }

    /// Notifies the adapter that a token has been processed.
    ///
    /// This includes tokens that have been processed with an error.
    ///
    /// Calling this contributes to the reporting cooldown.
    pub fn token_parsed(&mut self) {
        self.cooldown = self.cooldown.saturating_sub(1);
    }
}
