use petgraph::prelude::NodeIndex;
use thiserror::Error;

pub(crate) type Result<T> = std::result::Result<T, TopologicalOrderingError>;

#[derive(Error, Debug)]
pub enum TopologicalOrderingError {
    #[error("cycle detected")]
    CycleDetected,
    #[error("No such item: {0:?}")]
    NoSuchItem(NodeIndex<u32>),
}
