//! Visualization tree model.

use derive_more::{Display, Error, From};

/// Error type that indicates the use of an invalid handle
/// to access a visualization entity.
#[derive(Clone, Copy, Debug, Display, Default, Error)]
#[display("invalid visualization entity handle")]
pub struct InvalidHandle;

/// Error type of [`VisElement::insert_into`].
#[derive(Clone, Copy, Debug, Display, Error, From)]
pub enum ParentAssignmentError {
    /// Parent handle is not valid.
    InvalidHandle(InvalidHandle),

    /// The operation would create a cycle in the parent-child relationships.
    #[display("visualization tree operation was rejected because it would create a cycle")]
    StructureViolation,
}

/// Owning handle to a visualization element or connector.
pub trait VisHandle: Clone {}

impl<T: Clone> VisHandle for T {}

/// Container for string attributes.
pub trait AttributeMap {
    /// Gets the value of an attribute, if present.
    fn get_attribute(&self, name: &str) -> Option<&str>;

    /// Updates the value of an attribute or removes it.
    fn set_attribute(&mut self, name: &str, value: Option<&str>);
}

/// Visualization tree element.
pub trait VisElement: AttributeMap {
    /// Type of handles to elements.
    type Handle: VisHandle;

    /// Updates the parent element of this element.
    fn insert_into(&mut self, parent: Option<&Self::Handle>) -> Result<(), ParentAssignmentError>;
}

/// Visualization tree connector.
pub trait VisConnector: AttributeMap {
    /// Type of handles to elements (not connectors).
    type Handle: VisHandle;

    /// Type of references to connector pins
    type PinRef<'a>: VisPin<Handle = Self::Handle> + 'a
    where
        Self: 'a;

    /// Gets the start pin.
    fn start_mut(&mut self) -> Self::PinRef<'_>;

    /// Gets the end pin.
    fn end_mut(&mut self) -> Self::PinRef<'_>;
}

/// Visualization tree connector pin.
pub trait VisPin: AttributeMap {
    /// Type of handles to elements (not connectors or pins).
    type Handle: VisHandle;

    /// Updates the target element of this pin.
    fn attach_to(&mut self, target: Option<&Self::Handle>) -> Result<(), InvalidHandle>;
}

/// Container for a visualization tree.
pub trait VisTree {
    /// Type of handles to elements.
    type ElementHandle: VisHandle;

    /// Type of handles to connectors.
    type ConnectorHandle: VisHandle;

    /// Type of references to elements.
    type ElementRef<'a>: VisElement<Handle = Self::ElementHandle> + 'a
    where
        Self: 'a;

    /// Type of references to connectors.
    type ConnectorRef<'a>: VisConnector<Handle = Self::ElementHandle> + 'a
    where
        Self: 'a;

    /// Sets an element to be the root of the visualization tree.
    fn set_root(&mut self, handle: Option<&Self::ElementHandle>) -> Result<(), InvalidHandle>;

    /// Creates a new element.
    fn add_element(&mut self, tag_name: &str) -> Self::ElementHandle;

    /// Creates a new connector.
    fn add_connector(&mut self) -> Self::ConnectorHandle;

    /// Finds an element by its handle.
    fn get_element(
        &mut self,
        handle: &Self::ElementHandle,
    ) -> Result<Self::ElementRef<'_>, InvalidHandle>;

    /// Finds a connector by its handle.
    fn get_connector(
        &mut self,
        handle: &Self::ConnectorHandle,
    ) -> Result<Self::ConnectorRef<'_>, InvalidHandle>;
}
