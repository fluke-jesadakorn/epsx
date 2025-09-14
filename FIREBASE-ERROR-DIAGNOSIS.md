# Firebase Console Error Diagnosis Guide

## Current Issue
The user reports Firebase errors on the live site: https://epsx-frontend-dev-307278481624.us-central1.run.app

## Steps to Diagnose

### 1. Open Browser Developer Tools
1. Navigate to https://epsx-frontend-dev-307278481624.us-central1.run.app
2. Open Developer Tools (F12 or right-click → Inspect)
3. Go to the Console tab
4. Refresh the page to capture all initialization errors

### 2. Run Diagnostic Script
Copy and paste the following script into the browser console:

```javascript
// FIREBASE ERROR DIAGNOSIS SCRIPT
console.log('=== EPSX Firebase Error Diagnosis ===');

// Check environment variables
const checkFirebaseEnv = () => {
  console.log('🔥 Firebase Environment Variables Check:');
  
  // These should be embedded in the built Next.js app at build time
  const firebaseVars = [
    'NEXT_PUBLIC_FIREBASE_API_KEY',
    'NEXT_PUBLIC_FIREBASE_AUTH_DOMAIN', 
    'NEXT_PUBLIC_FIREBASE_PROJECT_ID',
    'NEXT_PUBLIC_FIREBASE_STORAGE_BUCKET',
    'NEXT_PUBLIC_FIREBASE_MESSAGING_SENDER_ID',
    'NEXT_PUBLIC_FIREBASE_APP_ID',
    'NEXT_PUBLIC_FIREBASE_MEASUREMENT_ID'
  ];
  
  // Try to access process.env (only available server-side, not in browser)
  firebaseVars.forEach(varName => {
    const value = typeof process !== 'undefined' && process.env ? process.env[varName] : undefined;
    const status = value ? '✅' : '❌';
    const displayValue = value && value.length > 20 ? `${value.substring(0, 20)}...` : (value || 'undefined');
    console.log(`${status} ${varName}:`, displayValue);
  });
  
  console.log('📋 Note: In production, these variables are embedded at build time and not accessible via process.env in browser');
};

// Check Firebase SDK
const checkFirebaseSDK = () => {
  console.log('\n🔥 Firebase SDK Initialization Check:');
  
  try {
    // Check if Firebase modules are loaded
    if (typeof window.firebase !== 'undefined') {
      console.log('✅ Firebase SDK loaded globally');
    } else {
      console.log('❌ Firebase SDK not found on window object');
    }
    
    // Try to access Firebase config (if it exists)
    if (window.__FIREBASE_CONFIG__) {
      console.log('✅ Firebase config found:', window.__FIREBASE_CONFIG__);
    } else {
      console.log('❌ Firebase config not found on window object');
    }
    
  } catch (error) {
    console.error('❌ Error checking Firebase SDK:', error);
  }
};

// Monitor console for Firebase errors
const monitorFirebaseErrors = () => {
  console.log('\n🐛 Firebase Error Monitoring (for next 10 seconds):');
  
  const errors = [];
  const warnings = [];
  
  const originalError = console.error;
  const originalWarn = console.warn;
  
  console.error = function(...args) {
    const message = args.join(' ');
    if (message.toLowerCase().includes('firebase') || 
        message.toLowerCase().includes('auth') ||
        message.toLowerCase().includes('remote-config')) {
      errors.push(message);
    }
    originalError.apply(console, args);
  };
  
  console.warn = function(...args) {
    const message = args.join(' ');
    if (message.toLowerCase().includes('firebase') || 
        message.toLowerCase().includes('auth') ||
        message.toLowerCase().includes('remote-config')) {
      warnings.push(message);
    }
    originalWarn.apply(console, args);
  };
  
  setTimeout(() => {
    console.log('\n📊 Firebase Error Summary:');
    
    if (errors.length > 0) {
      console.log('🚨 FIREBASE ERRORS FOUND:');
      errors.forEach((error, i) => {
        console.log(`${i + 1}. ${error}`);
      });
    } else {
      console.log('✅ No Firebase errors detected');
    }
    
    if (warnings.length > 0) {
      console.log('\n⚠️ FIREBASE WARNINGS FOUND:');
      warnings.forEach((warning, i) => {
        console.log(`${i + 1}. ${warning}`);
      });
    } else {
      console.log('✅ No Firebase warnings detected');
    }
    
    // Restore original console methods
    console.error = originalError;
    console.warn = originalWarn;
    
  }, 10000);
};

// Check network requests
const monitorNetworkRequests = () => {
  console.log('\n🌐 Firebase Network Request Monitoring:');
  
  const originalFetch = window.fetch;
  window.fetch = function(...args) {
    const url = args[0];
    if (typeof url === 'string' && (
        url.includes('firebase') || 
        url.includes('googleapis') ||
        url.includes('firebaseapp')
    )) {
      console.log('🔗 Firebase API request:', url);
    }
    
    return originalFetch.apply(this, args).catch(error => {
      if (typeof url === 'string' && (
          url.includes('firebase') || 
          url.includes('googleapis') ||
          url.includes('firebaseapp')
      )) {
        console.error('❌ Firebase API request failed:', url, error);
      }
      throw error;
    });
  };
};

// Run all checks
checkFirebaseEnv();
checkFirebaseSDK();
monitorFirebaseErrors();
monitorNetworkRequests();

console.log('✅ Firebase diagnosis script loaded. Monitor console for the next 10 seconds...');
```

### 3. Expected Firebase Errors (Before Fix)

Based on code analysis, you should see these errors:

```
❌ Firebase configuration validation failed. Missing or invalid fields: apiKey, projectId, appId
🔧 Firebase config values: { 
  apiKey: 'undefined', 
  authDomain: 'undefined', 
  projectId: 'undefined', 
  ... 
}
⚠️ Firebase initialization skipped due to invalid configuration
❌ Firebase Auth initialization failed: FirebaseError: Error (auth/invalid-api-key)
❌ Firebase Remote Config initialization failed: FirebaseError: Error (remote-config/fetch-failed)
```

### 4. Root Cause Analysis

The issue is likely that the production deployment doesn't have the `NEXT_PUBLIC_FIREBASE_*` environment variables properly set.

**In Next.js**, client-side environment variables must:
1. Start with `NEXT_PUBLIC_` prefix
2. Be available at **build time** (not just runtime)
3. Be embedded into the JavaScript bundle during build

### 5. Check Deployment Configuration

The current deployment likely has these issues:
1. ❌ Missing `NEXT_PUBLIC_FIREBASE_API_KEY` 
2. ❌ Missing `NEXT_PUBLIC_FIREBASE_APP_ID`
3. ❌ Missing `NEXT_PUBLIC_FIREBASE_PROJECT_ID`
4. ❌ Other optional Firebase variables

### 6. Immediate Fix Applied

I've updated `/apps/frontend/lib/firebase.ts` with:

1. **Configuration Validation**: Checks if Firebase config is valid before initialization
2. **Graceful Degradation**: Skips Firebase initialization with clear error messages if config is invalid
3. **Better Error Handling**: Prevents crashes when Firebase variables are undefined
4. **Detailed Logging**: Shows exactly which variables are missing

### 7. Next Steps

After running the diagnostic script, you should either:

**Option A: If Firebase variables are missing:**
- Update the deployment script to include `NEXT_PUBLIC_FIREBASE_*` variables
- Redeploy with proper Firebase configuration

**Option B: If Firebase is not needed in production:**
- The current fix will gracefully handle missing Firebase config
- Authentication can work without Firebase client-side initialization

### 8. Test the Fix

After deploying the updated `firebase.ts`, you should see these improved error messages instead of crashes:

```
🔧 Firebase initialization skipped due to invalid configuration
💡 To enable Firebase features, ensure NEXT_PUBLIC_FIREBASE_* environment variables are properly set
⚠️ Firebase Auth not initialized - invalid or missing configuration
📋 Required environment variables:
  - NEXT_PUBLIC_FIREBASE_API_KEY (35+ chars)
  - NEXT_PUBLIC_FIREBASE_PROJECT_ID (5+ chars) 
  - NEXT_PUBLIC_FIREBASE_APP_ID (15+ chars)
```

## Summary

The fix I've implemented will:
1. ✅ Prevent Firebase initialization crashes
2. ✅ Provide clear diagnostic information
3. ✅ Allow the application to run without Firebase client-side features
4. ✅ Show exactly what environment variables are needed