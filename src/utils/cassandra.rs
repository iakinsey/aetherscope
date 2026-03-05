use std::sync::Arc;

use crate::types::error::AppError;
use cdrs_tokio::cluster::session::TcpSessionBuilder;
use cdrs_tokio::types::IntoRustByName;
use cdrs_tokio::types::prelude::Row;
use cdrs_tokio::{
    cluster::{TcpConnectionManager, session::Session},
    load_balancing::RoundRobinLoadBalancingStrategy,
    transport::TransportTcp,
};

use cdrs_tokio::cluster::NodeTcpConfigBuilder;

use cdrs_tokio::cluster::session::SessionBuilder;

pub type DbSession = Session<
    TransportTcp,
    TcpConnectionManager,
    RoundRobinLoadBalancingStrategy<TransportTcp, TcpConnectionManager>,
>;

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

pub async fn create_session(contact_point: &str) -> Result<Arc<DbSession>, AppError> {
    let cluster_config = NodeTcpConfigBuilder::new()
        .with_contact_point(contact_point.into())
        .build()
        .await
        .map_err(|e| AppError::Generic(format!("node config build failed: {e:?}")))?;

    let session = TcpSessionBuilder::new(RoundRobinLoadBalancingStrategy::new(), cluster_config)
        .build()
        .await
        .map_err(|e| AppError::Generic(format!("session build failed: {e:?}")))?;

    Ok(Arc::new(session))
}
