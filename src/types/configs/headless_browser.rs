pub struct HeadlessBrowserConfig {
    pub proxy_server: Option<String>,
    pub browser_path: Option<String>,
    pub object_store: String,
    pub timeout: i32,
    pub user_agent: Option<String>,
}

impl HeadlessBrowserConfig {
    pub fn get_user_agent(&self) -> String {
        if let Some(user_agent) = self.user_agent.clone() {
            user_agent
        } else {
            format!("{} - {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
        }
    }
}
