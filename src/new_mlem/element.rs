use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fmt::Display;
use std::rc::Rc;

use crate::new_mlem::node::Node;
use crate::new_mlem::qname::QName;

use super::attribute::Attribute;

#[derive(Debug, Clone)]
pub struct InnerElement {
    name: QName,
}

impl Display for InnerElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.name, f)
    }
}

#[derive(Debug)]
pub struct Element<'a> {
    inner_element: Rc<RefCell<InnerElement>>,
    attributes: Vec<&'a Attribute>,
    parent: Option<&'a Element<'a>>,
    children: Vec<Node>,
}

impl<'a> Element<'a> {
    pub fn new_root_element(name: impl Into<String>) -> Result<Self, super::Error> {
        let qname = QName::new_without_namespace(name)?;

        let inner_element = InnerElement { name: qname };

        Ok(Self {
            inner_element: Rc::new(RefCell::new(inner_element)),
            attributes: vec![],
            parent: None,
            children: vec![],
        })
    }

    pub fn add_attribute(&mut self, attribute: &'a Attribute) -> Result<(), super::Error> {
        self.attributes.push(attribute);

        Ok(())
    }

    pub fn attributes(&self) -> Result<Vec<&'a Attribute>, super::Error> {
        Ok(self.attributes.clone())
    }
}

impl Clone for Element<'_> {
    fn clone(&self) -> Self {
        let borrow = RefCell::borrow(&*self.inner_element);

        Self {
            inner_element: Rc::new(RefCell::new(borrow.clone())),
            attributes: self.attributes.clone(),
            parent: self.parent.clone(),
            children: self.children.clone(),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        *self = source.clone();
    }
}

impl Display for Element<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&RefCell::borrow(&*self.inner_element), f)?;

        // Display parent?

        for child in &self.children {
            Display::fmt(&child, f)?;
        }

        Ok(())
    }
}
