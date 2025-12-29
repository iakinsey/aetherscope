pub fn get_user_agent(user_agent: Option<String>) -> String {
    if let Some(user_agent) = user_agent {
        user_agent
    } else {
        format!("{} - {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
    }
}
