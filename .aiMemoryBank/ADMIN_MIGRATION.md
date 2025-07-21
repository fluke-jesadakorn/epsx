# Admin Feature Migration Guide

This document describes the migration of admin features from the frontend application to the dedicated admin-frontend application.

## Overview

All administrative features have been moved from `/apps/frontend` to `/apps/admin-frontend` to provide better separation of concerns and enhanced security for admin operations.

## What Was Moved

### Pages
- `/admin` → Admin dashboard (now in admin-frontend)
- `/admin/users` → User management (now in admin-frontend)
- All future admin routes will be in admin-frontend

### API Routes
- `/api/admin/users` → Now proxied to admin-frontend
- Future admin APIs will be implemented in admin-frontend

### Server Actions
- `app/actions/admin-server.ts` → Moved to admin-frontend

## Migration Implementation

### 1. Seamless Redirects
- All `/admin*` routes in frontend now redirect to admin-frontend
- Middleware automatically forwards users to the correct admin application
- Authentication cookies are preserved during redirection

### 2. API Proxying
- Admin API calls from frontend are automatically proxied to admin-frontend
- Maintains backward compatibility for existing integrations

### 3. Environment Configuration
```env
# Frontend .env.development
NEXT_PUBLIC_ADMIN_FRONTEND_URL=http://localhost:3001
```

### 4. Updated Navigation
- Dashboard links now open admin-frontend in new tab
- Maintains security isolation between applications

## File Changes

### Frontend (`/apps/frontend`)
```
app/admin/page.tsx → Redirect component
app/admin/users/page.tsx → Redirect component
app/api/admin/users/route.ts → Proxy to admin-frontend
middleware/admin-redirect.ts → NEW: Admin redirect middleware
middleware.ts → Updated to include admin redirect logic
config/iam.ts → Removed admin route protection (handled by admin-frontend)
app/dashboard/page.tsx → Updated links to point to admin-frontend
.env.development → Added NEXT_PUBLIC_ADMIN_FRONTEND_URL
```

### Admin Frontend (`/apps/admin-frontend`)
```
app/actions/admin-server.ts → Copied from frontend
```

### Backup Files (Preserved for reference)
```
app/admin/page.tsx.backup → Original admin dashboard
app/admin/users/page.tsx.backup → Original user management
app/api/admin/users/route.ts.backup → Original admin API
app/actions/admin-server.ts.backup → Original server actions
```

## Development Setup

1. **Start Frontend** (Port 3000):
   ```bash
   cd apps/frontend
   pnpm dev
   ```

2. **Start Admin Frontend** (Port 3001):
   ```bash
   cd apps/admin-frontend
   pnpm dev
   ```

3. **Access Admin Features**:
   - Navigate to `http://localhost:3000/admin` 
   - Automatically redirected to `http://localhost:3001/admin`
   - Or directly access `http://localhost:3001`

## Production Deployment

Ensure both applications are deployed and update the environment variable:
```env
NEXT_PUBLIC_ADMIN_FRONTEND_URL=https://admin.yourdomain.com
```

## Benefits

1. **Security Isolation**: Admin features are completely separated
2. **Scalability**: Admin and user applications can scale independently  
3. **Development Efficiency**: Admin features can be developed without affecting main app
4. **Backward Compatibility**: Existing bookmarks and links continue to work
5. **Progressive Migration**: Users experience seamless transition

## Testing

1. **Manual Testing**:
   - Visit `/admin` on frontend → Should redirect to admin-frontend
   - Test all admin navigation links
   - Verify authentication state is preserved

2. **API Testing**:
   - Frontend API calls to `/api/admin/*` should work seamlessly
   - Admin-frontend APIs should function independently

## Rollback Plan

If issues arise, restore backup files:
```bash
cd apps/frontend
mv app/admin/page.tsx.backup app/admin/page.tsx
mv app/admin/users/page.tsx.backup app/admin/users/page.tsx
mv app/api/admin/users/route.ts.backup app/api/admin/users/route.ts
mv app/actions/admin-server.ts.backup app/actions/admin-server.ts
```

And revert middleware and configuration changes.

## Future Considerations

1. **Authentication Sync**: Consider implementing SSO between applications
2. **Shared Components**: Move common UI components to shared packages
3. **API Gateway**: Consider implementing an API gateway for better request routing
4. **Monitoring**: Set up separate monitoring for admin operations
