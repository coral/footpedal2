use thiserror::Error;

#[derive(Error, Debug)]
pub enum DataStoreError {
    #[error("the data for key `{0}` is not available")]
    Redaction(String),
}
