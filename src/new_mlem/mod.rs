mod attribute;
mod document;
mod element;
mod node;
mod qname;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("error from quick_xml: `{0}`")]
    Disconnect(#[from] quick_xml::Error),

    #[error("Invalid QName: {0}")]
    InvalidQName(String),
}
