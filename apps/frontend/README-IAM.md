# IAM Implementation Guide for Frontend

This document outlines the complete IAM (Identity and Access Management) implementation for the EPSX frontend application.

## Overview

The IAM system is built using Firebase Authentication and includes:
- User authentication (login/register)
- Role-based access control (RBAC)
- Protected routes
- Session management
- User permissions

## Architecture

### Core Components

1. **AuthProvider** (`context/auth-context.tsx`) - Main authentication context
2. **Firebase IAM** (`lib/firebase-iam.ts`) - Firebase configuration
3. **API Routes** (`app/api/auth/`) - Authentication endpoints
4. **Middleware** (`middleware.ts`) - Route protection
5. **Hooks** (`hooks/useAuth.ts`) - Authentication utilities

### File Structure

```
apps/frontend/
├── app/
│   ├── api/auth/
│   │   ├── login/route.ts
│   │   ├── register/route.ts
│   │   ├── logout/route.ts
│   │   └── me/route.ts
│   ├── login/page.tsx
│   ├── register/page.tsx
│   └── layout.tsx (with AuthProvider)
├── lib/
│   ├── firebase-iam.ts
│   └── firebase-iam-helpers.ts
├── context/
│   └── auth-context.tsx
├── hooks/
│   └── useAuth.ts
└── middleware.ts
```

## Setup Instructions

### 1. Environment Variables

Ensure your `.env.local` file contains:

```bash
NEXT_PUBLIC_FIREBASE_API_KEY=your_api_key
NEXT_PUBLIC_FIREBASE_AUTH_DOMAIN=your_auth_domain
NEXT_PUBLIC_FIREBASE_PROJECT_ID=your_project_id
NEXT_PUBLIC_FIREBASE_STORAGE_BUCKET=your_storage_bucket
NEXT_PUBLIC_FIREBASE_MESSAGING_SENDER_ID=your_sender_id
NEXT_PUBLIC_FIREBASE_APP_ID=your_app_id
NEXT_PUBLIC_FIREBASE_MEASUREMENT_ID=your_measurement_id
```

### 2. Firebase Configuration

The Firebase configuration is automatically loaded from environment variables in `lib/firebase-iam.ts`.

### 3. Usage Examples

#### Login
```typescript
import { useAuth } from '@/hooks/useAuth';

function LoginComponent() {
  const { login } = useAuth();
  
  const handleLogin = async () => {
    try {
      await login(email, password);
      // Redirect to dashboard
    } catch (error) {
      console.error('Login failed:', error);
    }
  };
}
```

#### Register
```typescript
import { useAuth } from '@/hooks/useAuth';

function RegisterComponent() {
  const { register } = useAuth();
  
  const handleRegister = async () => {
    try {
      await register(email, password, displayName);
      // Redirect to dashboard
    } catch (error) {
      console.error('Registration failed:', error);
    }
  };
}
```

#### Protected Component
```typescript
import { useAuth } from '@/hooks/useAuth';

function ProtectedComponent() {
  const { user, loading } = useAuth();
  
  if (loading) return <div>Loading...</div>;
  if (!user) return <div>Please login</div>;
  
  return <div>Welcome {user.displayName}</div>;
}
```

### 4. Route Protection

Protected routes are automatically handled by middleware:
- `/dashboard/*` - Requires authentication
- `/analytics/*` - Requires authentication
- `/my-data/*` - Requires authentication

### 5. User Permissions

User permissions are stored in Firestore under the `users` collection:
- `role`: User role (user, admin, etc.)
- `permissions`: Array of permission strings
- `subscriptionLevel`: Subscription tier

### 6. API Endpoints

- `POST /api/auth/login` - User login
- `POST /api/auth/register` - User registration
- `POST /api/auth/logout` - User logout
- `GET /api/auth/me` - Get current user info

## Security Features

1. **Token-based authentication** - Uses Firebase ID tokens
2. **HTTPS only cookies** - Secure session management
3. **CSRF protection** - Built into Next.js API routes
4. **Input validation** - Server-side validation for all auth endpoints
5. **Rate limiting** - Can be added to API routes

## Testing

### Manual Testing
1. Visit `/login` to test login functionality
2. Visit `/register` to test registration
3. Try accessing `/dashboard` without authentication (should redirect to login)
4. Login and verify access to protected routes

### Automated Testing
```bash
# Run tests
npm test

# Run e2e tests
npm run test:e2e
```

## Troubleshooting

### Common Issues

1. **Firebase configuration errors**
   - Check environment variables
   - Verify Firebase project settings

2. **Authentication failures**
   - Check browser console for errors
   - Verify Firebase Auth is enabled in console

3. **Route protection not working**
   - Check middleware.ts configuration
   - Verify cookie settings

### Debug Mode

Enable debug mode by setting:
```bash
NEXT_PUBLIC_DEBUG=true
```

This will show additional authentication information in the AuthDebugger component.

## Next Steps

1. Add role-based UI components
2. Implement subscription management
3. Add admin panel for user management
4. Implement password reset flow
5. Add social login providers
