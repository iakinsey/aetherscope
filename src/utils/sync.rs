use std::sync::Arc;

use chromiumoxide::Browser;
use chromiumoxide::Page;
use chromiumoxide::cdp::browser_protocol::network::EnableParams;
use fastpool::ManageObject;
use fastpool::ObjectStatus;

use crate::types::error::AppError;

pub struct TabPool {
    browser: Arc<Browser>,
    enable_network_events: bool,
}

impl TabPool {
    pub fn new(browser: Arc<Browser>, enable_network_events: bool) -> Self {
        Self {
            browser,
            enable_network_events,
        }
    }
}

impl ManageObject for TabPool {
    type Object = Page;
    type Error = AppError;

    async fn create(&self) -> Result<Self::Object, Self::Error> {
        let tab = self.browser.new_page("about:blank").await?;

        if self.enable_network_events {
            tab.execute(EnableParams::default()).await?;
        }

        Ok(tab)
    }

    async fn is_recyclable(
        &self,
        _o: &mut Self::Object,
        _status: &ObjectStatus,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}
