/**
 * Google Calendar provider for TypeScript
 */

// Import native bindings
const { GoogleCalendarProvider: NativeGoogleProvider, GoogleWatchChannel } = require('../../index.node');

import type { 
  Calendar,
  UnifiedCalendarEvent,
  CalendarSource,
  TokenStorage
} from '../index';

export interface GoogleProviderConfig {
  clientId: string;
  clientSecret: string;
  redirectUri: string;
  tokenStorage: TokenStorage;
  webhookEndpoint?: string;
}

export interface WatchChannel {
  id: string;
  resourceId: string;
  resourceUri: string;
  token?: string;
  expiration: string;
}

export interface WebhookNotification {
  channelId: string;
  channelToken?: string;
  channelExpiration?: string;
  resourceId: string;
  resourceState: string;
  resourceUri: string;
  messageNumber?: string;
}

/**
 * Google Calendar provider
 */
export class GoogleCalendarProvider {
  private native: typeof NativeGoogleProvider;

  constructor(config: GoogleProviderConfig) {
    // Create a simple token storage wrapper for the native code
    const nativeTokenStorage = {
      getToken: async () => {
        const token = await config.tokenStorage.getToken('Google' as CalendarSource);
        return token;
      },
      saveToken: async (token: any) => {
        await config.tokenStorage.saveToken('Google' as CalendarSource, token);
      },
      removeToken: async () => {
        await config.tokenStorage.removeToken('Google' as CalendarSource);
      }
    };

    this.native = new NativeGoogleProvider(
      config.clientId,
      config.clientSecret,
      config.redirectUri,
      nativeTokenStorage,
      config.webhookEndpoint
    );
  }

  /**
   * Get the OAuth2 authorization URL
   */
  async getAuthUrl(): Promise<string> {
    return this.native.getAuthUrl();
  }

  /**
   * Exchange authorization code for tokens
   */
  async exchangeCode(code: string): Promise<void> {
    return this.native.exchangeCode(code);
  }

  /**
   * List all calendars
   */
  async listCalendars(): Promise<Calendar[]> {
    return this.native.listCalendars();
  }

  /**
   * List events in a calendar
   */
  async listEvents(
    calendarId: string,
    startDate?: Date,
    endDate?: Date
  ): Promise<UnifiedCalendarEvent[]> {
    const start = startDate?.toISOString();
    const end = endDate?.toISOString();
    return this.native.listEvents(calendarId, start, end);
  }

  /**
   * Create a new event
   */
  async createEvent(
    calendarId: string,
    event: UnifiedCalendarEvent
  ): Promise<UnifiedCalendarEvent> {
    return this.native.createEvent(calendarId, event);
  }

  /**
   * Update an existing event
   */
  async updateEvent(
    calendarId: string,
    eventId: string,
    event: UnifiedCalendarEvent
  ): Promise<UnifiedCalendarEvent> {
    return this.native.updateEvent(calendarId, eventId, event);
  }

  /**
   * Delete an event
   */
  async deleteEvent(calendarId: string, eventId: string): Promise<void> {
    return this.native.deleteEvent(calendarId, eventId);
  }

  /**
   * Check if webhook support is enabled
   */
  hasWebhookSupport(): boolean {
    return this.native.hasWebhookSupport();
  }

  /**
   * Watch a calendar for changes
   */
  async watchCalendar(
    calendarId: string,
    token?: string,
    ttlHours?: number
  ): Promise<WatchChannel> {
    return this.native.watchCalendar(calendarId, token, ttlHours);
  }

  /**
   * Stop watching a calendar
   */
  async stopWatch(channelId: string, resourceId: string): Promise<void> {
    return this.native.stopWatch(channelId, resourceId);
  }

  /**
   * Process a webhook notification
   */
  async processNotification(
    notification: WebhookNotification,
    expectedToken?: string
  ): Promise<UnifiedCalendarEvent[]> {
    return this.native.processNotification(
      notification.channelId,
      notification.channelToken,
      notification.channelExpiration,
      notification.resourceId,
      notification.resourceState,
      notification.resourceUri,
      notification.messageNumber,
      expectedToken
    );
  }

  /**
   * Parse webhook headers from HTTP request
   * @param headers - HTTP headers object (e.g., from Express req.headers)
   */
  static parseWebhookHeaders(headers: Record<string, string | string[] | undefined>): WebhookNotification {
    const getHeader = (name: string): string | undefined => {
      const value = headers[name.toLowerCase()];
      return Array.isArray(value) ? value[0] : value;
    };

    const channelId = getHeader('x-goog-channel-id');
    const resourceId = getHeader('x-goog-resource-id');
    const resourceState = getHeader('x-goog-resource-state');
    const resourceUri = getHeader('x-goog-resource-uri');

    if (!channelId || !resourceId || !resourceState || !resourceUri) {
      throw new Error('Missing required Google webhook headers');
    }

    return {
      channelId,
      channelToken: getHeader('x-goog-channel-token'),
      channelExpiration: getHeader('x-goog-channel-expiration'),
      resourceId,
      resourceState,
      resourceUri,
      messageNumber: getHeader('x-goog-message-number'),
    };
  }

  /**
   * Check if a watch channel needs renewal
   */
  static needsRenewal(channel: WatchChannel): boolean {
    const expiration = new Date(channel.expiration);
    const hoursUntilExpiry = (expiration.getTime() - Date.now()) / (1000 * 60 * 60);
    return hoursUntilExpiry < 24;
  }
}