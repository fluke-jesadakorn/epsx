# OIDC Authentication Flow Testing Guide

## Overview
This document provides a checklist to test the complete OpenID Connect implementation across both frontend applications.

## Prerequisites

### Environment Variables
Ensure the following environment variables are set:

**Frontend (.env.local)**:
```env
NEXTAUTH_SECRET=your-nextauth-secret-at-least-32-chars
NEXTAUTH_URL=http://localhost:3000
NEXT_PUBLIC_BACKEND_URL=http://localhost:8080
NEXT_PUBLIC_APP_URL=http://localhost:3000
OIDC_CLIENT_ID=epsx-frontend
```

**Admin Frontend (.env.local)**:
```env
NEXTAUTH_SECRET=your-nextauth-secret-at-least-32-chars
NEXTAUTH_URL=http://localhost:3001
NEXT_PUBLIC_BACKEND_URL=http://localhost:8080
NEXT_PUBLIC_ADMIN_URL=http://localhost:3001
OIDC_CLIENT_ID=epsx-admin
NEXT_PUBLIC_OAUTH_CLIENT_ID=epsx-admin
```

**Backend**:
```env
BACKEND_URL=http://localhost:8080
JWT_SECRET=your-jwt-secret-at-least-32-chars
```

## Test Cases

### 1. Frontend Authentication Flow

#### Test Case 1.1: Basic Login Flow
1. **Start Applications**:
   ```bash
   pnpm dev:frontend    # Port 3000
   pnpm dev:backend     # Port 8080
   ```

2. **Navigate to Login**:
   - Go to `http://localhost:3000/login`
   - Should redirect to backend OAuth authorization page
   - Check network tab: should see redirect to `http://localhost:8080/oauth/authorize`

3. **Complete Backend Login**:
   - Fill in credentials on backend login form
   - Submit form
   - Should redirect back to `http://localhost:3000/api/auth/callback/epsx-backend`
   - Check browser cookies: should have `epsx_frontend_jwt`

4. **Verify Authentication**:
   - Should redirect to `/dashboard` 
   - Check session: GET `http://localhost:3000/api/auth/session`
   - Should return user data with `isAuthenticated: true`

#### Test Case 1.2: PKCE Flow
1. **Enable PKCE Flow**:
   - Open browser dev tools
   - Navigate to `http://localhost:3000/login`
   - Check network requests for `/api/auth/initiate`

2. **Verify PKCE Cookies**:
   - Check browser cookies after initiate call
   - Should have `oauth_code_verifier`, `oauth_state`, `oauth_callback_url`

3. **Complete Flow**:
   - Complete backend login
   - Verify callback cleans up PKCE cookies
   - Only `epsx_frontend_jwt` should remain

#### Test Case 1.3: Registration Flow
1. **Navigate to Register**:
   - Go to `http://localhost:3000/register`
   - Should redirect to backend registration page

2. **Complete Registration**:
   - Fill in registration form on backend
   - Submit form
   - Should create account and redirect back to frontend

#### Test Case 1.4: Password Reset Flow
1. **Initiate Reset**:
   - Go to `http://localhost:3000/forgot-password`
   - Should redirect to backend password reset form

2. **Complete Reset**:
   - Enter email on backend form
   - Check that reset process redirects appropriately

3. **Reset with Token**:
   - Go to `http://localhost:3000/reset-password?token=test`
   - Should redirect to backend confirmation page

### 2. Admin Frontend Authentication Flow

#### Test Case 2.1: Admin Login Flow
1. **Start Admin Application**:
   ```bash
   pnpm dev:admin    # Port 3001
   pnpm dev:backend  # Port 8080
   ```

2. **Navigate to Admin Login**:
   - Go to `http://localhost:3001/login`
   - Should redirect to backend with admin client parameters
   - Check URL contains `client_id=epsx-admin` and `scope=openid profile email admin`

3. **Complete Admin Login**:
   - Login with admin credentials on backend
   - Should redirect back to admin frontend
   - Check cookies: should have `epsx_admin_jwt`

4. **Verify Admin Session**:
   - Check session: GET `http://localhost:3001/api/auth/session`
   - Should return admin user data with admin_modules

#### Test Case 2.2: Admin Permissions
1. **Check Admin Access**:
   - Navigate through admin dashboard
   - Verify access to admin-only features
   - Check that regular users can't access admin

### 3. Session Management

#### Test Case 3.1: Session Refresh
1. **Test Session Validation**:
   - Login to either application
   - Call PUT `/api/auth/session` to test refresh
   - Should validate session with backend

2. **Test Session Expiry**:
   - Wait for token to approach expiry (or manually adjust token)
   - Check that refresh mechanism works

#### Test Case 3.2: Logout Flow
1. **Frontend Logout**:
   - Click logout in frontend app
   - Should clear JWT cookie
   - Should redirect to home page

2. **Admin Logout**:
   - Click logout in admin app
   - Should clear admin JWT cookie
   - Should redirect to admin login

### 4. Error Handling

#### Test Case 4.1: Invalid Credentials
1. **Test Bad Login**:
   - Try to login with invalid credentials
   - Should show error message on backend form
   - Should not create frontend session

#### Test Case 4.2: Missing PKCE Parameters
1. **Clear Cookies Mid-Flow**:
   - Start login flow
   - Clear cookies before completing
   - Should gracefully handle missing PKCE

#### Test Case 4.3: Backend Unavailable
1. **Stop Backend**:
   - Stop backend server
   - Try to login
   - Should show appropriate error messages

### 5. Security Testing

#### Test Case 5.1: JWT Validation
1. **Invalid JWT**:
   - Manually modify JWT cookie
   - Should reject invalid tokens

2. **Expired JWT**:
   - Use expired JWT token
   - Should require re-authentication

#### Test Case 5.2: CSRF Protection
1. **State Parameter**:
   - Verify state parameter is validated
   - Test with invalid state

### 6. Cross-Application Testing

#### Test Case 6.1: Separate Sessions
1. **Login Both Apps**:
   - Login to frontend with user account
   - Login to admin with admin account
   - Verify sessions are independent

2. **Cookie Isolation**:
   - Check that frontend and admin cookies don't interfere

## Expected Results

### Successful Flow Indicators:
- ✅ Redirects work smoothly without errors
- ✅ Backend shows PancakeSwap-themed auth pages
- ✅ JWT cookies are set correctly
- ✅ Session APIs return valid user data
- ✅ PKCE cookies are cleaned up after auth
- ✅ Admin users get admin_modules in session
- ✅ Error states are handled gracefully

### Common Issues:
- ❌ 404 on callback routes - check route files exist
- ❌ CORS errors - check backend CORS configuration
- ❌ Cookie not set - check sameSite and secure settings
- ❌ Infinite redirects - check environment URLs
- ❌ PKCE errors - check cookie settings and timeouts

## Debugging Tools

### Browser DevTools:
- **Network Tab**: Monitor redirects and API calls
- **Application Tab**: Check cookies and local storage
- **Console**: Look for error messages and logs

### Server Logs:
- Check backend logs for OIDC endpoint calls
- Check frontend/admin server logs for errors
- Look for JWT validation messages

### Useful Commands:
```bash
# Check if applications are running
curl http://localhost:3000/api/auth/session
curl http://localhost:3001/api/auth/session
curl http://localhost:8080/oauth/.well-known/openid-configuration

# Test backend endpoints
curl http://localhost:8080/oauth/userinfo -H "Authorization: Bearer test"
```

## Manual Testing Checklist

- [ ] Frontend login redirects to backend
- [ ] Backend shows PancakeSwap login form
- [ ] Login completes and redirects back to frontend
- [ ] JWT cookie is set in browser
- [ ] Session API returns user data
- [ ] Admin login uses admin client ID
- [ ] Admin session includes admin_modules
- [ ] PKCE cookies are generated and cleaned up
- [ ] Registration flow works end-to-end
- [ ] Password reset flows work end-to-end
- [ ] Logout clears cookies and redirects
- [ ] Error states show appropriate messages
- [ ] Sessions can be refreshed
- [ ] Invalid tokens are rejected

## Automated Testing

Run the following commands to execute automated tests:

```bash
# Frontend tests
cd apps/frontend
pnpm test

# Admin frontend tests  
cd apps/admin-frontend
pnpm test

# E2E tests
pnpm test:e2e
```