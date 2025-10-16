# Notification System - Future Enhancements

This document tracks planned enhancements for the notification system.

## 1. Group Notifications

**Status:** Planned
**Priority:** Medium
**Location:** `apps/backend/src/web/admin/notification_handlers.rs:173`

**Description:**
Currently, the `recipient_group` parameter in SendNotificationRequest accepts a group name but uses it as a placeholder. Future implementation should:

1. Fetch all wallet addresses belonging to the specified permission group
2. Send notifications to all members of that group
3. Support dynamic group membership (users joining/leaving groups)

**Implementation Requirements:**
- Integration with permission group system
- Query to fetch all wallet addresses for a given group
- Batch notification creation for all group members
- Consider caching group membership for performance

**Related Files:**
- `apps/backend/src/web/admin/notification_handlers.rs` - send_notification_handler function
- Permission group repository adapters

---

## 2. Full Notifications Page

**Status:** Planned
**Priority:** Low
**Location:** `apps/admin-frontend/components/layout/AdminNotificationBellClient.tsx:184`

**Description:**
The "View All Notifications" button in the notification dropdown currently has no destination page. Future implementation should:

1. Create a dedicated `/notifications` page in both frontend and admin-frontend
2. Implement full notification list with filtering and search
3. Add pagination or virtual scrolling for large notification lists
4. Include notification management features (bulk delete, mark all as read, etc.)

**Implementation Requirements:**
- Create new page components in both frontends
- Implement advanced filtering (date range, type, priority, status)
- Add search functionality for notification content
- Consider virtual scrolling for performance (50+ notifications)
- Add bulk actions (select multiple, delete, mark as read)

**Related Files:**
- `apps/frontend/app/notifications/page.tsx` (to be created)
- `apps/admin-frontend/app/notifications/page.tsx` (to be created)
- `apps/admin-frontend/components/layout/AdminNotificationBellClient.tsx`
- `apps/frontend/components/notifications/NotificationBellClient.tsx`

**Dependencies:**
- Current dropdown shows only 5 notifications (MAX_DROPDOWN_NOTIFICATIONS = 5)
- Full page would benefit from virtual scrolling implementation
- May require additional backend pagination endpoints

---

## 3. Additional Future Enhancements (Not Currently Tracked as TODOs)

### Notification Scheduling
- Schedule notifications to be sent at specific times
- Recurring notifications (daily, weekly, monthly)
- Time zone aware scheduling

### Rich Notifications
- Support for markdown formatting in notification messages
- Inline images and attachments
- Interactive buttons and actions within notifications

### Notification Templates
- Pre-defined templates for common notification types
- Template variables for personalization
- Template management interface in admin panel

### Advanced Analytics
- Notification engagement metrics
- A/B testing for notification content
- Optimal send time recommendations

### Multi-Channel Delivery
- Email notifications (currently not implemented)
- SMS notifications (currently not implemented)
- Push notifications (foundation exists but not fully implemented)
- Webhook notifications for external integrations

### Notification Preferences UI
- User-facing interface for managing notification preferences
- Granular control over notification types and priorities
- Quiet hours configuration
- Do Not Disturb mode

---

## Migration Path

When implementing these enhancements:

1. **Phase 1:** Group Notifications (depends on permission group system maturity)
2. **Phase 2:** Full Notifications Page (can be done independently)
3. **Phase 3:** Advanced features (scheduling, templates, analytics)
4. **Phase 4:** Multi-channel delivery (requires external service integrations)

## Performance Considerations

- Current system is optimized for real-time delivery via SSE
- Database indexes are in place for common queries (Phase 5.1 complete)
- Virtual scrolling should be implemented when full pages are created
- Consider Redis caching for group membership lookups
- Monitor notification volume and implement rate limiting if needed

## Security Considerations

- Group notifications must respect permission boundaries
- Ensure users can only view their own notifications (already implemented)
- Admin-only access to broadcast notifications (already implemented)
- Validate all notification content to prevent XSS attacks (already implemented)

---

Last Updated: 2025-10-14
