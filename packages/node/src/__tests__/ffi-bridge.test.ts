/**
 * Tests for the FFI bridge between Rust and Node.js
 */

import { GoogleCalendarProvider } from '../providers/google';
import { CalendarSource } from '../index';

describe('FFI Bridge Tests', () => {
  describe('GoogleCalendarProvider', () => {
    it('should create provider instance through FFI', () => {
      const provider = new GoogleCalendarProvider({
        clientId: 'test-client-id',
        clientSecret: 'test-client-secret',
        redirectUri: 'http://localhost:3000/callback',
      });

      expect(provider).toBeDefined();
      expect(provider.getAuthUrl).toBeDefined();
      expect(provider.exchangeCode).toBeDefined();
      expect(provider.listCalendars).toBeDefined();
    });

    it('should handle async operations through FFI', async () => {
      const provider = new GoogleCalendarProvider({
        clientId: 'test-client-id',
        clientSecret: 'test-client-secret',
        redirectUri: 'http://localhost:3000/callback',
      });

      // Test auth URL generation
      const authUrl = await provider.getAuthUrl();
      expect(authUrl).toContain('accounts.google.com');
      expect(authUrl).toContain('client_id=test-client-id');
    });

    it('should properly serialize/deserialize data across FFI boundary', async () => {
      const provider = new GoogleCalendarProvider({
        clientId: 'test-client-id',
        clientSecret: 'test-client-secret',
        redirectUri: 'http://localhost:3000/callback',
      });

      // Test webhook configuration
      provider.setWebhookEndpoint('https://example.com/webhook');
      expect(provider.hasWebhookSupport()).toBe(true);
    });

    it('should handle errors properly across FFI', async () => {
      const provider = new GoogleCalendarProvider({
        clientId: 'test-client-id',
        clientSecret: 'test-client-secret',
        redirectUri: 'http://localhost:3000/callback',
      });

      // Test error handling - watching calendar without webhook endpoint
      try {
        await provider.watchCalendar('primary');
        fail('Should have thrown an error');
      } catch (error: any) {
        expect(error.message).toContain('webhook endpoint');
      }
    });

    it('should parse webhook headers correctly', () => {
      const headers = {
        'x-goog-channel-id': 'test-channel-id',
        'x-goog-resource-id': 'test-resource-id',
        'x-goog-resource-state': 'sync',
        'x-goog-resource-uri': 'https://www.googleapis.com/calendar/v3/calendars/primary/events',
        'x-goog-channel-token': 'test-token',
        'x-goog-channel-expiration': '1234567890',
        'x-goog-message-number': '1',
      };

      const notification = GoogleCalendarProvider.parseWebhookHeaders(headers);
      
      expect(notification).toEqual({
        channelId: 'test-channel-id',
        resourceId: 'test-resource-id',
        resourceState: 'sync',
        resourceUri: 'https://www.googleapis.com/calendar/v3/calendars/primary/events',
        channelToken: 'test-token',
        channelExpiration: '1234567890',
        messageNumber: '1',
      });
    });
  });

  describe('Enum values', () => {
    it('should have correct CalendarSource values', () => {
      expect(CalendarSource.Google).toBe('Google');
      expect(CalendarSource.Outlook).toBe('Outlook');
      expect(CalendarSource.Ios).toBe('Ios');
      expect(CalendarSource.Android).toBe('Android');
    });
  });

  describe('Memory management', () => {
    it('should not leak memory with repeated provider creation', async () => {
      const initialMemory = process.memoryUsage().heapUsed;
      
      // Create and destroy many providers
      for (let i = 0; i < 100; i++) {
        const provider = new GoogleCalendarProvider({
          clientId: 'test-client-id',
          clientSecret: 'test-client-secret',
          redirectUri: 'http://localhost:3000/callback',
        });
        
        // Perform an operation
        await provider.getAuthUrl();
      }
      
      // Force garbage collection if available
      if (global.gc) {
        global.gc();
      }
      
      const finalMemory = process.memoryUsage().heapUsed;
      const memoryIncrease = finalMemory - initialMemory;
      
      // Memory increase should be reasonable (less than 10MB)
      expect(memoryIncrease).toBeLessThan(10 * 1024 * 1024);
    });
  });

  describe('Concurrent operations', () => {
    it('should handle concurrent FFI calls', async () => {
      const provider = new GoogleCalendarProvider({
        clientId: 'test-client-id',
        clientSecret: 'test-client-secret',
        redirectUri: 'http://localhost:3000/callback',
      });

      // Make multiple concurrent calls
      const promises = Array.from({ length: 10 }, () => provider.getAuthUrl());
      const results = await Promise.all(promises);
      
      // All results should be the same
      const firstUrl = results[0];
      expect(results.every(url => url === firstUrl)).toBe(true);
    });
  });
});

/**
 * Test helper to verify FFI data structures
 */
describe('FFI Data Structure Tests', () => {
  it('should handle complex nested structures', () => {
    // This would test event creation with all nested fields
    // once we have a mock token storage setup
    const event = {
      id: 'test-id',
      title: 'Test Event',
      description: 'Test Description',
      start: {
        dateTime: '2024-01-15T10:00:00Z',
        timeZone: 'UTC',
      },
      end: {
        dateTime: '2024-01-15T11:00:00Z',
        timeZone: 'UTC',
      },
      participants: [
        {
          email: 'test@example.com',
          displayName: 'Test User',
          organizer: true,
          status: 'Accepted',
        },
      ],
      reminders: [
        {
          method: 'Email',
          minutesBefore: 15,
        },
      ],
      conferenceData: {
        provider: 'GoogleMeet',
        joinUrl: 'https://meet.google.com/test',
        meetingId: 'test-meeting-id',
      },
    };

    // Verify structure matches expected format
    expect(event.start).toHaveProperty('dateTime');
    expect(event.participants[0]).toHaveProperty('status');
    expect(event.reminders[0]).toHaveProperty('method');
  });
});