# Calblend

A unified calendar integration library for Node.js, providing a single API for Google Calendar, Outlook, iOS, and Android calendars.

## Features

- 🚀 **High Performance**: Core engine written in Rust with FFI bindings
- 🔐 **Secure**: Pluggable token storage - you control your credentials
- 📱 **Cross-Platform**: Works with web and native calendar providers
- 🎯 **Type-Safe**: Full TypeScript support with generated types
- 📦 **Zero Dependencies**: No runtime dependencies or external services
- ⚡ **Async/Await**: Modern async API throughout

## Installation

```bash
npm install @calblend/calendar
```

## Quick Start

```typescript
import { createClient, CalendarSource } from '@calblend/calendar';

// Implement your own token storage
const tokenStorage = {
  async getToken(provider: CalendarSource) {
    // Retrieve token from your database
  },
  async saveToken(provider: CalendarSource, token: TokenData) {
    // Save token to your database
  },
  async removeToken(provider: CalendarSource) {
    // Remove token from your database
  }
};

// Create client
const client = createClient({ tokenStorage });

// List calendars
const calendars = await client.listCalendars(CalendarSource.Google);

// Get events
const events = await client.listEvents(
  CalendarSource.Google,
  'primary',
  '2024-01-01T00:00:00Z',
  '2024-12-31T23:59:59Z'
);

// Create event
const newEvent = await client.createEvent(CalendarSource.Google, 'primary', {
  id: 'unique-id',
  source: CalendarSource.Google,
  title: 'Team Meeting',
  start: {
    dateTime: '2024-01-15T10:00:00-08:00',
    allDay: false
  },
  end: {
    dateTime: '2024-01-15T11:00:00-08:00',
    allDay: false
  }
});
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

## Development

### Prerequisites

- Rust 1.88+
- Node.js 20.11.0+
- npm 10.0.0+

### Building

```bash
# Clone the repository
git clone https://github.com/yourusername/calblend.git
cd calblend

# Install dependencies
npm install

# Build everything
npm run build

# Run tests
npm test
```

## Contributing

Contributions are welcome! Please read our [Contributing Guide](CONTRIBUTING.md) for details.

## License

This project is licensed under the Elastic License 2.0 (ELv2). See the [LICENSE](LICENSE) file for details.