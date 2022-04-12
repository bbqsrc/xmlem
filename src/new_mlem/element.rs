use std::cell::RefCell;
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
        Display::fmt(&self.name, f)?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct Element<'a> {
    inner_element: Rc<RefCell<InnerElement>>,
    attributes: Rc<RefCell<Vec<Attribute>>>,
    pub parent: Option<&'a Element<'a>>,
    pub children: Vec<Node>,
}

impl<'a> Element<'a> {
    pub fn new_root_element(name: impl Into<String>) -> Result<Self, super::Error> {
        let qname = QName::new_without_namespace(name)?;

        let inner_element = InnerElement { name: qname };

        Ok(Self {
            inner_element: Rc::new(RefCell::new(inner_element)),
            attributes: Rc::new(RefCell::new(vec![])),
            parent: None,
            children: vec![],
        })
    }

    pub fn add_attribute(&self, attribute: Attribute) -> Result<(), super::Error> {
        let mut borrow_mut = RefCell::borrow_mut(&*self.attributes);

        borrow_mut.push(attribute);

        Ok(())
    }

    // Returns a clone of the attributes just for reading
    pub fn attributes(&self) -> Result<Vec<Attribute>, super::Error> {
        let borrow = RefCell::borrow(&*self.attributes);

        Ok(borrow.clone())
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
