pub struct HeadlessBrowserConfig {
    pub proxy_server: Option<String>,
    pub browser_path: Option<String>,
    pub object_store: String,
    pub timeout: i32,
    pub user_agent: Option<String>,
}
