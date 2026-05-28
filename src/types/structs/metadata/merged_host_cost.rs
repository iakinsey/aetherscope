use crate::types::signals::host_stats_stripe::HostStatsStripe;

pub struct MergedHostCost {
    pub latency_ms_ema: f64,
    pub bytes_ema: f64,
    pub http5xx_ema: f64,
    pub http429_ema: f64,
    pub timeout_ema: f64,
    pub redirect_ema: f64,
}

impl MergedHostCost {
    pub fn from_host_stats_stripes(stripes: &[&HostStatsStripe]) -> MergedHostCost {
        if stripes.is_empty() {
            return MergedHostCost {
                latency_ms_ema: 0.0,
                bytes_ema: 0.0,
                http5xx_ema: 0.0,
                http429_ema: 0.0,
                timeout_ema: 0.0,
                redirect_ema: 0.0,
            };
        }

        let n = stripes.len() as f64;

        MergedHostCost {
            latency_ms_ema: stripes.iter().map(|s| s.latency_ms_ema).sum::<f64>() / n,
            bytes_ema: stripes.iter().map(|s| s.bytes_ema).sum::<f64>() / n,
            http5xx_ema: stripes.iter().map(|s| s.http5xx_ema).sum::<f64>() / n,
            http429_ema: stripes.iter().map(|s| s.http429_ema).sum::<f64>() / n,
            timeout_ema: stripes.iter().map(|s| s.timeout_ema).sum::<f64>() / n,
            redirect_ema: stripes.iter().map(|s| s.redirect_ema).sum::<f64>() / n,
        }
    }
}
