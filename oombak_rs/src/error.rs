use thiserror::Error;

use crate::{dut, parser, probe};

pub type OombakResult<T> = Result<T, OombakError>;

#[derive(Debug, Error)]
pub enum OombakError {
    #[error("oombak_rs: libloading: {}", _0)]
    Libloading(libloading::Error),
    #[error("oombak_rs: dut: {}", _0)]
    Dut(dut::Error),
    #[error("oombak_rs: parse: {}", _0)]
    Parser(parser::Error),
    #[error("oombak_rs: probe: {}", _0)]
    Probe(probe::Error),
    #[error("oombak_rs: internal error: {}", _0)]
    InternalError(String),
}

impl From<libloading::Error> for OombakError {
    fn from(error: libloading::Error) -> Self {
        OombakError::Libloading(error)
    }
}
