use crate::types::{
    error::AppError,
    structs::{queue_status::QueueStatus, record::Record},
};

pub trait QueueGettable {
    async fn get() -> Result<Record, AppError>;
}

pub trait QueuePuttable {
    async fn put(record: Record) -> Result<(), AppError>;
}

pub trait QueueStatusReadable {
    async fn status() -> Result<QueueStatus, AppError>;
}
