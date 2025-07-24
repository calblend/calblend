# Calblend

A unified calendar integration library for Node.js, providing a single API for Google Calendar, Outlook, iOS, and Android calendars.

## Features

- 🚀 **High Performance**: Core engine written in Rust with FFI bindings
- 🔐 **Secure**: Pluggable token storage - you control your credentials
- 📱 **Cross-Platform**: Works with web and native calendar providers
- 🎯 **Type-Safe**: Full TypeScript support with generated types
- 📦 **Zero Dependencies**: No runtime dependencies or external services
- ⚡ **Async/Await**: Modern async API throughout
- 🔄 **Real-time Sync**: Webhook support for instant updates
- 💾 **Smart Caching**: Built-in cache with automatic invalidation
- 🛡️ **Rate Limiting**: Automatic rate limit handling per provider
- 🔒 **OAuth 2.0**: PKCE support for enhanced security

## Installation

```bash
npm install @calblend/calendar
```

## Current Status

🚧 **Alpha Release** - Google Calendar provider is fully implemented with OAuth, webhooks, and caching. Other providers coming soon.

### Supported Providers
- ✅ **Google Calendar** - Complete with OAuth 2.0, real-time sync, caching
- 🚧 **Outlook** - In development
- 📅 **iOS (EventKit)** - Planned
- 📅 **Android** - Planned

## Quick Start

```typescript
import { GoogleCalendarProvider, TokenStorage } from '@calblend/calendar';

// Implement your own secure token storage
class MyTokenStorage implements TokenStorage {
  async getToken(provider: CalendarSource) {
    // Retrieve from your database
    return await db.tokens.findOne({ provider });
  }
  async saveToken(provider: CalendarSource, token: TokenData) {
    // Save to your database
    await db.tokens.upsert({ provider }, token);
  }
  async removeToken(provider: CalendarSource) {
    // Remove from your database
    await db.tokens.delete({ provider });
  }
}

// Initialize Google Calendar provider
const google = new GoogleCalendarProvider({
  clientId: process.env.GOOGLE_CLIENT_ID,
  clientSecret: process.env.GOOGLE_CLIENT_SECRET,
  redirectUri: 'http://localhost:3000/auth/google/callback',
  tokenStorage: new MyTokenStorage(),
  webhookEndpoint: 'https://api.myapp.com/webhooks/google' // Optional
});

// OAuth flow
const authUrl = await google.getAuthUrl();
// Redirect user to authUrl...
// After callback with code:
await google.exchangeCode(code);

// List calendars
const calendars = await google.listCalendars();

// Get events with automatic caching
const events = await google.listEvents(
  'primary',
  '2024-01-01T00:00:00Z',
  '2024-12-31T23:59:59Z'
);

// Real-time updates with webhooks
const watchChannel = await google.watchCalendar('primary');
// Handle webhook notifications at your endpoint
```

## Token Storage

Calblend requires you to implement secure token storage. This gives you complete control over your sensitive OAuth credentials.

```typescript
import { TokenStorage, TokenData, CalendarSource } from '@calblend/calendar';

// Example: Database-backed token storage
class DatabaseTokenStorage implements TokenStorage {
  async getToken(provider: CalendarSource): Promise<TokenData | null> {
    const token = await db.tokens.findOne({ 
      provider,
      userId: currentUser.id 
    });
    
    if (!token) return null;
    
    return {
      accessToken: token.accessToken,
      refreshToken: token.refreshToken,
      expiresAt: token.expiresAt,
      tokenType: 'Bearer',
      scope: token.scope
    };
  }

  async saveToken(provider: CalendarSource, token: TokenData): Promise<void> {
    await db.tokens.upsert(
      { provider, userId: currentUser.id },
      {
        accessToken: token.accessToken,
        refreshToken: token.refreshToken,
        expiresAt: token.expiresAt,
        scope: token.scope,
        updatedAt: new Date()
      }
    );
  }

  async removeToken(provider: CalendarSource): Promise<void> {
    await db.tokens.delete({ 
      provider, 
      userId: currentUser.id 
    });
  }
}

// Example: Encrypted file storage for desktop apps
class EncryptedFileStorage implements TokenStorage {
  private filePath = path.join(app.getPath('userData'), 'tokens.enc');
  
  async getToken(provider: CalendarSource): Promise<TokenData | null> {
    const encrypted = await fs.readFile(this.filePath, 'utf8');
    const tokens = decrypt(encrypted, masterKey);
    return tokens[provider] || null;
  }
  // ... implement saveToken and removeToken
}
```

### Security Best Practices

- **Never store tokens in plain text**
- **Implement encryption at rest** for token storage
- **Use secure key management** (e.g., AWS KMS, HashiCorp Vault)
- **Set appropriate database permissions**
- **Implement token rotation** when refresh tokens expire
- **Log token usage** for security auditing

## Webhooks / Real-time Sync

Calblend supports Google Calendar push notifications for real-time updates.

### Setting Up Webhooks

```typescript
// 1. Configure webhook endpoint during initialization
const google = new GoogleCalendarProvider({
  clientId: process.env.GOOGLE_CLIENT_ID,
  clientSecret: process.env.GOOGLE_CLIENT_SECRET,
  redirectUri: 'http://localhost:3000/auth/google/callback',
  tokenStorage: new MyTokenStorage(),
  webhookEndpoint: 'https://api.myapp.com/webhooks/google'
});

// 2. Start watching a calendar
const watchChannel = await google.watchCalendar('primary', {
  token: 'my-secure-verification-token', // Optional: for verification
  ttlHours: 24 * 7 // Optional: max 7 days for Google
});

console.log('Watch channel:', {
  id: watchChannel.id,
  resourceId: watchChannel.resourceId,
  expiration: watchChannel.expiration
});

// 3. Handle webhooks in your server
app.post('/webhooks/google', async (req, res) => {
  const notification = {
    channelId: req.headers['x-goog-channel-id'],
    channelToken: req.headers['x-goog-channel-token'],
    channelExpiration: req.headers['x-goog-channel-expiration'],
    resourceId: req.headers['x-goog-resource-id'],
    resourceState: req.headers['x-goog-resource-state'],
    resourceUri: req.headers['x-goog-resource-uri'],
    messageNumber: req.headers['x-goog-message-number']
  };

  // Verify the notification (important for security!)
  if (notification.channelToken !== 'my-secure-verification-token') {
    return res.status(401).send('Unauthorized');
  }

  // Process the notification
  if (notification.resourceState === 'sync') {
    console.log('Initial sync notification received');
  } else if (notification.resourceState === 'exists') {
    // Fetch recent changes
    const events = await google.processNotification(notification);
    console.log('Calendar changed, fetched events:', events.length);
  }

  res.status(200).send();
});

// 4. Stop watching when done
await google.stopWatch(watchChannel.id, watchChannel.resourceId);
```

### Webhook Security

- **Use HTTPS only** for webhook endpoints
- **Verify channel tokens** to prevent unauthorized notifications
- **Implement idempotency** to handle duplicate notifications
- **Set up SSL certificates** with proper validation
- **Rate limit webhook endpoints** to prevent abuse

## OAuth 2.0 Flow

### Complete OAuth Implementation

```typescript
// 1. Initialize provider
const google = new GoogleCalendarProvider({
  clientId: process.env.GOOGLE_CLIENT_ID,
  clientSecret: process.env.GOOGLE_CLIENT_SECRET,
  redirectUri: 'http://localhost:3000/auth/google/callback',
  tokenStorage: new MyTokenStorage()
});

// 2. Generate authorization URL
app.get('/auth/google', async (req, res) => {
  const authUrl = await google.getAuthUrl();
  // Store state in session for CSRF protection
  req.session.oauthState = crypto.randomBytes(32).toString('hex');
  res.redirect(authUrl);
});

// 3. Handle OAuth callback
app.get('/auth/google/callback', async (req, res) => {
  const { code, state } = req.query;
  
  // Verify state for CSRF protection
  if (state !== req.session.oauthState) {
    return res.status(400).send('Invalid state parameter');
  }
  
  try {
    // Exchange code for tokens
    await google.exchangeCode(code);
    res.redirect('/calendar');
  } catch (error) {
    console.error('OAuth error:', error);
    res.status(500).send('Authentication failed');
  }
});

// 4. Token refresh is automatic
// When tokens expire, Calblend automatically refreshes them
const calendars = await google.listCalendars(); // Auto-refreshes if needed
```

### OAuth Security Features

- **PKCE (Proof Key for Code Exchange)** - Enhanced security for public clients
- **Automatic token refresh** - Seamless token management
- **Secure token storage** - You control where tokens are stored
- **State parameter** - CSRF protection built-in
- **Scope management** - Request only necessary permissions

## Data Caching

Calblend includes intelligent caching to reduce API calls and improve performance.

### Cache Configuration

```typescript
// Default: 60-minute cache
const google = new GoogleCalendarProvider({ /* ... */ });

// Custom cache TTL
const shortCache = google.withCacheTTL(5); // 5 minutes
const longCache = google.withCacheTTL(240); // 4 hours

// Disable cache for real-time requirements
const noCache = google.withoutCache();
```

### What Gets Cached

- **Calendar List** - TTL: 60 minutes (default)
- **Events** - TTL: 60 minutes, invalidated on create/update/delete
- **Free/Busy Data** - TTL: 60 minutes

### Cache Invalidation

```typescript
// Cache is automatically invalidated on mutations
await google.createEvent('primary', newEvent); // Clears event cache
await google.updateEvent('primary', eventId, updates); // Clears event cache
await google.deleteEvent('primary', eventId); // Clears event cache

// Manual cache control (if needed in future versions)
// await google.clearCache('primary'); // Coming soon
```

## Architecture

```
calblend/
├── crates/
│   ├── calblend-core/     # Rust calendar engine
│   └── calblend-ffi/       # N-API FFI bindings
├── packages/
│   └── node/               # TypeScript SDK
└── examples/               # Example applications
```

## API Documentation

### Core Features

- **OAuth 2.0 Authentication** with PKCE support
- **Event Management**: Create, read, update, delete calendar events
- **Calendar Operations**: List and manage multiple calendars
- **Free/Busy Queries**: Check availability across calendars
- **Real-time Sync**: Webhook support for instant updates
- **Smart Caching**: Automatic cache with configurable TTL
- **Rate Limiting**: Built-in rate limit handling
- **Batch Operations**: Efficient bulk operations (coming soon)

### Advanced Configuration

```typescript
// Configure caching and webhooks
const google = new GoogleCalendarProvider({
  // ... basic config ...
  cacheMinutes: 30, // Default: 60
  maxRetries: 5,    // Default: 3
  timeout: 60000,   // Default: 30000ms
});

// Disable caching for specific operations
const provider = google.withoutCache();

// Custom cache TTL
const cachedProvider = google.withCacheTTL(120); // 2 hours
```

## Development

### Prerequisites

- Rust 1.88+
- Node.js 20.11.0+
- npm 10.0.0+

### Building

```bash
# Clone the repository
git clone https://github.com/calblend/calblend.git
cd calblend

# Install dependencies
npm install

# Build Rust core and FFI
cargo build --workspace

# Build TypeScript SDK
npm run build -w @calblend/calendar

# Run tests
cargo test --workspace
npm test -w @calblend/calendar

# Run example
cargo run --example google_oauth
```

## Contributing

Contributions are welcome! Please read our [Contributing Guide](CONTRIBUTING.md) for details.

## License

This project is licensed under the Elastic License 2.0 (ELv2). See the [LICENSE](LICENSE) file for details.