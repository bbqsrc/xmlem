mod document;
mod element;
mod node;
mod qname;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid QName: {0}")]
    InvalidQName(String),
}
