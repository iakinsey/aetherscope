use std::sync::Arc;

use chromiumoxide::Browser;
use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::network::EnableParams;
use chromiumoxide::cdp::browser_protocol::network::SetUserAgentOverrideParams;
use fastpool::ManageObject;
use fastpool::ObjectStatus;

use crate::types::configs::headless_browser_config::HeadlessBrowserConfig;
use crate::types::error::AppError;
use crate::utils::web::get_user_agent;

pub struct TabPool<'a> {
    browser: Arc<Browser>,
    config: &'a HeadlessBrowserConfig,
}

impl<'a> TabPool<'a> {
    pub fn new(browser: Arc<Browser>, config: &'a HeadlessBrowserConfig) -> Self {
        Self { browser, config }
    }
}

impl<'a> ManageObject for TabPool<'a> {
    type Object = Page;
    type Error = AppError;

    async fn create(&self) -> Result<Self::Object, Self::Error> {
        let tab = self.browser.new_page("about:blank").await?;

        tab.execute(EnableParams::default()).await?;

        tab.execute(SetUserAgentOverrideParams {
            user_agent: get_user_agent(self.config.user_agent.clone()),
            accept_language: None,
            platform: None,
            user_agent_metadata: None,
        })
        .await?;

        Ok(tab)
    }

    async fn is_recyclable(
        &self,
        _o: &mut Self::Object,
        _status: &ObjectStatus,
    ) -> Result<(), Self::Error> {
        Err(AppError::Generic("discard tab after use".to_string()))
    }
}
