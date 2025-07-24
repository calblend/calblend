/**
 * Basic example of using Calblend
 */

import { createClient, CalendarSource, TokenStorage, TokenData } from '@calblend/calendar';

// Simple in-memory token storage for demo purposes
class InMemoryTokenStorage implements TokenStorage {
  private tokens = new Map<CalendarSource, TokenData>();

  async getToken(provider: CalendarSource): Promise<TokenData | null> {
    return this.tokens.get(provider) || null;
  }

  async saveToken(provider: CalendarSource, token: TokenData): Promise<void> {
    this.tokens.set(provider, token);
  }

  async removeToken(provider: CalendarSource): Promise<void> {
    this.tokens.delete(provider);
  }
}

async function main() {
  console.log('🚀 Calblend Basic Example\n');

  // Create token storage
  const tokenStorage = new InMemoryTokenStorage();

  // Create client
  const client = createClient({ tokenStorage });

  console.log('✅ Client created successfully\n');

  // Example: List calendars (will be empty in this demo)
  try {
    console.log('📅 Listing Google calendars...');
    const calendars = await client.listCalendars(CalendarSource.Google);
    console.log(`Found ${calendars.length} calendars\n`);
  } catch (error) {
    console.error('Error listing calendars:', error);
  }

  // Example: Create an event object (local only in this demo)
  const event = {
    id: 'demo-event-001',
    source: CalendarSource.Google,
    title: 'Team Standup',
    description: 'Daily team sync meeting',
    location: 'Conference Room A',
    start: {
      dateTime: new Date('2024-01-15T10:00:00-08:00').toISOString(),
      allDay: false,
    },
    end: {
      dateTime: new Date('2024-01-15T10:30:00-08:00').toISOString(),
      allDay: false,
    },
    attendees: [
      {
        email: 'team@example.com',
        responseStatus: 'NeedsAction' as const,
      },
    ],
    reminders: [
      {
        minutesBefore: 10,
        method: 'Popup' as const,
      },
    ],
  };

  console.log('📝 Created event object:');
  console.log(JSON.stringify(event, null, 2));
  console.log('\n');

  // In a real implementation, this would create the event in Google Calendar
  console.log('✨ Demo complete! In production, this would sync with actual calendars.');
}

// Run the example
main().catch(console.error);