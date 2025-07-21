# Firebase IAM Integration Setup Guide

This guide will help you set up the complete Firebase IAM (Identity and Access Management) system that combines Firebase Authentication with Firestore-based permission management.

## 🚀 Quick Start

1. **Configure Firebase** (see [Firebase Setup](#firebase-setup) below)
2. **Initialize IAM System**: `npm run firebase-init`
3. **Start the admin app**: `npm run dev`
4. **Test the integration** in the IAM Dashboard

## 📋 Prerequisites

- Firebase project with Authentication and Firestore enabled
- Firebase configuration in your environment variables or Firebase config
- Node.js and npm/pnpm installed

## 🔥 Firebase Setup

### 1. Create Firebase Project

1. Go to [Firebase Console](https://console.firebase.google.com)
2. Create a new project or select existing one
3. Enable **Authentication** and **Firestore Database**

### 2. Configure Authentication

1. In Firebase Console → Authentication → Sign-in method
2. Enable **Email/Password** provider
3. Optionally enable other providers (Google, GitHub, etc.)

### 3. Configure Firestore

1. In Firebase Console → Firestore Database
2. Create database in test mode (or production with security rules)
3. The initialization script will create the required collections

### 4. Get Firebase Configuration

1. In Firebase Console → Project Settings → General
2. Scroll down to "Your apps" and click on your web app
3. Copy the Firebase configuration object
4. Add to your environment variables or `lib/firebase.ts`

```javascript
// Example Firebase config
const firebaseConfig = {
  apiKey: "your-api-key",
  authDomain: "your-project.firebaseapp.com",
  projectId: "your-project-id",
  storageBucket: "your-project.appspot.com",
  messagingSenderId: "123456789",
  appId: "your-app-id"
};
```

## 🛠️ Installation & Setup

### 1. Initialize Firebase IAM System

Run the initialization script to create sample users and permissions:

```bash
npm run firebase-init
```

This script will:
- Create Firebase Auth users for each package tier (Free, Bronze, Silver, Gold, Platinum)
- Set up user profiles in Firestore with IAM data
- Apply package-based permissions automatically
- Create sample custom permissions
- Set up audit logging

### 2. Sample Users Created

The initialization creates these test users:

| Email | Password | Package Tier | Features |
|-------|----------|--------------|----------|
| `free@example.com` | `test123456` | Free | Basic features |
| `bronze@example.com` | `test123456` | Bronze | Analytics, API access |
| `silver@example.com` | `test123456` | Silver | + Advanced charts, exports |
| `gold@example.com` | `test123456` | Gold | + Real-time data, priority support |
| `platinum@example.com` | `test123456` | Platinum | All features + white-label |

## 🔍 Testing the Integration

### 1. Use the IAM Dashboard

1. Start the admin app: `npm run dev`
2. Navigate to `/users` (IAM Dashboard)
3. Click on the "Firebase Debug" tab
4. Test Firebase connection and view diagnostics

### 2. Authentication Flow

```typescript
import { useFirebaseAuth } from '../hooks/useFirebaseAuth';

function MyComponent() {
  const { user, signIn, signOut, hasFeatureAccess } = useFirebaseAuth();
  
  // Check if user has access to a feature
  const canUseAdvancedAnalytics = hasFeatureAccess('advanced_analytics');
  
  // Sign in a user
  const handleSignIn = async () => {
    try {
      await signIn('bronze@example.com', 'test123456');
    } catch (error) {
      console.error('Sign in failed:', error);
    }
  };
  
  return (
    <div>
      {user ? (
        <div>
          <h2>Welcome, {user.email}</h2>
          <p>Package: {user.packageTier}</p>
          {canUseAdvancedAnalytics && (
            <button>Advanced Analytics</button>
          )}
          <button onClick={signOut}>Sign Out</button>
        </div>
      ) : (
        <button onClick={handleSignIn}>Sign In</button>
      )}
    </div>
  );
}
```

### 3. Direct Service Usage

```typescript
import { firebaseAuthIAMService } from '../services/firebaseAuthIAMService';
import { firebaseIAMService } from '../services/firebaseIAMService';

// Create a new user with IAM profile
const newUser = await firebaseAuthIAMService.createUser({
  email: 'user@example.com',
  password: 'password123',
  packageTier: 'SILVER',
  subscriptionStatus: 'ACTIVE'
});

// Check user permissions
const hasAccess = await firebaseIAMService.checkUserPermission(
  newUser.uid,
  'advanced_analytics'
);

// Grant custom permission
await firebaseIAMService.grantCustomPermission(
  newUser.uid,
  'beta_feature',
  { action: 'READ', resource: 'beta_feature' },
  'admin_user_id'
);
```

## 🏗️ Architecture Overview

### Services

1. **`firebaseIAMService.ts`** - Firestore operations for IAM data
2. **`firebaseAuthIAMService.ts`** - Firebase Auth + IAM integration  
3. **`useFirebaseAuth.ts`** - React hook for auth state and permissions

### Data Structure

#### Users Collection (`/users/{userId}`)
```typescript
{
  id: string;
  email: string;
  packageTier: 'FREE' | 'BRONZE' | 'SILVER' | 'GOLD' | 'PLATINUM';
  subscriptionStatus: 'ACTIVE' | 'INACTIVE' | 'CANCELLED';
  permissions: Permission[];
  customPermissions: CustomPermission[];
  createdAt: Date;
  updatedAt: Date;
}
```

#### Permissions Subcollection (`/users/{userId}/permissions/{permissionId}`)
```typescript
{
  id: string;
  userId: string;
  featureId: string;
  permission: { action: string; resource: string };
  grantedBy: string;
  grantedAt: Date;
  expiresAt?: Date;
  metadata?: Record<string, any>;
}
```

## 🔒 Security Considerations

### Firestore Security Rules

Add these rules to your Firestore to secure IAM data:

```javascript
rules_version = '2';
service cloud.firestore {
  match /databases/{database}/documents {
    // Users can only read their own profile
    match /users/{userId} {
      allow read: if request.auth != null && request.auth.uid == userId;
      allow write: if request.auth != null && request.auth.uid == userId;
      
      // Admin users can read/write all profiles
      allow read, write: if request.auth != null && 
        get(/databases/$(database)/documents/users/$(request.auth.uid)).data.isAdmin == true;
      
      // Permissions subcollection
      match /permissions/{permissionId} {
        allow read: if request.auth != null && request.auth.uid == userId;
        allow write: if request.auth != null && 
          get(/databases/$(database)/documents/users/$(request.auth.uid)).data.isAdmin == true;
      }
    }
    
    // Audit logs - admin only
    match /audit_logs/{logId} {
      allow read, write: if request.auth != null && 
        get(/databases/$(database)/documents/users/$(request.auth.uid)).data.isAdmin == true;
    }
  }
}
```

### Environment Variables

Set these environment variables for production:

```bash
NEXT_PUBLIC_FIREBASE_API_KEY=your-api-key
NEXT_PUBLIC_FIREBASE_AUTH_DOMAIN=your-project.firebaseapp.com
NEXT_PUBLIC_FIREBASE_PROJECT_ID=your-project-id
NEXT_PUBLIC_FIREBASE_STORAGE_BUCKET=your-project.appspot.com
NEXT_PUBLIC_FIREBASE_MESSAGING_SENDER_ID=123456789
NEXT_PUBLIC_FIREBASE_APP_ID=your-app-id
```

## 🚨 Troubleshooting

### Common Issues

1. **"Permission denied" errors**
   - Check Firestore security rules
   - Verify user authentication status
   - Ensure user has required permissions

2. **Firebase connection issues**
   - Use the Firebase IAM Debug Panel
   - Check Firebase configuration
   - Verify internet connection

3. **TypeScript errors**
   - Ensure all Firebase types are properly imported
   - Check that Firebase SDK is up to date

### Debug Tools

Use the built-in debug panel at `/users` → "Firebase Debug" tab:
- Test Firebase connection
- View collection status
- Check user permissions
- Create sample data
- View diagnostic information

### Fallback Mode

The system automatically falls back to mock data when Firebase is unavailable:
- Perfect for development without Firebase setup
- Seamless transition when Firebase connection is restored
- All features work the same way in both modes

## 📈 Next Steps

1. **Customize Package Permissions** - Edit `config/packagePermissions.ts`
2. **Add More Features** - Extend the permission system for new features
3. **Implement Billing** - Connect package tiers to payment processing
4. **Add Admin Tools** - Build admin interface for user management
5. **Set Up Monitoring** - Add logging and analytics for IAM operations

## 🔗 Related Files

- **Services**: `services/firebaseIAMService.ts`, `services/firebaseAuthIAMService.ts`
- **Hooks**: `hooks/useFirebaseAuth.ts`
- **Types**: `types/admin/iam-enhanced.ts`
- **Config**: `config/packagePermissions.ts`
- **Components**: `components/admin/firebase-iam-debug-panel.tsx`
- **Scripts**: `scripts/initializeFirebaseIAM.ts`

Need help? Check the debug panel or review the comprehensive error handling in the services!
