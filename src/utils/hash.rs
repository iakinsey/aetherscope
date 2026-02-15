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
