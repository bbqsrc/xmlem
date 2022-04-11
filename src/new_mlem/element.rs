use std::borrow::Borrow;
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct QName {
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct InnerElement {
    name: QName,
}

#[derive(Debug)]
pub struct Element {
    inner_element: Rc<RefCell<InnerElement>>,
}

impl Clone for Element {
    fn clone(&self) -> Self {
        let borrow = RefCell::borrow(&*self.inner_element);

        Self {
            inner_element: Rc::new(RefCell::new(borrow.clone())),
        }
    }

    fn clone_from(&mut self, source: &Self) {
        *self = source.clone();
    }
}
