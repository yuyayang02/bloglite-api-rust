#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    HandlerPainc(String),
    #[error("no subsrcibers.")]
    NoSubsrcibers,
}

#[derive(Debug, thiserror::Error)]
pub enum HandleError {
    #[error("{0}")]
    Wrap(String),
}
