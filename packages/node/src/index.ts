/**
 * @calblend/calendar - Unified calendar integration library
 */

// Import the native binding
const binding = require('../index.node');

// Re-export all types from the generated index.d.ts
export * from '../index';

// Re-export the binding functions with better names
export const {
  CalendarClient,
  CalendarSource,
  ParticipantStatus,
  ReminderMethod,
  EventStatus,
  EventVisibility,
  ShowAs,
} = binding;

// Export TypeScript-friendly interfaces
export interface TokenStorage {
  getToken(provider: CalendarSource): Promise<TokenData | null>;
  saveToken(provider: CalendarSource, token: TokenData): Promise<void>;
  removeToken(provider: CalendarSource): Promise<void>;
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
  providers?: CalendarSource[];
  userAgent?: string;
  timeout?: number;
}

/**
 * Create a new CalendarClient instance
 */
export function createClient(config: CalblendConfig): CalendarClient {
  // Create a wrapper that converts the TypeScript TokenStorage to what N-API expects
  const jsTokenStorage = {
    getToken: async (provider: CalendarSource) => {
      const token = await config.tokenStorage.getToken(provider);
      return token ? JSON.stringify(token) : null;
    },
    saveToken: async (provider: CalendarSource, tokenJson: string) => {
      const token = JSON.parse(tokenJson) as TokenData;
      await config.tokenStorage.saveToken(provider, token);
    },
    removeToken: async (provider: CalendarSource) => {
      await config.tokenStorage.removeToken(provider);
    },
  };

  return new CalendarClient(jsTokenStorage);
}

// Convenience exports
export type {
  UnifiedCalendarEvent,
  EventMoment,
  Participant,
  Reminder,
  ConferenceLink,
  Calendar,
} from '../index';

// Export providers
export * from './providers';