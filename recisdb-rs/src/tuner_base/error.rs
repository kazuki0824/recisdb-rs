#![allow(dead_code)]

use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub enum GeneralError{
    EnvCompatFailure
}

impl Display for GeneralError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        type E = GeneralError;
        match *self {
            E::EnvCompatFailure=>write!(f, "Compatibility isn't satisfied."),
        }
    }
}

impl std::error::Error for GeneralError
{

}

#[derive(Debug, Clone)]
pub enum BonDriverError{
    OpenError,
    TuneError
}

impl Display for BonDriverError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        type E = BonDriverError;
        match *self {
            E::OpenError=>write!(f, "OpenTuner failed"),
            E::TuneError=>write!(f, "Unable to tune with the specified channel")
        }
    }
}

impl std::error::Error for BonDriverError
{

}