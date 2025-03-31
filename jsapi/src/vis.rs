//! Bindings for a Javascript-side implementation of [`aili_model::vis::VisTree`].

use aili_model::vis;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const TYPESCRIPT_INTERFACES: &str = r"
    /**
     * Container for attributes of a visual entity.
     */
    type AttributeMap = Record<string, { value: string | undefined }>;
    /**
     * Container for a visualization tree.
     */
    interface VisTree {
        /**
         * Updates the root element of the tree.
         */
        set root(root: VisElement | undefined);
        /**
         * Constructs a new {@link VisElement} that can be used with the tree.
         * 
         * @param tagName Tag name of the element.
         */
        createElement(tagName: string): VisElement;
        /**
         * Constructs a new {@link VisConnector that can be used with the tree.}
         */
        createConnector(): VisConnector;
    }
    /**
     * Single element of a {@link VisTree}.
     */
    interface VisElement {
        /**
         * Attributes of the element.
         */
        readonly attributes: AttributeMap;
        /**
         * Updates the parent element.
         * 
         * @throws when the update would create a cycle in the tree.
         */
        set parent(parent: VisElement | undefined);
    }
    /**
     * Connector that can connect two {@link VisElement}s.
     */
    interface VisConnector {
        /**
         * Attributes of the connector.
         */
        readonly attributes: AttributeMap;
        /**
         * Start pin of the connector.
         */
        readonly start: VisPin;
        /**
         * End pin of the connector.
         */
        readonly end: VisPin;
    }
    /**
     * Either endpoint of a {@link VisConnector}.
     */
    interface VisPin {
        /**
         * Attributes of the pin.
         */
        readonly attributes: AttributeMap;
        /**
         * Updates what element the pin is attached to.
         */
        set target(target: VisElement | undefined);
    }
";

#[wasm_bindgen]
extern "C" {
    /// Base of types that have an attribute map.
    #[derive(Clone)]
    pub type WithAttributeMap;

    /// Visualization tree.
    #[wasm_bindgen(typescript_type = "VisTree")]
    pub type VisTree;

    /// Element of a [`VisTree`].
    #[wasm_bindgen(extends = WithAttributeMap, typescript_type = "VisElement")]
    #[derive(Clone)]
    pub type VisElement;

    /// Connector that can connect two [`VisElement`]s.
    #[wasm_bindgen(extends = WithAttributeMap, typescript_type = "VisConnector")]
    #[derive(Clone)]
    pub type VisConnector;

    /// Endpoint of a [`VisConnector`].
    #[wasm_bindgen(extends = WithAttributeMap, typescript_type = "VisPin")]
    pub type VisPin;

    /// Attribute map associated with the object.
    #[wasm_bindgen(method, getter)]
    pub fn attributes(this: &WithAttributeMap) -> js_sys::Object;

    /// Updates the root element of the tree.
    #[wasm_bindgen(method, setter, js_name = "root")]
    pub fn set_root(this: &VisTree, root_element: Option<&VisElement>);

    /// Creates a new element for use with the tree.
    #[wasm_bindgen(method, js_name = "createElement")]
    pub fn create_element(this: &VisTree, tag_name: &str) -> VisElement;

    /// Creates a new connector for use with the tree.
    #[wasm_bindgen(method, js_name = "createConnector")]
    pub fn create_connector(this: &VisTree) -> VisConnector;

    /// Start pin of the connector.
    #[wasm_bindgen(method, getter)]
    pub fn start(this: &VisConnector) -> VisPin;

    /// End pin of the connector.
    #[wasm_bindgen(method, getter)]
    pub fn end(this: &VisConnector) -> VisPin;

    /// Updates what element a pin is attached to.
    #[wasm_bindgen(method, setter, js_name = "target")]
    pub fn attach_to(this: &VisPin, target: Option<&VisElement>);

    /// Updates an element's parent element.
    ///
    /// Fails if the operation would create a loop.
    #[wasm_bindgen(method, setter, js_name = "parent", catch)]
    pub fn insert_into(this: &VisElement, parent: Option<&VisElement>) -> Result<(), JsValue>;
}

/// Updates an attribute in an attribute map.
///
/// Wasm-bindgen cannot express the specific interface we are trying to create,
/// so we implement this accessor manually.
fn set_attribute(attributes: &WithAttributeMap, name: &str, value: Option<&str>) {
    let new_value = value.map(JsValue::from_str).unwrap_or(JsValue::UNDEFINED);
    let attribute_record = js_sys::Reflect::get(&attributes.attributes(), &JsValue::from_str(name))
        .expect("Cannot access attribute values");
    let succeeded =
        js_sys::Reflect::set(&attribute_record, &JsValue::from_str("value"), &new_value)
            .expect("Cannot access attribute value");
    if !succeeded {
        panic!("Cannot access attribute value");
    }
}

impl vis::VisTree for VisTree {
    type ElementRef<'a> = VisElement;
    type ConnectorRef<'a> = VisConnector;
    type ElementHandle = VisElement;
    type ConnectorHandle = VisConnector;

    fn get_connector(
        &mut self,
        handle: &Self::ConnectorHandle,
    ) -> Result<Self::ConnectorRef<'_>, vis::InvalidHandle> {
        Ok(handle.clone())
    }

    fn get_element(
        &mut self,
        handle: &Self::ElementHandle,
    ) -> Result<Self::ElementRef<'_>, vis::InvalidHandle> {
        Ok(handle.clone())
    }

    fn set_root(&mut self, handle: Option<&Self::ElementHandle>) -> Result<(), vis::InvalidHandle> {
        VisTree::set_root(self, handle);
        Ok(())
    }

    fn add_connector(&mut self) -> Self::ConnectorHandle {
        self.create_connector()
    }

    fn add_element(&mut self, tag_name: &str) -> Self::ElementHandle {
        self.create_element(tag_name)
    }
}

impl vis::AttributeMap for VisElement {
    fn get_attribute(&self, _: &str) -> Option<&str> {
        None
    }

    fn set_attribute(&mut self, name: &str, value: Option<&str>) {
        set_attribute(self, name, value);
    }
}

impl vis::VisElement for VisElement {
    type Handle = VisElement;

    fn insert_into(
        &mut self,
        parent: Option<&Self::Handle>,
    ) -> Result<(), vis::ParentAssignmentError> {
        VisElement::insert_into(self, parent)
            .map_err(|_| vis::ParentAssignmentError::StructureViolation)
    }
}

impl vis::AttributeMap for VisConnector {
    fn get_attribute(&self, _: &str) -> Option<&str> {
        None
    }

    fn set_attribute(&mut self, name: &str, value: Option<&str>) {
        set_attribute(self, name, value);
    }
}

impl vis::VisConnector for VisConnector {
    type Handle = VisElement;
    type PinRef<'a> = VisPin;

    fn start_mut(&mut self) -> Self::PinRef<'_> {
        self.start()
    }

    fn end_mut(&mut self) -> Self::PinRef<'_> {
        self.end()
    }
}

impl vis::AttributeMap for VisPin {
    fn get_attribute(&self, _: &str) -> Option<&str> {
        None
    }

    fn set_attribute(&mut self, name: &str, value: Option<&str>) {
        set_attribute(self, name, value);
    }
}

impl vis::VisPin for VisPin {
    type Handle = VisElement;

    fn attach_to(&mut self, target: Option<&Self::Handle>) -> Result<(), vis::InvalidHandle> {
        VisPin::attach_to(self, target);
        Ok(())
    }
}
