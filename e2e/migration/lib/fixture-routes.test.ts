import { describe, expect, test } from 'bun:test';

import { isNotificationItemMutationPath } from './fixture-routes';

describe('isNotificationItemMutationPath', () => {
  test('accepts item mutations and their supported lifecycle actions', () => {
    for (const pathname of [
      '/api/v1/notification/idem_notification_e2e_1',
      '/api/v1/notification/idem_notification_e2e_1/read',
      '/api/v1/notification/idem_notification_e2e_1/unread',
      '/api/v1/notification/idem_notification_e2e_1/acknowledge',
      '/api/v1/notification/idem_notification_e2e_1/dismiss',
      '/api/v1/notification/idem_notification_e2e_1/click',
    ]) {
      expect(isNotificationItemMutationPath(pathname)).toBe(true);
    }
  });

  test('never shadows reserved notification endpoints', () => {
    for (const pathname of [
      '/api/v1/notification/admin',
      '/api/v1/notification/clear-all',
      '/api/v1/notification/mark-all-read',
      '/api/v1/notification/send',
      '/api/v1/notification/unread-count',
    ]) {
      expect(isNotificationItemMutationPath(pathname)).toBe(false);
    }
  });

  test('rejects unsupported actions and ambiguous path shapes', () => {
    for (const pathname of [
      '/api/v1/notification/',
      '/api/v1/notification/id/',
      '/api/v1/notification/id/send',
      '/api/v1/notification/id/read/again',
      '/api/v1/notifications/id/read',
    ]) {
      expect(isNotificationItemMutationPath(pathname)).toBe(false);
    }
  });
});
