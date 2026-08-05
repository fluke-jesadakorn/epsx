const NOTIFICATION_ITEM_MUTATION =
  /^\/api\/v1\/notification\/([A-Za-z0-9_-]+)(?:\/(?:read|unread|acknowledge|dismiss|click))?$/;

const RESERVED_NOTIFICATION_SEGMENTS = new Set([
  'admin',
  'clear-all',
  'mark-all-read',
  'send',
  'unread-count',
]);

export function isNotificationItemMutationPath(pathname: string): boolean {
  const match = NOTIFICATION_ITEM_MUTATION.exec(pathname);
  return match !== null && !RESERVED_NOTIFICATION_SEGMENTS.has(match[1]);
}
