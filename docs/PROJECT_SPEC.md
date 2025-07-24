# Calblend Project Specification

## Core Purpose
Open-source library enabling developers to integrate with calendars across Android/iOS native calendars, Outlook, and Gmail. Provides unified API for reading calendar data, managing events, and maintaining real-time synchronization.

## Key Features

### 1. Calendar Operations

**Read Operations**
- Fetch events by time range (day, week, month, year, custom)
- Get detailed event information (title, description, location, attendees, attachments)
- Query free/busy time blocks
- Search events by text, attendee, or location
- Handle recurring events and patterns
- Access calendar metadata (name, color, timezone, permissions)

**Write Operations**
- Create single and recurring events
- Update event properties
- Delete events (single instance or series)
- Move events between calendars
- RSVP to invitations
- Manage attendees

### 2. Multi-Calendar Support

**Aggregation Utilities**
- Get free/busy data across multiple calendars for a single user
- Batch operations for multiple users' availability
- Efficient event fetching across multiple calendars
- Merged calendar view generation

### 3. Real-time Synchronization
- Event change notifications via webhooks/push
- Delta sync for efficient updates
- Sync state management
- Offline queue for pending changes
- Conflict resolution strategies

### 4. Platform Integration

**Supported Platforms**
- iOS (EventKit)
- Android (Calendar Provider API)
- Outlook (Microsoft Graph API)
- Google Calendar (Google Calendar API)

**Authentication**
- OAuth 2.0 flow helpers for each platform
- Token refresh handling
- Multi-account support
- Permission scope management

### 5. Storage Interface

**Pluggable Storage System**

The library requires developers to implement storage methods for:
- OAuth token persistence
- Token retrieval by user and platform
- Optional calendar data caching
- Cache invalidation

## Architecture

### Data Models
- Unified event format across platforms
- Standardized timezone handling (IANA timezone database)
- Normalized attendee statuses
- Common error types across platforms

### Performance Features
- Intelligent batching for API calls
- Rate limit management per platform
- Suggested caching strategies with TTL hints
- Parallel request optimization

### Developer Experience
- Platform-specific error mapping to common error types
- Comprehensive error messages
- Extensive debug logging capabilities
- Clear documentation for each platform's limitations

## What's NOT Included
- Database implementation
- Custom availability rules
- Business logic for scheduling
- UI components
- Hosting infrastructure
- User management

## Core Operations

### Authentication Flow
1. Initialize library with platform credentials
2. Generate OAuth URL for user authorization
3. Handle OAuth callback
4. Store tokens using provided storage interface
5. Automatic token refresh

### Basic Event Fetching
1. Retrieve user's calendars
2. Fetch events within time range
3. Return normalized event data
4. Handle pagination for large result sets

### Free/Busy Queries
1. Accept user ID and calendar IDs
2. Query specified time range
3. Return busy time blocks
4. Support batch queries for multiple users

### Real-time Updates
1. Register webhook endpoints (where supported)
2. Handle platform-specific push notifications
3. Provide unified change event format
4. Queue changes when offline

## Platform-Specific Considerations

**iOS/Android**
- Native module requirements
- Privacy permission handling
- Background sync limitations
- Local calendar database access

**Outlook/Gmail**
- API quotas and rate limits
- Webhook endpoint requirements
- Service account options
- Delegation support