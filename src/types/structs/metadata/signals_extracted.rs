#[derive(Debug, Clone)]
pub struct SignalsExtracted {
    pub signals: Vec<ExtractedSignal>,
}

#[derive(Debug, Clone)]
pub struct ExtractedSignal {
    pub name: String,
    pub error: Option<String>,
}
