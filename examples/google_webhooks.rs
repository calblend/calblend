//! Example demonstrating Google Calendar webhooks

use calblend_core::{
    CalblendConfig, Calendar, CalendarProvider, CalendarSource,
    auth::InMemoryTokenStorage,
    providers::google::{GoogleCalendarProvider, PushNotification},
};
use std::sync::Arc;
use tracing::info;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    // Configuration from environment
    let client_id = std::env::var("GOOGLE_CLIENT_ID")
        .expect("GOOGLE_CLIENT_ID environment variable required");
    let client_secret = std::env::var("GOOGLE_CLIENT_SECRET")
        .expect("GOOGLE_CLIENT_SECRET environment variable required");
    let webhook_endpoint = std::env::var("WEBHOOK_ENDPOINT")
        .unwrap_or_else(|_| "https://your-domain.com/webhooks/google".to_string());

    // Create token storage and config
    let token_storage = Arc::new(InMemoryTokenStorage::new());
    let config = CalblendConfig::default()
        .with_timeout_seconds(30)
        .with_max_retries(3);

    // Create provider with webhook support
    let provider = GoogleCalendarProvider::new(
        client_id,
        client_secret,
        "http://localhost:8080/callback".to_string(),
        Arc::clone(&token_storage),
        config,
    )?
    .with_webhook_endpoint(webhook_endpoint.clone());

    // Get auth URL
    let auth_url = provider.get_auth_url().await?;
    println!("\n🔐 Please visit this URL to authorize:");
    println!("{}\n", auth_url);

    // In a real app, you'd receive the code via callback
    println!("Enter the authorization code:");
    let mut code = String::new();
    std::io::stdin().read_line(&mut code)?;
    let code = code.trim().to_string();

    // Exchange code for tokens
    provider.exchange_code(code).await?;
    println!("✅ Successfully authenticated!\n");

    // List calendars
    let calendars = provider.list_calendars().await?;
    println!("📅 Found {} calendars:", calendars.len());
    
    for (i, calendar) in calendars.iter().enumerate() {
        println!("  {}. {} ({})", i + 1, calendar.name, calendar.id);
    }

    if let Some(calendar) = calendars.first() {
        println!("\n🔔 Setting up webhook for calendar: {}", calendar.name);
        
        // Create a unique token for this subscription
        let webhook_token = uuid::Uuid::new_v4().to_string();
        
        // Watch the calendar for changes (24 hour TTL)
        let watch_channel = provider.watch_calendar(
            &calendar.id,
            Some(webhook_token.clone()),
            Some(24),
        ).await?;
        
        info!("✅ Webhook created successfully!");
        info!("   Channel ID: {}", watch_channel.id);
        info!("   Resource ID: {}", watch_channel.resource_id);
        info!("   Expiration: {}", watch_channel.expiration);
        info!("   Token: {}", webhook_token);
        
        println!("\n📮 Your webhook endpoint ({}) will now receive notifications", webhook_endpoint);
        println!("   for any changes to calendar: {}\n", calendar.name);
        
        // Simulate processing a webhook notification
        println!("🧪 Simulating webhook notification processing...");
        
        // This is what you'd receive in your webhook handler
        let simulated_notification = PushNotification {
            channel_id: watch_channel.id.clone(),
            channel_token: Some(webhook_token.clone()),
            channel_expiration: Some(watch_channel.expiration.to_rfc3339()),
            resource_id: watch_channel.resource_id.clone(),
            resource_state: "exists".to_string(),
            resource_uri: format!(
                "https://www.googleapis.com/calendar/v3/calendars/{}/events",
                urlencoding::encode(&calendar.id)
            ),
            message_number: Some("1".to_string()),
        };
        
        // Process the notification
        match provider.process_notification(&simulated_notification, Some(&webhook_token)).await {
            Ok(events) => {
                println!("✅ Notification processed successfully!");
                println!("   Found {} recent events", events.len());
            }
            Err(e) => {
                println!("❌ Error processing notification: {}", e);
            }
        }
        
        // Check if renewal is needed
        if calblend_core::providers::google::GoogleWebhookManager::needs_renewal(&watch_channel) {
            println!("\n⏰ Channel expires soon and needs renewal!");
        }
        
        // Stop watching (cleanup)
        println!("\n🛑 Stopping webhook...");
        provider.stop_watch(&watch_channel.id, &watch_channel.resource_id).await?;
        println!("✅ Webhook stopped successfully!");
    }

    Ok(())
}

// Example webhook handler (for your web server)
#[allow(dead_code)]
async fn handle_webhook(
    headers: http::HeaderMap,
    provider: &GoogleCalendarProvider,
    expected_token: &str,
) -> Result<Vec<calblend_core::UnifiedCalendarEvent>, Box<dyn std::error::Error>> {
    // Parse notification from headers
    let notification = calblend_core::providers::google::GoogleWebhookManager::parse_notification_headers(&headers)?;
    
    // Process the notification
    let events = provider.process_notification(&notification, Some(expected_token)).await?;
    
    Ok(events)
}