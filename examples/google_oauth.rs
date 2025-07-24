//! Example: Google OAuth flow and basic calendar operations

use calblend_core::{
    CalblendConfig, CalendarProvider, CalendarSource, TokenStorage,
    auth::{TokenData, test_utils::InMemoryTokenStorage},
    providers::GoogleCalendarProvider,
};
use std::sync::Arc;
use tokio::io::{self, AsyncBufReadExt, BufReader};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Configuration - In a real app, these would come from environment variables
    let client_id = std::env::var("GOOGLE_CLIENT_ID")
        .expect("GOOGLE_CLIENT_ID environment variable required");
    let client_secret = std::env::var("GOOGLE_CLIENT_SECRET")
        .expect("GOOGLE_CLIENT_SECRET environment variable required");
    let redirect_uri = "http://localhost:8080/callback".to_string();

    // Create token storage (in production, implement your own persistent storage)
    let token_storage = Arc::new(InMemoryTokenStorage::default());

    // Create the Google Calendar provider
    let provider = GoogleCalendarProvider::new(
        client_id,
        client_secret,
        redirect_uri,
        Arc::clone(&token_storage),
        CalblendConfig::default(),
    )?;

    // Check if we already have a token
    let has_token = token_storage
        .get_token(CalendarSource::Google)
        .await?
        .is_some();

    if !has_token {
        // Get the authorization URL
        let auth_url = provider.get_auth_url().await?;
        
        println!("\nPlease visit this URL to authorize the application:");
        println!("{}\n", auth_url);
        println!("After authorization, you'll be redirected to a callback URL.");
        println!("Copy the 'code' parameter from that URL and paste it here:");
        
        // Read the authorization code from stdin
        let stdin = io::stdin();
        let mut reader = BufReader::new(stdin);
        let mut code = String::new();
        reader.read_line(&mut code).await?;
        let code = code.trim().to_string();
        
        // Exchange the code for tokens
        println!("\nExchanging code for tokens...");
        provider.exchange_code(code).await?;
        println!("✅ Successfully authenticated!");
    }

    // List calendars
    println!("\n📅 Your calendars:");
    let calendars = provider.list_calendars().await?;
    for (i, calendar) in calendars.iter().enumerate() {
        println!("  {}. {} ({})", i + 1, calendar.name, calendar.id);
        if calendar.is_primary {
            println!("     ^ Primary calendar");
        }
    }

    // Get the primary calendar
    let primary_calendar = calendars
        .iter()
        .find(|c| c.is_primary)
        .or_else(|| calendars.first())
        .ok_or("No calendars found")?;

    // List upcoming events
    println!("\n📆 Upcoming events in '{}':", primary_calendar.name);
    let events = provider
        .list_events(&primary_calendar.id, None, None)
        .await?;

    if events.is_empty() {
        println!("  No upcoming events");
    } else {
        for event in events.iter().take(10) {
            let title = event.title.as_deref().unwrap_or("(No title)");
            let start = &event.start.date_time;
            println!("  - {} at {}", title, start.format("%Y-%m-%d %H:%M"));
            
            if let Some(location) = &event.location {
                println!("    Location: {}", location);
            }
            
            if let Some(attendees) = &event.attendees {
                println!("    Attendees: {}", attendees.len());
            }
        }
    }

    // Get free/busy information
    println!("\n⏱️  Free/busy for the next 7 days:");
    let now = chrono::Utc::now();
    let week_later = now + chrono::Duration::days(7);
    
    let free_busy = provider
        .get_free_busy(&[primary_calendar.id.clone()], now, week_later)
        .await?;

    if free_busy.is_empty() {
        println!("  No busy periods");
    } else {
        for period in free_busy.iter().take(10) {
            println!(
                "  - Busy: {} to {}",
                period.start.format("%Y-%m-%d %H:%M"),
                period.end.format("%Y-%m-%d %H:%M")
            );
        }
    }

    Ok(())
}

// Instructions for running this example:
// 1. Set up Google OAuth credentials:
//    - Go to https://console.cloud.google.com/
//    - Create a new project or select existing
//    - Enable Google Calendar API
//    - Create OAuth 2.0 credentials
//    - Add http://localhost:8080/callback to authorized redirect URIs
//
// 2. Set environment variables:
//    export GOOGLE_CLIENT_ID="your-client-id"
//    export GOOGLE_CLIENT_SECRET="your-client-secret"
//
// 3. Run the example:
//    cargo run --example google_oauth