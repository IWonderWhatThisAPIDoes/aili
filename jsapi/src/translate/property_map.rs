//! Serialization of property mappings

use aili_model::state::NodeId;
use aili_style::selectable::Selectable;
use aili_translate::property::{FragmentKey, PropertyMap};
use wasm_bindgen::prelude::*;

/// Serialized type of property key.
#[wasm_bindgen]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PropertyKeyType {
    Display = "display",
    Parent = "parent",
    Target = "target",
    Attribute = "attr",
}

/// Serialized identifier of entity fragment.
#[wasm_bindgen(js_name = "FragmentKey")]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FragmentKeyOutput {
    Start = "start",
    End = "end",
}

impl From<FragmentKey> for FragmentKeyOutput {
    fn from(value: FragmentKey) -> Self {
        match value {
            FragmentKey::Start => FragmentKeyOutput::Start,
            FragmentKey::End => FragmentKeyOutput::End,
        }
    }
}

/// Single assignment of a value to a property.
#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
pub struct PropertyMapEntry {
    /// Type of the property.
    #[wasm_bindgen(js_name = "keyType")]
    pub key_type: PropertyKeyType,
    /// Name of the property, if it is an attribute.
    #[wasm_bindgen(js_name = "attributeName")]
    pub attribute_name: Option<String>,
    /// Fragment of the property, if it is a fragment attribute.
    #[wasm_bindgen(js_name = "fragmentKey")]
    pub fragment_key: Option<FragmentKeyOutput>,
    /// Value assigned to the property.
    pub value: String,
}

impl PropertyMapEntry {
    fn attribute(attribute_name: String, value: String) -> Self {
        Self {
            key_type: PropertyKeyType::Attribute,
            attribute_name: Some(attribute_name),
            value,
            fragment_key: None,
        }
    }

    fn fragment_attribute(
        attribute_name: String,
        value: String,
        fragment_key: FragmentKeyOutput,
    ) -> Self {
        Self {
            key_type: PropertyKeyType::Attribute,
            attribute_name: Some(attribute_name),
            value,
            fragment_key: Some(fragment_key),
        }
    }

    fn from_key_value(key_type: PropertyKeyType, value: String) -> Self {
        Self {
            key_type,
            value,
            attribute_name: None,
            fragment_key: None,
        }
    }
}

/// Property mapping related to a single entity.
#[wasm_bindgen(getter_with_clone, js_name = "PropertyMap")]
#[derive(Clone)]
pub struct PropertyMapSnapshot {
    /// Serialized identifier of the entity.
    #[wasm_bindgen(js_name = "nodeId")]
    pub node_id: String,
    /// Properties assigned to the entity.
    pub properties: Vec<PropertyMapEntry>,
}

impl PropertyMapSnapshot {
    /// Constructs an entity property mapping from a [`PropertyMap`].
    pub fn from_property_map<T: NodeId>(target: &Selectable<T>, props: &PropertyMap<T>) -> Self {
        let mut properties = Vec::new();

        if let Some(display) = &props.display {
            properties.push(PropertyMapEntry::from_key_value(
                PropertyKeyType::Display,
                format!("{display:?}"),
            ));
        }
        if let Some(parent) = &props.parent {
            properties.push(PropertyMapEntry::from_key_value(
                PropertyKeyType::Parent,
                format!("{parent:?}"),
            ));
        }
        if let Some(target) = &props.target {
            properties.push(PropertyMapEntry::from_key_value(
                PropertyKeyType::Target,
                format!("{target:?}"),
            ));
        }
        for (attr, value) in &props.attributes {
            properties.push(PropertyMapEntry::attribute(attr.clone(), value.clone()));
        }
        for (fragment, attrs) in &props.fragment_attributes {
            for (attr, value) in attrs {
                properties.push(PropertyMapEntry::fragment_attribute(
                    attr.clone(),
                    value.clone(),
                    (*fragment).into(),
                ));
            }
        }

        properties.sort_by(|a, b| {
            (a.key_type, &a.fragment_key, &a.attribute_name).cmp(&(
                b.key_type,
                &b.fragment_key,
                &b.attribute_name,
            ))
        });

        Self {
            node_id: format!("{target:?}"),
            properties,
        }
    }
}
