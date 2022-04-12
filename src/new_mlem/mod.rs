pub mod attribute;
pub mod document;
pub mod element;
pub mod node;
pub mod qname;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("error from quick_xml: `{0}`")]
    QuickXml(#[from] quick_xml::Error),

    #[error("Invalid QName: {0}")]
    InvalidQName(String),

    #[error("UTF-8 error from std: `{0}`")]
    StdUtf8(#[from] std::str::Utf8Error),
}
