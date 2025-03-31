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

pub struct TestVisElementRef<'a>(&'a mut TestVisTree, usize);

#[derive(PartialEq, Eq, Debug, Default)]
pub struct TestVisConnector {
    pub attributes: HashMap<String, String>,
    pub start: TestVisPin,
    pub end: TestVisPin,
}

#[derive(PartialEq, Eq, Debug, Default)]
pub struct TestVisPin {
    pub attributes: HashMap<String, String>,
    pub target_index: Option<usize>,
}

impl VisTree for TestVisTree {
    type ElementHandle = usize;
    type ConnectorHandle = usize;
    type ElementRef<'a> = TestVisElementRef<'a>;
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
        Ok(TestVisElementRef(self, *handle))
    }

    fn set_root(&mut self, handle: Option<&Self::ElementHandle>) -> Result<(), InvalidHandle> {
        self.root_index = handle.copied();
        Ok(())
    }
}

impl AttributeMap for TestVisElementRef<'_> {
    fn get_attribute(&self, name: &str) -> Option<&str> {
        self.element().attributes.get(name).map(String::as_str)
    }

    fn set_attribute(&mut self, name: &str, value: Option<&str>) {
        if let Some(value) = value {
            self.element_mut()
                .attributes
                .insert(name.to_owned(), value.to_owned());
        } else {
            self.element_mut().attributes.remove(name);
        }
    }
}

impl VisElement for TestVisElementRef<'_> {
    type Handle = usize;

    fn insert_into(&mut self, parent: Option<&Self::Handle>) -> Result<(), ParentAssignmentError> {
        if parent.is_some_and(|p| self.0.is_ancestor_of(self.1, *p)) {
            Err(ParentAssignmentError::StructureViolation)
        } else {
            self.element_mut().parent_index = parent.copied();
            Ok(())
        }
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

impl VisPin for &mut TestVisPin {
    type Handle = usize;

    fn attach_to(&mut self, target: Option<&Self::Handle>) -> Result<(), InvalidHandle> {
        self.target_index = target.copied();
        Ok(())
    }
}

impl TestVisElementRef<'_> {
    fn element(&self) -> &TestVisElement {
        &self.0.elements[self.1]
    }

    fn element_mut(&mut self) -> &mut TestVisElement {
        &mut self.0.elements[self.1]
    }
}

impl TestVisTree {
    fn is_ancestor_of(&self, ancestor: usize, mut descendant: usize) -> bool {
        loop {
            if descendant == ancestor {
                break true;
            }
            match self.elements[descendant].parent_index {
                Some(parent) => descendant = parent,
                None => break false,
            }
        }
    }

    pub fn expect_find_element(&self, pred: impl Fn(&TestVisElement) -> bool) -> usize {
        self.elements
            .iter()
            .enumerate()
            .find(|(_, e)| pred(e))
            .map(|(i, _)| i)
            .expect("Expected element was not found")
    }
}
