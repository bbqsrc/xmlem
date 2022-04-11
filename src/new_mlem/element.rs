use std::rc::Rc;
use std::cell::RefCell;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct QName {
    pub name: String,
}

#[derive(Debug)]
pub struct InnerElement {
    name: QName,
}

#[derive(Debug)]
pub struct Element {
    inner_element: Rc<RefCell<InnerElement>>,
}
