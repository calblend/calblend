//! FFI-safe token storage implementation

use async_trait::async_trait;
use calblend_core::{CalendarSource, TokenStorage, Result as CoreResult};
use calblend_core::auth::TokenData;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Simple in-memory token storage for FFI
#[derive(Clone)]
pub struct JsTokenStorage {
    tokens: Arc<Mutex<HashMap<String, TokenData>>>,
}

impl JsTokenStorage {
    pub fn new() -> Self {
        Self {
            tokens: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl TokenStorage for JsTokenStorage {
    async fn get_token(&self, provider: CalendarSource) -> CoreResult<Option<TokenData>> {
        let tokens = self.tokens.lock().await;
        Ok(tokens.get(&calendar_source_to_string(&provider)).cloned())
    }

    async fn save_token(&self, provider: CalendarSource, token: TokenData) -> CoreResult<()> {
        let mut tokens = self.tokens.lock().await;
        tokens.insert(calendar_source_to_string(&provider), token);
        Ok(())
    }

    async fn remove_token(&self, provider: CalendarSource) -> CoreResult<()> {
        let mut tokens = self.tokens.lock().await;
        tokens.remove(&calendar_source_to_string(&provider));
        Ok(())
    }
}

// Helper function to convert CalendarSource to string
fn calendar_source_to_string(source: &CalendarSource) -> String {
    match source {
        CalendarSource::Google => "google".to_string(),
        CalendarSource::Outlook => "outlook".to_string(),
        CalendarSource::Ios => "ios".to_string(),
        CalendarSource::Android => "android".to_string(),
    }
}