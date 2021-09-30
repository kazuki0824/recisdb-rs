use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub enum BonDriverError{
    OpenError
}

impl Display for BonDriverError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            BonDriverError::OpenError=>write!(f, "OpenTuner failed")
        }
    }
}

impl std::error::Error for BonDriverError
{

}
