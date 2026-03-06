pub struct SignalExtractorConfig<'a> {
    pub signals: Vec<&'a str>,
    pub object_store: String,
    pub db_session: String,
}
