//! Mock error handler that can be used to verify
//! error reporting aspects of parsers.

#![cfg(test)]

/// Error handler that asserts
/// that specific errors were emited in a particular order.
pub struct ExpectErrors<T: Eq + std::fmt::Debug>(ExpectErrorsKind<T>);

/// Internal data for what errors should be permitted.
enum ExpectErrorsKind<T: Eq + std::fmt::Debug> {
    /// Exact sequence of errors is expected.
    Exact(Vec<T>),
    /// Expect any errors, fail if no errors are emited
    Some,
    /// Anything is allowed.
    Any,
}

impl<T: Eq + std::fmt::Debug> ExpectErrors<T> {
    /// Expects no errors to be emited.
    pub fn none() -> Self {
        Self(ExpectErrorsKind::Exact(Vec::new()))
    }

    /// Expects at least one error of any kind to be emited.
    pub fn some() -> Self {
        Self(ExpectErrorsKind::Some)
    }

    /// Expects a sequence of errors in a specific order.
    pub fn exact(expected: impl IntoIterator<Item = T, IntoIter: DoubleEndedIterator>) -> Self {
        Self(ExpectErrorsKind::Exact(
            expected.into_iter().rev().collect(),
        ))
    }

    /// Turns the handler into a callable function
    /// (manual implementation of [`FnMut`] trait is still not allowed)
    pub fn f(mut self) -> impl FnMut(T) {
        move |actual| {
            match &mut self.0 {
                ExpectErrorsKind::Exact(expected) => {
                    // Must be exactly the expected error
                    assert_eq!(expected.pop(), Some(actual), "Unexpected error was emited");
                }
                ExpectErrorsKind::Some => {
                    // An error has been reported, we are happy now
                    self.0 = ExpectErrorsKind::Any;
                }
                ExpectErrorsKind::Any => {
                    // Never fail
                }
            }
        }
    }
}

impl<T: Eq + std::fmt::Debug> Drop for ExpectErrors<T> {
    /// Asserts that all expected errors have been emited before the handler expires
    fn drop(&mut self) {
        match &self.0 {
            ExpectErrorsKind::Exact(expected) => {
                assert!(
                    expected.is_empty(),
                    "Expected errors were not emited: {:?}",
                    expected
                );
            }
            ExpectErrorsKind::Some => {
                // If we are here, it means no errors have been emited
                panic!("Expected an error to be emited, but none were");
            }
            ExpectErrorsKind::Any => {
                // Never fail
            }
        }
    }
}
