use thiserror::Error; 

#[derive(Debug, Error)] 
pub enum UsbError {
    #[error("UsbError: {0}")]
    AnyError(String),
}
