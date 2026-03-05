use std::sync::Arc;

use cdrs_tokio::query::{BatchQueryBuilder, QueryValues};

use crate::{
    types::{
        error::AppError,
        structs::{record::Record, signal_base::SignalBase},
        traits::object_store::ObjectStore,
    },
    utils::cassandra::DbSession,
};

pub trait Signal: Sized + Send + Sync {
    const CREATE_TABLE_QUERY: &'static str;
    const UPSERT_QUERY: &'static str;

    async fn from_record(
        session: Arc<DbSession>,
        object_store: Arc<dyn ObjectStore>,
        base: SignalBase,
        record: Record,
    ) -> Result<Vec<Self>, AppError>;

    fn bind_values(&self) -> QueryValues;

    async fn create_table(session: Arc<DbSession>) -> Result<(), AppError> {
        session.query(Self::CREATE_TABLE_QUERY).await?;
        Ok(())
    }

    async fn upsert_many(
        session: Arc<DbSession>,
        rows: &[Self],
        batch_size: usize,
    ) -> Result<(), AppError> {
        if rows.is_empty() {
            return Ok(());
        }

        let prepared = session.prepare(Self::UPSERT_QUERY).await?;

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
