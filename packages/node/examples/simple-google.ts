/**
 * Simple Google Calendar example - OAuth and basic CRUD
 */

import { GoogleCalendarProvider, TokenStorage, TokenData, CalendarSource } from '../src';

// File-based token storage example
import * as fs from 'fs/promises';
import * as path from 'path';

class FileTokenStorage implements TokenStorage {
  private tokenDir = path.join(process.env.HOME || '.', '.calblend');

  async getToken(provider: CalendarSource): Promise<TokenData | null> {
    try {
      const tokenPath = path.join(this.tokenDir, `${provider}.json`);
      const data = await fs.readFile(tokenPath, 'utf-8');
      return JSON.parse(data);
    } catch {
      return null;
    }
  }

  async saveToken(provider: CalendarSource, token: TokenData): Promise<void> {
    await fs.mkdir(this.tokenDir, { recursive: true });
    const tokenPath = path.join(this.tokenDir, `${provider}.json`);
    await fs.writeFile(tokenPath, JSON.stringify(token, null, 2));
  }

  async removeToken(provider: CalendarSource): Promise<void> {
    try {
      const tokenPath = path.join(this.tokenDir, `${provider}.json`);
      await fs.unlink(tokenPath);
    } catch {
      // Ignore if file doesn't exist
    }
  }
}

async function quickStart() {
  // Create provider with your OAuth credentials
  const provider = new GoogleCalendarProvider({
    clientId: process.env.GOOGLE_CLIENT_ID!,
    clientSecret: process.env.GOOGLE_CLIENT_SECRET!,
    redirectUri: 'http://localhost:8080/callback',
    tokenStorage: new FileTokenStorage(),
  });

  // Check if we have a saved token
  const tokenStorage = new FileTokenStorage();
  const existingToken = await tokenStorage.getToken('Google' as CalendarSource);
  
  if (!existingToken) {
    // Need to authenticate
    const authUrl = await provider.getAuthUrl();
    console.log('Visit this URL to authenticate:', authUrl);
    
    // In a real app, you'd handle the OAuth callback
    // For now, manually enter the code
    const code = process.argv[2];
    if (!code) {
      console.log('Usage: npm run example:google -- YOUR_AUTH_CODE');
      process.exit(1);
    }
    
    await provider.exchangeCode(code);
    console.log('✅ Authenticated successfully!');
  }

  // List calendars
  const calendars = await provider.listCalendars();
  console.log('\n📅 Your calendars:');
  calendars.forEach(cal => {
    console.log(`- ${cal.name} (${cal.id})`);
  });

  // Get upcoming events
  const primaryCalendar = calendars.find(c => c.is_primary) || calendars[0];
  if (primaryCalendar) {
    const events = await provider.listEvents(
      primaryCalendar.id,
      new Date(),
      new Date(Date.now() + 7 * 24 * 60 * 60 * 1000) // Next week
    );
    
    console.log(`\n📆 Upcoming events in ${primaryCalendar.name}:`);
    events.forEach(event => {
      const start = new Date(event.start.date_time);
      console.log(`- ${event.title || 'No title'} - ${start.toLocaleString()}`);
    });

    // Create a test event
    const testEvent = {
      id: `test-${Date.now()}`,
      source: 'Google' as const,
      calendar_id: primaryCalendar.id,
      title: 'Test Event from Calblend',
      description: 'This event was created using the Calblend library',
      start: {
        date_time: new Date(Date.now() + 24 * 60 * 60 * 1000).toISOString(), // Tomorrow
        all_day: false,
      },
      end: {
        date_time: new Date(Date.now() + 25 * 60 * 60 * 1000).toISOString(), // Tomorrow + 1 hour
        all_day: false,
      },
    };

    console.log('\n✏️  Creating test event...');
    const created = await provider.createEvent(primaryCalendar.id, testEvent);
    console.log(`✅ Created event: ${created.title} (ID: ${created.id})`);

    // Update the event
    console.log('\n✏️  Updating event...');
    created.title = 'Updated Test Event';
    created.description = 'This event was updated using Calblend';
    const updated = await provider.updateEvent(
      primaryCalendar.id,
      created.id,
      created
    );
    console.log(`✅ Updated event: ${updated.title}`);

    // Delete the event
    console.log('\n🗑️  Deleting test event...');
    await provider.deleteEvent(primaryCalendar.id, created.id);
    console.log('✅ Event deleted successfully!');
  }
}

quickStart().catch(console.error);