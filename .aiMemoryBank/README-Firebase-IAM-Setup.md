# Firebase IAM Setup Guide

## 🚀 Quick Start

Your IAM system now includes Firebase Firestore integration with automatic fallback to mock data. If you're seeing "Failed to fetch users from Firebase" errors, follow this guide.

## 📋 Prerequisites

1. **Firebase Project**: You need a Firebase project with Firestore enabled
2. **Environment Variables**: Firebase configuration in your .env files
3. **Security Rules**: Proper Firestore security rules deployed

## 🔧 Setup Steps

### 1. Firebase Console Setup

1. Go to [Firebase Console](https://console.firebase.google.com)
2. Create a new project or select existing one
3. Enable **Firestore Database**
4. Choose "Start in test mode" for now (we'll update security rules later)

### 2. Environment Configuration

Add these to your `.env.local` file in the admin-frontend directory:

```bash
NEXT_PUBLIC_FIREBASE_API_KEY=your_api_key_here
NEXT_PUBLIC_FIREBASE_AUTH_DOMAIN=your_project.firebaseapp.com
NEXT_PUBLIC_FIREBASE_PROJECT_ID=your_project_id
NEXT_PUBLIC_FIREBASE_STORAGE_BUCKET=your_project.appspot.com
NEXT_PUBLIC_FIREBASE_MESSAGING_SENDER_ID=your_sender_id
NEXT_PUBLIC_FIREBASE_APP_ID=your_app_id
```

You can find these values in:
- Firebase Console → Project Settings → General → Your apps → Config

### 3. Deploy Security Rules

Copy the content from `firestore.rules` and deploy it:

1. Install Firebase CLI: `npm install -g firebase-tools`
2. Login: `firebase login`
3. Initialize: `firebase init firestore` (in your project root)
4. Copy rules from `apps/admin-frontend/firestore.rules` to `firestore.rules`
5. Deploy: `firebase deploy --only firestore:rules`

### 4. Initialize Collections

Use the Debug Panel in the IAM Dashboard:

1. Go to IAM Dashboard → Debug tab
2. Click "Run Diagnostic" to check connection
3. Click "Initialize Collections" to create sample data
4. Verify collections are created in Firebase Console

## 🛠 Troubleshooting

### Error: "Failed to fetch users from Firebase"

**Possible causes:**
1. Missing or incorrect environment variables
2. Firestore not enabled
3. Security rules blocking access
4. No data in collections

**Solutions:**
1. Check environment variables in `.env.local`
2. Enable Firestore in Firebase Console
3. Deploy security rules (see step 3 above)
4. Initialize collections (see step 4 above)

### Error: "Permission denied"

**Cause:** Security rules are blocking access

**Solution:**
- Temporarily use test mode rules: `allow read, write: if true;`
- Later implement proper admin authentication

### Mock Data Being Used

If you see mock data instead of Firebase data:
1. Check browser console for Firebase errors
2. Use the Debug Panel to diagnose issues
3. Ensure Firebase is properly initialized

## 📊 Firebase Collections Structure

The IAM system creates these collections:

```
users/
  ├── {userId}
  └── packageTier, subscriptionStatus, email, etc.

custom_permissions/
  ├── {permissionId}
  └── userId, featureId, permission, grantedBy, etc.

effective_permissions/
  ├── {effectivePermissionId}
  └── userId, featureId, permission, source, etc.

permission_audit_logs/
  ├── {logId}
  └── userId, action, performedBy, timestamp, etc.
```

## 🔄 Migration from Mock to Firebase

The system automatically falls back to mock data when Firebase is unavailable:

1. **Development**: Mock data allows development without Firebase setup
2. **Staging**: Initialize Firebase and collections
3. **Production**: Full Firebase integration with real user data

## 🧪 Testing the Integration

1. **Debug Panel**: Use IAM Dashboard → Debug tab
2. **Payment Integration**: Test payment events with the Testing Tools tab
3. **Permission Checks**: Verify users can access features based on their package tier

## 📈 Production Deployment

1. **Environment Variables**: Set in production environment
2. **Security Rules**: Deploy proper security rules
3. **Monitoring**: Set up Firebase monitoring and alerts
4. **Backup**: Configure Firestore backup

## 🆘 Support

If you continue to have issues:

1. Check the Debug Panel for detailed error information
2. Review browser console for Firebase connection errors
3. Verify Firebase project configuration
4. Ensure all environment variables are correctly set

The system is designed to work seamlessly whether Firebase is available or not, so you can continue development even while setting up Firebase.
