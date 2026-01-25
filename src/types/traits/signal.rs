use crate::types::error::AppError;

pub trait Signal {
    fn get_query() -> String;
    fn create_table() -> Result<(), AppError>;
}
