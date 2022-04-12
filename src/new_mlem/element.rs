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
pub struct Element {
    inner_element: Rc<RefCell<InnerElement>>,
    children: Vec<Node>,
}

impl Clone for Element {
    fn clone(&self) -> Self {
        let borrow = RefCell::borrow(&*self.inner_element);

        Self {
            inner_element: Rc::new(RefCell::new(borrow.clone())),
            children: self.children.clone(),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        *self = source.clone();
    }
}

impl Display for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&RefCell::borrow(&*self.inner_element), f)?;

        for child in &self.children {
            Display::fmt(&child, f)?;
        }

        Ok(())
    }
}
