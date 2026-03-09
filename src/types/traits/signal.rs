use std::sync::Arc;

use async_trait::async_trait;
use cdrs_tokio::query::{BatchQueryBuilder, QueryValues};

use crate::{
    types::{
        error::AppError,
        structs::{record::Record, signal_base::SignalBase},
        traits::object_store::ObjectStore,
    },
    utils::cassandra::DbSession,
};

#[async_trait]
pub trait Signal: Send + Sync {
    fn create_table_query() -> &'static str
    where
        Self: Sized;

    fn upsert_query() -> &'static str
    where
        Self: Sized;

    async fn from_record(
        session: Arc<DbSession>,
        object_store: Arc<dyn ObjectStore>,
        base: SignalBase,
        record: Record,
    ) -> Result<Vec<Box<dyn Signal>>, AppError>
    where
        Self: Sized;

    fn bind_values(&self) -> QueryValues;

    async fn create_table(session: Arc<DbSession>) -> Result<(), AppError>
    where
        Self: Sized,
    {
        session.query(Self::create_table_query()).await?;
        Ok(())
    }

    async fn upsert_many(
        session: Arc<DbSession>,
        rows: &[Self],
        batch_size: usize,
    ) -> Result<(), AppError>
    where
        Self: Sized,
    {
        if rows.is_empty() {
            return Ok(());
        }

        let prepared = session.prepare(Self::upsert_query()).await?;

        for chunk in rows.chunks(batch_size.max(1)) {
            let mut b = BatchQueryBuilder::new();
            for r in chunk {
                b = b.add_query_prepared(&prepared, r.bind_values());
            }
            let batch = b.build()?;
            session.batch(batch).await?;
        }

        Ok(())
    }
}
