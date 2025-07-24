//! Authentication and token management

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{CalendarSource, Result};

/// Token data that needs to be persisted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenData {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub token_type: String,
    pub scope: Option<String>,
}

impl TokenData {
    /// Check if the token is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            expires_at <= Utc::now()
        } else {
            false
        }
    }
}

/// Trait that users must implement to provide token storage
#[async_trait]
pub trait TokenStorage: Send + Sync {
    /// Retrieve stored token for a provider
    async fn get_token(&self, provider: CalendarSource) -> Result<Option<TokenData>>;
    
    /// Store token for a provider
    async fn save_token(&self, provider: CalendarSource, token: TokenData) -> Result<()>;
    
    /// Remove token for a provider
    async fn remove_token(&self, provider: CalendarSource) -> Result<()>;
}

/// OAuth configuration for web-based providers
#[derive(Debug, Clone)]
pub struct OAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub auth_url: String,
    pub token_url: String,
    pub scopes: Vec<String>,
}

/// Authentication method
#[derive(Debug, Clone)]
pub enum AuthMethod {
    /// OAuth2 for web APIs (Google, Outlook)
    OAuth(OAuthConfig),
    /// System permissions for mobile platforms
    SystemPermission {
        permission_type: String,
        reason: String,
    },
}

/// In-memory token storage for testing
#[cfg(test)]
pub mod test_utils {
    use super::*;
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};
    
    #[derive(Default)]
    pub struct InMemoryTokenStorage {
        tokens: Arc<Mutex<HashMap<String, TokenData>>>,
    }
    
    #[async_trait]
    impl TokenStorage for InMemoryTokenStorage {
        async fn get_token(&self, provider: CalendarSource) -> Result<Option<TokenData>> {
            let tokens = self.tokens.lock().unwrap();
            Ok(tokens.get(&format!("{:?}", provider)).cloned())
        }
        
        async fn save_token(&self, provider: CalendarSource, token: TokenData) -> Result<()> {
            let mut tokens = self.tokens.lock().unwrap();
            tokens.insert(format!("{:?}", provider), token);
            Ok(())
        }
        
        async fn remove_token(&self, provider: CalendarSource) -> Result<()> {
            let mut tokens = self.tokens.lock().unwrap();
            tokens.remove(&format!("{:?}", provider));
            Ok(())
        }
    }
}