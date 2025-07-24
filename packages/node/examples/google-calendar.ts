/**
 * Example: Google Calendar integration with webhooks
 */

import { GoogleCalendarProvider, TokenStorage, TokenData, CalendarSource } from '../src';
import * as readline from 'readline/promises';
import { stdin as input, stdout as output } from 'process';

// Simple in-memory token storage
class InMemoryTokenStorage implements TokenStorage {
  private tokens = new Map<string, TokenData>();

  async getToken(provider: CalendarSource): Promise<TokenData | null> {
    return this.tokens.get(provider.toString()) || null;
  }

  async saveToken(provider: CalendarSource, token: TokenData): Promise<void> {
    this.tokens.set(provider.toString(), token);
  }

  async removeToken(provider: CalendarSource): Promise<void> {
    this.tokens.delete(provider.toString());
  }
}

async function main() {
  // Configuration
  const clientId = process.env.GOOGLE_CLIENT_ID;
  const clientSecret = process.env.GOOGLE_CLIENT_SECRET;
  const webhookEndpoint = process.env.WEBHOOK_ENDPOINT || 'https://your-domain.com/webhooks/google';

  if (!clientId || !clientSecret) {
    console.error('Please set GOOGLE_CLIENT_ID and GOOGLE_CLIENT_SECRET environment variables');
    process.exit(1);
  }

  // Create provider
  const provider = new GoogleCalendarProvider({
    clientId,
    clientSecret,
    redirectUri: 'http://localhost:8080/callback',
    tokenStorage: new InMemoryTokenStorage(),
    webhookEndpoint,
  });

  console.log('🔐 Google Calendar OAuth Example\n');

  // Get auth URL
  const authUrl = await provider.getAuthUrl();
  console.log('Please visit this URL to authorize:');
  console.log(authUrl);
  console.log();

  // Get code from user
  const rl = readline.createInterface({ input, output });
  const code = await rl.question('Enter the authorization code: ');
  rl.close();

  // Exchange code for tokens
  await provider.exchangeCode(code);
  console.log('✅ Successfully authenticated!\n');

  // List calendars
  const calendars = await provider.listCalendars();
  console.log(`📅 Found ${calendars.length} calendars:`);
  calendars.forEach((cal, i) => {
    console.log(`  ${i + 1}. ${cal.name} (${cal.id})`);
    console.log(`     Primary: ${cal.is_primary}, Can write: ${cal.can_write}`);
  });

  // Get events from primary calendar
  const primaryCalendar = calendars.find(cal => cal.is_primary) || calendars[0];
  if (primaryCalendar) {
    console.log(`\n📆 Getting events from: ${primaryCalendar.name}`);
    
    const startDate = new Date();
    const endDate = new Date();
    endDate.setDate(endDate.getDate() + 7); // Next 7 days

    const events = await provider.listEvents(
      primaryCalendar.id,
      startDate,
      endDate
    );

    console.log(`Found ${events.length} events in the next week:`);
    events.forEach(event => {
      console.log(`  - ${event.title || 'Untitled'}`);
      console.log(`    Start: ${event.start.date_time}`);
      console.log(`    ${event.location ? `Location: ${event.location}` : ''}`);
    });

    // Set up webhook if supported
    if (provider.hasWebhookSupport()) {
      console.log('\n🔔 Setting up webhook for calendar changes...');
      
      const watchChannel = await provider.watchCalendar(
        primaryCalendar.id,
        'my-secret-token', // Use a secure random token in production
        24 // 24 hour TTL
      );

      console.log('✅ Webhook created successfully!');
      console.log(`   Channel ID: ${watchChannel.id}`);
      console.log(`   Resource ID: ${watchChannel.resourceId}`);
      console.log(`   Expiration: ${watchChannel.expiration}`);
      
      if (GoogleCalendarProvider.needsRenewal(watchChannel)) {
        console.log('   ⚠️  Channel expires soon and needs renewal!');
      }

      // Example of how to handle webhook in your server
      console.log('\n📮 Example webhook handler for your Express server:');
      console.log(`
app.post('/webhooks/google', async (req, res) => {
  try {
    const notification = GoogleCalendarProvider.parseWebhookHeaders(req.headers);
    const events = await provider.processNotification(notification, 'my-secret-token');
    console.log(\`Received \${events.length} updated events\`);
    res.status(200).send('OK');
  } catch (error) {
    console.error('Webhook error:', error);
    res.status(400).send('Bad Request');
  }
});
      `);

      // Clean up (optional)
      // await provider.stopWatch(watchChannel.id, watchChannel.resourceId);
    }
  }
}

main().catch(console.error);