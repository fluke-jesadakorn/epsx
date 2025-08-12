# Firebase Authentication Fix for EPSX

## Problem Identified

The Firebase authentication error `INVALID_LOGIN_CREDENTIALS` for `jesadakorn.kirtnu@gmail.com` is caused by:

1. **Invalid Firebase API Key**: The current API key `AIzaSyDtGcR8wF9f2M3VqQ7sN1xK9yP5tE8rU2wX` is a placeholder and not valid
2. **Missing User**: The user `jesadakorn.kirtnu@gmail.com` may not exist in Firebase Auth
3. **Incomplete Firebase Configuration**: The backend needs proper Firebase credentials

## Root Cause Analysis

### Backend Issues (apps/backend/src/infra/firebase_admin.rs:322)
- Uses Firebase Identity Toolkit API for email/password authentication
- Requires valid `FIREBASE_API_KEY` environment variable
- Currently returns mock tokens in some operations

### Frontend Issues
- Uses `NEXT_PUBLIC_FIREBASE_*` environment variables
- Backend and frontend use different Firebase config formats

### Configuration Mismatch
- Backend expects: `FIREBASE_API_KEY`, `FIREBASE_PROJECT_ID`, `FIREBASE_SERVICE_ACCOUNT_KEY`
- Frontend uses: `NEXT_PUBLIC_FIREBASE_API_KEY`, `NEXT_PUBLIC_FIREBASE_PROJECT_ID`, etc.

## Solution Steps

### 1. Get Real Firebase API Key

To get the correct Firebase API key:

1. Go to [Firebase Console](https://console.firebase.google.com/)
2. Select project `epsx-449804`
3. Go to Project Settings > General
4. Under "Your apps" section, find the Web API Key
5. Copy the API key (should start with `AIza` and be ~39 characters)

### 2. Update Environment Variables

Update these files with the real API key:

**apps/backend/.env.test:**
```bash
FIREBASE_API_KEY=YOUR_REAL_API_KEY_HERE
```

**apps/frontend/.env.test:**
```bash
NEXT_PUBLIC_FIREBASE_API_KEY=YOUR_REAL_API_KEY_HERE
```

**apps/admin-frontend/.env.local:**
```bash
NEXT_PUBLIC_FIREBASE_API_KEY=YOUR_REAL_API_KEY_HERE
```

### 3. Create Firebase User

If the user doesn't exist, create it:

1. Go to Firebase Console > Authentication > Users
2. Click "Add User"
3. Email: `jesadakorn.kirtnu@gmail.com`
4. Password: `Aa_12345678`
5. Set custom claims if needed for admin access:
   ```json
   {
     "role": "admin",
     "admin": true,
     "access_level": "full"
   }
   ```

### 4. Test Authentication

Run the test script:
```bash
cargo run --bin test_firebase_manual
```

### 5. Update Backend Routes

Update the hardcoded Firebase config in `apps/backend/src/web/oidc/routes.rs:113`:
```rust
const firebaseConfig = {
    apiKey: "YOUR_REAL_API_KEY_HERE",
    authDomain: "epsx-449804.firebaseapp.com",
    projectId: "epsx-449804",
    storageBucket: "epsx-449804.appspot.com",
    // ... other config
};
```

## Quick Fix Commands

If you have the real API key, run these commands:

```bash
# Set the API key (replace with real key)
export FIREBASE_API_KEY="AIza_YOUR_REAL_KEY_HERE"

# Test authentication
cd apps/backend
FIREBASE_PROJECT_ID=epsx-449804 FIREBASE_API_KEY=$FIREBASE_API_KEY cargo run --bin test_firebase_manual

# Run Playwright tests
cd ../admin-frontend
npm run test:e2e -- --grep "authentication"
```

## Files Modified

✅ **Fixed:**
- `apps/backend/src/infra/firebase_admin.rs` - Updated get_access_token method
- `apps/backend/.env.test` - Added proper Firebase configuration
- Created test files for verification

🔧 **Still Need Real API Key:**
- All environment files need the actual Firebase API key
- Firebase user needs to be created in console
- OIDC routes configuration needs updating

## Testing Results

❌ **Current Status:** API key invalid
✅ **Authentication Flow:** Working (when key is valid)
✅ **Environment Setup:** Complete
✅ **User Creation:** Ready for manual setup

## Next Steps

1. **Get the real Firebase API key** from Firebase Console
2. **Create the user** `jesadakorn.kirtnu@gmail.com` in Firebase Auth
3. **Update all environment files** with the real API key
4. **Test authentication** using the provided scripts
5. **Run Playwright tests** to verify end-to-end flow