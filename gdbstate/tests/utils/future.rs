//! Shorthand for resolving a synchronous future.

use std::task::{Context, Poll, Waker};

/// Extension that allows us to synchronously resolve a future
/// that is in ready state.
pub trait ExpectReady: Future {
    /// Asserts that a future is in ready state and returns its result.
    ///
    /// While [`GdbMi`] is generally asynchronous, [`TestGdbMi`] implements
    /// all operations synchronously, meaning all futures are immediately
    /// resolved. This convenience method allows us to unwrap that
    /// resolved value.
    fn expect_ready(self) -> Self::Output;
}

impl<F: Future> ExpectReady for F {
    fn expect_ready(self) -> Self::Output {
        let mut context = Context::from_waker(Waker::noop());
        match std::pin::pin!(self).poll(&mut context) {
            Poll::Pending => {
                panic!("Called expect_ready on a future that was not ready")
            }
            Poll::Ready(output) => output,
        }
    }
}
