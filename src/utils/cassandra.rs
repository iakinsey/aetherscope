use crate::types::error::AppError;
use cdrs_tokio::types::IntoRustByName;
use cdrs_tokio::types::prelude::Row;

pub fn get_fp_minhash(row: &Row, name: &str) -> Result<Option<Vec<u64>>, AppError> {
    Ok({
        let s: Option<String> = row.get_by_name(name)?;

        s.map(|txt| {
            txt.split(',')
                .filter(|x| !x.is_empty())
                .map(|x| x.parse::<u64>().unwrap())
                .collect()
        })
    })
}
