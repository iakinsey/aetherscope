use crate::types::{error::AppError, structs::record::Record};

pub trait Worker {
    async fn on_message(&self, message: Record) -> Result<Record, AppError>;
}
