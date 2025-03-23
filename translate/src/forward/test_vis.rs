//! Stub [`VisTree`] for testing.

#![cfg(test)]

use aili_model::vis::*;
use std::collections::HashMap;

#[derive(Default)]
pub struct TestVisTree {
    pub elements: Vec<TestVisElement>,
    pub connectors: Vec<TestVisConnector>,
    pub root_index: Option<usize>,
}

#[derive(PartialEq, Eq, Debug, Default)]
pub struct TestVisElement {
    pub tag_name: String,
    pub attributes: HashMap<String, String>,
    pub parent_index: Option<usize>,
}

#[derive(PartialEq, Eq, Debug, Default)]
pub struct TestVisConnector {
    pub attributes: HashMap<String, String>,
    pub start: TestVisPin,
    pub end: TestVisPin,
}

#[derive(PartialEq, Eq, Debug, Default)]
pub struct TestVisPin {
    pub target_index: Option<usize>,
}

impl VisTree for TestVisTree {
    type ElementHandle = usize;
    type ConnectorHandle = usize;
    type ElementRef<'a> = &'a mut TestVisElement;
    type ConnectorRef<'a> = &'a mut TestVisConnector;

    fn add_connector(&mut self) -> Self::ConnectorHandle {
        self.connectors.push(TestVisConnector::default());
        self.connectors.len() - 1
    }

    fn add_element(&mut self, tag_name: &str) -> Self::ElementHandle {
        self.elements.push(TestVisElement {
            tag_name: tag_name.to_owned(),
            ..TestVisElement::default()
        });
        self.elements.len() - 1
    }

    fn get_connector(
        &mut self,
        handle: &Self::ConnectorHandle,
    ) -> Result<Self::ConnectorRef<'_>, InvalidHandle> {
        Ok(&mut self.connectors[*handle])
    }

    fn get_element(
        &mut self,
        handle: &Self::ElementHandle,
    ) -> Result<Self::ElementRef<'_>, InvalidHandle> {
        Ok(&mut self.elements[*handle])
    }

    fn set_root(&mut self, handle: Option<&Self::ElementHandle>) -> Result<(), InvalidHandle> {
        self.root_index = handle.copied();
        Ok(())
    }
}

impl AttributeMap for &mut TestVisElement {
    fn get_attribute(&self, name: &str) -> Option<&str> {
        self.attributes.get(name).map(String::as_str)
    }

    fn set_attribute(&mut self, name: &str, value: Option<&str>) {
        if let Some(value) = value {
            self.attributes.insert(name.to_owned(), value.to_owned());
        } else {
            self.attributes.remove(name);
        }
    }
}

impl VisElement for &mut TestVisElement {
    type Handle = usize;

    fn insert_into(&mut self, parent: Option<&Self::Handle>) -> Result<(), ParentAssignmentError> {
        // We disregard the possibility of creating circular references here,
        // that is not what this test is about
        self.parent_index = parent.copied();
        Ok(())
    }
}

impl AttributeMap for &mut TestVisConnector {
    fn get_attribute(&self, name: &str) -> Option<&str> {
        self.attributes.get(name).map(String::as_str)
    }

    fn set_attribute(&mut self, name: &str, value: Option<&str>) {
        if let Some(value) = value {
            self.attributes.insert(name.to_owned(), value.to_owned());
        } else {
            self.attributes.remove(name);
        }
    }
}

impl VisConnector for &mut TestVisConnector {
    type Handle = usize;
    type PinRef<'a>
        = &'a mut TestVisPin
    where
        Self: 'a;

    fn start_mut(&mut self) -> Self::PinRef<'_> {
        &mut self.start
    }

    fn end_mut(&mut self) -> Self::PinRef<'_> {
        &mut self.end
    }
}

impl AttributeMap for &mut TestVisPin {
    fn get_attribute(&self, _: &str) -> Option<&str> {
        unimplemented!()
    }

    fn set_attribute(&mut self, _: &str, _: Option<&str>) {
        unimplemented!()
    }
}

impl VisPin for &mut TestVisPin {
    type Handle = usize;

    fn attach_to(&mut self, target: Option<&Self::Handle>) -> Result<(), InvalidHandle> {
        self.target_index = target.copied();
        Ok(())
    }
}
