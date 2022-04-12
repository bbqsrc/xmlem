use std::cell::RefCell;
use std::fmt::Display;
use std::rc::Rc;

use crate::new_mlem::node::Node;
use crate::new_mlem::qname::QName;

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
    parent: Option<&'a Element<'a>>,
    children: Vec<Node>,
}

impl Element<'_> {
    pub fn new_root_element(name: impl Into<String>) -> Result<Self, super::Error> {
        let qname = QName::new_without_namespace(name)?;

        let inner_element = InnerElement { name: qname };

        Ok(Element {
            inner_element: Rc::new(RefCell::new(inner_element)),
            parent: None,
            children: vec![],
        })
    }
}

impl Clone for Element<'_> {
    fn clone(&self) -> Self {
        let borrow = RefCell::borrow(&*self.inner_element);

        Self {
            inner_element: Rc::new(RefCell::new(borrow.clone())),
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
