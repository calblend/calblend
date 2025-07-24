/**
 * Integration tests for Calblend FFI
 */

describe('Calblend Integration Tests', () => {
  // Skip these tests if native module is not built
  const skipIfNoBinary = process.env.SKIP_NATIVE_TESTS === 'true' ? describe.skip : describe;

  skipIfNoBinary('Native Module Loading', () => {
    it('should load the native module', () => {
      let binding;
      try {
        binding = require('../../index.js');
      } catch (error: any) {
        // If the native module isn't built yet, skip the test
        if (error.message.includes('Failed to load native binding')) {
          console.log('Native module not built. Run "npm run build:napi" first.');
          return;
        }
        throw error;
      }

      expect(binding).toBeDefined();
      expect(binding.CalendarClient).toBeDefined();
      expect(binding.GoogleCalendarProvider).toBeDefined();
    });
  });

  describe('Type Exports', () => {
    it('should export all required types', async () => {
      const calblend = await import('../index');
      
      // Check type exports (these are compile-time checks)
      expect(calblend.CalendarSource).toBeDefined();
      expect(calblend.ParticipantStatus).toBeDefined();
      expect(calblend.ReminderMethod).toBeDefined();
      expect(calblend.EventStatus).toBeDefined();
      expect(calblend.EventVisibility).toBeDefined();
      expect(calblend.ShowAs).toBeDefined();
      expect(calblend.BusyStatus).toBeDefined();
    });

    it('should have correct enum values', async () => {
      const { CalendarSource, EventStatus, ReminderMethod } = await import('../index');
      
      expect(CalendarSource.Google).toBe('Google');
      expect(EventStatus.Confirmed).toBe('Confirmed');
      expect(ReminderMethod.Email).toBe('Email');
    });
  });

  describe('Google Provider exports', () => {
    it('should export GoogleCalendarProvider wrapper', async () => {
      const { GoogleCalendarProvider } = await import('../providers/google');
      
      expect(GoogleCalendarProvider).toBeDefined();
      expect(GoogleCalendarProvider.parseWebhookHeaders).toBeDefined();
    });
  });
});