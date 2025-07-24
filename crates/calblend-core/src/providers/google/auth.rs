//! Google OAuth2 authentication

use chrono::{Duration, Utc};
use oauth2::{
    AuthorizationCode, AuthUrl, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge,
    PkceCodeVerifier, RedirectUrl, RefreshToken, RevocationUrl, Scope, TokenResponse,
    TokenUrl, basic::BasicClient, reqwest::async_http_client,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, instrument};

use crate::{
    CalblendError, CalendarSource, Result, auth::TokenData, TokenStorage,
    http::HttpClient,
};

/// Google OAuth2 authentication handler
pub struct GoogleAuth {
    oauth_client: BasicClient,
    token_storage: Arc<dyn TokenStorage>,
    http_client: HttpClient,
    pkce_verifier: RwLock<Option<PkceCodeVerifier>>,
}

impl GoogleAuth {
    /// OAuth2 endpoints
    const AUTH_URL: &'static str = "https://accounts.google.com/o/oauth2/v2/auth";
    const TOKEN_URL: &'static str = "https://oauth2.googleapis.com/token";
    const REVOKE_URL: &'static str = "https://oauth2.googleapis.com/revoke";
    
    /// Required OAuth2 scopes for Google Calendar
    const SCOPES: &'static [&'static str] = &[
        "https://www.googleapis.com/auth/calendar",
        "https://www.googleapis.com/auth/calendar.events",
        "https://www.googleapis.com/auth/calendar.readonly",
    ];

    pub fn new(
        client_id: String,
        client_secret: String,
        redirect_uri: String,
        token_storage: Arc<dyn TokenStorage>,
        http_client: HttpClient,
    ) -> Self {
        let oauth_client = BasicClient::new(
            ClientId::new(client_id),
            Some(ClientSecret::new(client_secret)),
            AuthUrl::new(Self::AUTH_URL.to_string()).unwrap(),
            Some(TokenUrl::new(Self::TOKEN_URL.to_string()).unwrap()),
        )
        .set_redirect_uri(RedirectUrl::new(redirect_uri).unwrap())
        .set_revocation_uri(RevocationUrl::new(Self::REVOKE_URL.to_string()).unwrap());

        Self {
            oauth_client,
            token_storage,
            http_client,
            pkce_verifier: RwLock::new(None),
        }
    }

    /// Generate authorization URL with PKCE
    #[instrument(skip(self))]
    pub async fn get_authorization_url(&self) -> Result<String> {
        // Generate PKCE challenge
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
        
        // Build authorization URL
        let (auth_url, _csrf_token) = self
            .oauth_client
            .authorize_url(CsrfToken::new_random)
            .add_scopes(Self::SCOPES.iter().map(|&s| Scope::new(s.to_string())))
            .set_pkce_challenge(pkce_challenge)
            .url();

        // Store PKCE verifier for later use
        let mut verifier = self.pkce_verifier.write().await;
        *verifier = Some(pkce_verifier);

        debug!("Generated authorization URL");
        Ok(auth_url.to_string())
    }

    /// Exchange authorization code for tokens
    #[instrument(skip(self, code))]
    pub async fn exchange_code(&self, code: String) -> Result<()> {
        let pkce_verifier = {
            let mut verifier = self.pkce_verifier.write().await;
            verifier.take().ok_or_else(|| {
                CalblendError::Authentication("No PKCE verifier found".to_string())
            })?
        };

        let token_result = self
            .oauth_client
            .exchange_code(AuthorizationCode::new(code))
            .set_pkce_verifier(pkce_verifier)
            .request_async(async_http_client)
            .await
            .map_err(|e| CalblendError::Authentication(e.to_string()))?;

        // Convert to our TokenData format
        let token_data = TokenData {
            access_token: token_result.access_token().secret().to_string(),
            refresh_token: token_result.refresh_token().map(|rt| rt.secret().to_string()),
            expires_at: token_result.expires_in().map(|duration| {
                Utc::now() + Duration::seconds(duration.as_secs() as i64)
            }),
            token_type: "Bearer".to_string(),
            scope: token_result.scopes().map(|scopes| {
                scopes
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>()
                    .join(" ")
            }),
        };

        // Store the token
        self.token_storage
            .save_token(CalendarSource::Google, token_data)
            .await?;

        debug!("Successfully exchanged code for tokens");
        Ok(())
    }

    /// Get a valid access token, refreshing if necessary
    #[instrument(skip(self))]
    pub async fn get_access_token(&self) -> Result<String> {
        let token_data = self
            .token_storage
            .get_token(CalendarSource::Google)
            .await?
            .ok_or_else(|| CalblendError::Authentication("No token found".to_string()))?;

        // Check if token is expired
        if token_data.is_expired() {
            debug!("Token expired, refreshing");
            self.refresh_token(token_data).await
        } else {
            Ok(token_data.access_token)
        }
    }

    /// Get valid token (alias for get_access_token for webhook compatibility)
    pub async fn get_valid_token(&self) -> Result<String> {
        self.get_access_token().await
    }

    /// Refresh an expired token
    #[instrument(skip(self, token_data))]
    async fn refresh_token(&self, token_data: TokenData) -> Result<String> {
        let refresh_token = token_data
            .refresh_token
            .clone()
            .ok_or_else(|| CalblendError::Authentication("No refresh token".to_string()))?;

        let token_result = self
            .oauth_client
            .exchange_refresh_token(&RefreshToken::new(refresh_token))
            .request_async(async_http_client)
            .await
            .map_err(|e| CalblendError::Authentication(e.to_string()))?;

        // Update token data
        let new_token_data = TokenData {
            access_token: token_result.access_token().secret().to_string(),
            refresh_token: token_result
                .refresh_token()
                .map(|rt| rt.secret().to_string())
                .or(token_data.refresh_token),
            expires_at: token_result.expires_in().map(|duration| {
                Utc::now() + Duration::seconds(duration.as_secs() as i64)
            }),
            token_type: "Bearer".to_string(),
            scope: token_data.scope,
        };

        // Store updated token
        self.token_storage
            .save_token(CalendarSource::Google, new_token_data.clone())
            .await?;

        debug!("Successfully refreshed token");
        Ok(new_token_data.access_token)
    }

    /// Revoke the stored token
    #[instrument(skip(self))]
    pub async fn revoke_token(&self) -> Result<()> {
        let token_data = self
            .token_storage
            .get_token(CalendarSource::Google)
            .await?
            .ok_or_else(|| CalblendError::Authentication("No token found".to_string()))?;

        // Revoke the token with Google
        let revoke_url = format!("{}?token={}", Self::REVOKE_URL, token_data.access_token);
        let response = self.http_client.client()
            .post(&revoke_url)
            .send()
            .await
            .map_err(|e| CalblendError::InternalError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(CalblendError::Authentication(
                "Failed to revoke token".to_string(),
            ));
        }

        // Remove from storage
        self.token_storage
            .remove_token(CalendarSource::Google)
            .await?;

        debug!("Successfully revoked token");
        Ok(())
    }
}