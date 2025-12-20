use std::sync::Arc;

use chromiumoxide::Browser;
use chromiumoxide::Page;
use fastpool::ManageObject;
use fastpool::ObjectStatus;

use crate::types::error::AppError;

pub struct TabPool {
    browser: Arc<Browser>,
}

impl TabPool {
    pub fn new(browser: Arc<Browser>) -> Self {
        Self { browser }
    }
}

impl ManageObject for TabPool {
    type Object = Page;
    type Error = AppError;

    async fn create(&self) -> Result<Self::Object, Self::Error> {
        Ok(self.browser.new_page("about:blank").await?)
    }

    async fn is_recyclable(
        &self,
        _o: &mut Self::Object,
        _status: &ObjectStatus,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}
