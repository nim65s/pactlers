#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("nb error")]
    Nb(nb::Error<()>),
}
