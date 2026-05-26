use crate::types::signals::{
    domain_authority_prior::DomainAuthorityPrior, domain_coverage::DomainCoverage,
    host_gate::HostGate, host_stats_stripe::HostStatsStripe, inlink_agg::InlinkAgg,
    prefix_stats::PrefixStats, url_depth::UrlDepth, url_state::UrlState,
};

#[derive(Debug, Clone)]
pub struct SignalsExtracted {
    pub signals: Vec<ExtractedSignal>,
}

#[derive(Debug, Clone)]
pub struct ExtractedSignal {
    pub name: String,
    pub error: Option<String>,
    pub value: ExtractedSignalValue,
}

#[derive(Debug, Clone)]
pub enum ExtractedSignalValue {
    DomainAuthorityPrior(DomainAuthorityPrior),
    DomainCoverage(DomainCoverage),
    HostGate(HostGate),
    HostStatsStripe(HostStatsStripe),
    InlinkAgg(InlinkAgg),
    PrefixStats(PrefixStats),
    UrlDepth(UrlDepth),
    UrlState(UrlState),
}
