/**
 * @calblend/calendar - Unified calendar integration library
 */

// Import the native binding
const binding = require('../index.js');

// Re-export the native binding classes and enums
export const {
  CalendarClient,
  GoogleCalendarProvider: NativeGoogleCalendarProvider,
} = binding;

// Import types from the generated type definitions
import type {
  UnifiedCalendarEvent,
  EventMoment,
  Participant,
  Reminder,
  ConferenceLink,
  Calendar,
  CalendarSource as CalendarSourceType,
  ParticipantStatus as ParticipantStatusType,
  ReminderMethod as ReminderMethodType,
  EventStatus as EventStatusType,
  EventVisibility as EventVisibilityType,
  ShowAs as ShowAsType,
  FreeBusyPeriod,
  BusyStatus as BusyStatusType,
  WatchChannel,
  WebhookNotification,
} from '../index.d.ts';

// Re-export types
export type {
  UnifiedCalendarEvent,
  EventMoment,
  Participant,
  Reminder,
  ConferenceLink,
  Calendar,
  CalendarSourceType as CalendarSource,
  ParticipantStatusType as ParticipantStatus,
  ReminderMethodType as ReminderMethod,
  EventStatusType as EventStatus,
  EventVisibilityType as EventVisibility,
  ShowAsType as ShowAs,
  FreeBusyPeriod,
  BusyStatusType as BusyStatus,
  WatchChannel,
  WebhookNotification,
};

// Export TypeScript-friendly interfaces
export interface TokenStorage {
  getToken(provider: CalendarSourceType): Promise<TokenData | null>;
  saveToken(provider: CalendarSourceType, token: TokenData): Promise<void>;
  removeToken(provider: CalendarSourceType): Promise<void>;
}

export interface TokenData {
  access_token: string;
  refresh_token?: string;
  expires_at?: string; // ISO 8601 date string
  token_type: string;
  scope?: string;
}

export interface CalblendConfig {
  tokenStorage: TokenStorage;
  providers?: CalendarSourceType[];
  userAgent?: string;
  timeout?: number;
}

/**
 * Create a new CalendarClient instance
 */
export function createClient(config: CalblendConfig): typeof CalendarClient {
  // Create a wrapper that converts the TypeScript TokenStorage to what N-API expects
  const jsTokenStorage = {
    getToken: async (provider: CalendarSourceType) => {
      const token = await config.tokenStorage.getToken(provider);
      return token ? JSON.stringify(token) : null;
    },
    saveToken: async (provider: CalendarSourceType, tokenJson: string) => {
      const token = JSON.parse(tokenJson) as TokenData;
      await config.tokenStorage.saveToken(provider, token);
    },
    removeToken: async (provider: CalendarSourceType) => {
      await config.tokenStorage.removeToken(provider);
    },
  };

  return new CalendarClient(jsTokenStorage);
}

// Export providers
export * from './providers';

// Export enums as const objects for runtime use
export const CalendarSource = {
  Google: 'Google' as const,
  Outlook: 'Outlook' as const,
  Ios: 'Ios' as const,
  Android: 'Android' as const,
} as const;

export const ParticipantStatus = {
  NeedsAction: 'NeedsAction' as const,
  Accepted: 'Accepted' as const,
  Declined: 'Declined' as const,
  Tentative: 'Tentative' as const,
} as const;

export const ReminderMethod = {
  Email: 'Email' as const,
  Popup: 'Popup' as const,
  Sms: 'Sms' as const,
  Push: 'Push' as const,
} as const;

export const EventStatus = {
  Confirmed: 'Confirmed' as const,
  Tentative: 'Tentative' as const,
  Cancelled: 'Cancelled' as const,
} as const;

export const EventVisibility = {
  Default: 'Default' as const,
  Public: 'Public' as const,
  Private: 'Private' as const,
  Confidential: 'Confidential' as const,
} as const;

export const ShowAs = {
  Free: 'Free' as const,
  Busy: 'Busy' as const,
  Tentative: 'Tentative' as const,
  OutOfOffice: 'OutOfOffice' as const,
} as const;

export const BusyStatus = {
  Free: 'Free' as const,
  Busy: 'Busy' as const,
  Tentative: 'Tentative' as const,
  OutOfOffice: 'OutOfOffice' as const,
} as const;