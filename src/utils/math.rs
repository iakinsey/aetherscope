pub fn norm_log1p(value: f64, max_value: f64) -> f64 {
    if max_value <= 0.0 {
        return 0.0;
    }

    (value.max(0.0).ln_1p() / max_value.ln_1p()).clamp(0.0, 1.0)
}
