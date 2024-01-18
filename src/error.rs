use std::string::FromUtf8Error;
use inquire::InquireError;

#[derive(Debug)]
pub enum CustomError {
    BtDualBootError(Box<dyn std::error::Error>),
    InquireError(InquireError),
    SerdeError(serde_ini::de::Error),
}


impl Into<CustomError> for InquireError {
    fn into(self) -> CustomError {
        CustomError::InquireError(self)
    }
}

impl Into<CustomError> for &str {
    fn into(self) -> CustomError {
        CustomError::BtDualBootError(self.into())
    }
}

impl Into<CustomError> for String {
    fn into(self) -> CustomError {
        CustomError::BtDualBootError(self.into())
    }
}

impl Into<CustomError> for std::io::Error {
    fn into(self) -> CustomError {
        CustomError::BtDualBootError(self.into())
    }
}

impl Into<CustomError> for FromUtf8Error {
    fn into(self) -> CustomError {
        CustomError::BtDualBootError(self.into())
    }
}

impl Into<CustomError> for serde_ini::de::Error {
    fn into(self) -> CustomError {
        CustomError::SerdeError(self)
    }
}