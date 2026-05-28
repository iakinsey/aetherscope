#[derive(Debug, Clone)]
pub struct CostScorerConfig {
    pub max_latency_ms: f64,
    pub max_bytes: f64,

    pub latency_weight: f64,
    pub bytes_weight: f64,
    pub timeout_weight: f64,
    pub http5xx_weight: f64,
    pub http429_weight: f64,
    pub redirect_weight: f64,
}
