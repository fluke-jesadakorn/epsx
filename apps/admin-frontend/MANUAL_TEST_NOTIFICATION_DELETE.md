# Manual Test: Notification Delete with Toast

## Setup
1. Start backend: `cd apps/backend && cargo run`
2. Start admin frontend: `cd apps/admin-frontend && pnpm dev`

## Test Steps

### 1. Create a Notification
1. Navigate to http://localhost:3001
2. Sign in with admin wallet
3. Go to Notifications page
4. Click "Send" tab
5. Fill in:
   - Title: "Test Delete Notification"
   - Message: "This notification will be deleted to test toast"
   - Type: System
   - Priority: Normal
   - Check "Broadcast" checkbox
6. Click "Send Notification"
7. Verify success toast appears

### 2. View Notification in Bell
1. Click the notification bell icon 🔔 in header
2. Verify the notification "Test Delete Notification" appears in the dropdown
3. Take screenshot: `screenshot-01-notification-in-bell.png`

### 3. Delete Notification
1. Hover over the notification item
2. Verify delete button (✕) appears on the right
3. Take screenshot: `screenshot-02-delete-button-visible.png`
4. Click the delete button (✕)

### 4. Verify Toast Message
1. Verify toast notification appears in top-right with:
   - Text: "Notification deleted"
   - Green gradient background (success style)
   - White text
2. Take screenshot: `screenshot-03-toast-visible.png`
3. Wait for toast to auto-dismiss or manually dismiss

### 5. Verify Notification Removed
1. Verify the notification is no longer in the bell dropdown
2. Verify the notification count badge decreased by 1
3. Take screenshot: `screenshot-04-notification-removed.png`

## Expected Results
✅ Toast appears with "Notification deleted" message
✅ Toast has success styling (green gradient background)
✅ Notification is removed from the bell dropdown
✅ Notification count is updated correctly
✅ No errors in browser console

## Screenshots Location
Save all screenshots to: `/Users/fluke/Desktop/Work/epsx/test-screenshots/`

## Code Verification

### Toast Provider
File: `apps/admin-frontend/components/providers/ToastProvider.tsx`
- ✅ Toaster component with proper configuration
- ✅ Success styling with green gradient
- ✅ zIndex: 99999 to appear above all content

### Delete Handler
File: `apps/admin-frontend/components/layout/AdminNotificationBellClient.tsx`
```typescript
const handleDeleteNotification = async (e: React.MouseEvent, notificationId: string) => {
  e.stopPropagation()
  try {
    const client = createNotificationsClient(createAdminApiClient())
    await client.deleteAdminNotification(notificationId)

    // Remove from local state
    setNotifications(prev => prev.filter(n => n.id !== notificationId))
    setCount(prev => Math.max(0, prev - 1))

    toast.success('Notification deleted')  // ✅ Toast is called here
  } catch (error) {
    console.error('Failed to delete notification:', error)
    toast.error('Failed to delete notification')
  }
}
```

### Delete Button UI
```typescript
<button
  onClick={(e) => handleDeleteNotification(e, notification.id)}
  className="opacity-0 group-hover:opacity-100 text-gray-400 hover:text-red-500"
  title="Delete notification"
>
  ✕
</button>
```

## Common Issues

### Toast Not Showing
- Check browser console for errors
- Verify `ToastProvider` is mounted in layout.tsx
- Check if toast is behind other elements (z-index)
- Verify `react-hot-toast` is properly imported

### Delete Button Not Appearing
- Hover properly over the notification card
- Check if `group` and `group-hover` classes are working
- Inspect element to verify button exists in DOM

### API Errors
- Check backend is running on port 8080
- Check network tab for delete API call status
- Verify admin authentication token is valid

## Success Criteria
All checkboxes above must be ✅ for the feature to be considered working correctly.
