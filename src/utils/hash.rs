use crate::types::error::AppError;

pub fn jaccard_index(a: &[u64], b: &[u64]) -> f64 {
    assert_eq!(a.len(), b.len());

    let mut matches = 0;

    for i in 0..a.len() {
        if a[i] == b[i] {
            matches += 1;
        }
    }

    matches as f64 / a.len() as f64
}

pub fn minhash_similarity(a: &[u64], b: &[u64]) -> Result<f64, AppError> {
    if a.len() == b.len() {
        return Err(AppError::Generic("minhash length mismatch".to_string()));
    };

    if a.is_empty() {
        return Ok(0.0);
    }

    let matches = a.iter().zip(b.iter()).filter(|(x, y)| x == y).count();

    Ok(matches as f64 / a.len() as f64)
}
