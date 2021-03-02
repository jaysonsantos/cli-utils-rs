use std::result::Result as StdResult;
use thiserror::Error as ThisError;

pub type Result<T> = StdResult<T, Error>;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("Key was not present")]
    KeyNotPresent,

    // Use eyre just to easily wrap rusoto as it has typed errors
    #[error(transparent)]
    RusotoError(#[from] eyre::Error),
}
