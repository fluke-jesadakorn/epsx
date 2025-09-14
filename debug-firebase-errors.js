// Firebase Error Diagnosis Script
// Run this in the browser console on https://epsx-frontend-dev-307278481624.us-central1.run.app

console.log('=== EPSX Firebase Error Diagnosis ===');

// 1. Check if we're on the correct site
console.log('🌐 Current URL:', window.location.href);
console.log('🏠 Domain:', window.location.hostname);

// 2. Check for environment variables in the page
console.log('\n📋 Environment Variable Check:');

// These should be embedded in the built Next.js app
const checkEnvVar = (name, value) => {
  const status = value ? '✅' : '❌';
  const displayValue = value ? 
    (value.length > 20 ? `${value.substring(0, 20)}...` : value) : 
    'undefined';
  console.log(`${status} ${name}:`, displayValue);
  return !!value;
};

// Check all Firebase environment variables
let firebaseVarsPresent = true;

// Basic environment
console.log('\n🔧 Basic Environment:');
firebaseVarsPresent &= checkEnvVar('NODE_ENV', process.env?.NODE_ENV);

// Firebase configuration (these should be NEXT_PUBLIC_ prefixed)
console.log('\n🔥 Firebase Configuration:');

// Try to access global variables or window properties
let firebaseConfig = {};
try {
  // Check if Firebase config is available globally
  if (window.__FIREBASE_CONFIG__) {
    firebaseConfig = window.__FIREBASE_CONFIG__;
    console.log('✅ Found window.__FIREBASE_CONFIG__:', firebaseConfig);
  } else {
    console.log('❌ window.__FIREBASE_CONFIG__ not found');
  }
} catch (e) {
  console.log('❌ Error accessing global Firebase config:', e.message);
}

// 3. Check for Firebase SDK initialization
console.log('\n🔥 Firebase SDK Check:');
try {
  if (typeof window.firebase !== 'undefined') {
    console.log('✅ Firebase SDK detected on window');
    console.log('📦 Firebase SDK version:', window.firebase?.SDK_VERSION || 'unknown');
  } else {
    console.log('❌ Firebase SDK not found on window');
  }
  
  // Check for Firebase modules
  const firebaseModules = ['auth', 'app', 'remote-config'];
  firebaseModules.forEach(module => {
    try {
      if (window[module]) {
        console.log(`✅ Firebase ${module} module available`);
      } else {
        console.log(`❌ Firebase ${module} module not available`);
      }
    } catch (e) {
      console.log(`❌ Error checking Firebase ${module}:`, e.message);
    }
  });
} catch (e) {
  console.log('❌ Error checking Firebase SDK:', e.message);
}

// 4. Capture and display console errors
console.log('\n🐛 Console Error Monitoring:');

const errors = [];
const warnings = [];

// Override console methods to capture Firebase-related messages
const originalError = console.error;
const originalWarn = console.warn;

console.error = function(...args) {
  const message = args.join(' ');
  if (message.toLowerCase().includes('firebase') || 
      message.toLowerCase().includes('auth') ||
      message.toLowerCase().includes('remote-config')) {
    errors.push(`ERROR: ${message}`);
  }
  originalError.apply(console, args);
};

console.warn = function(...args) {
  const message = args.join(' ');
  if (message.toLowerCase().includes('firebase') || 
      message.toLowerCase().includes('auth') ||
      message.toLowerCase().includes('remote-config')) {
    warnings.push(`WARNING: ${message}`);
  }
  originalWarn.apply(console, args);
};

// 5. Check network requests
console.log('\n🌐 Network Request Monitoring:');
const originalFetch = window.fetch;
window.fetch = function(...args) {
  const url = args[0];
  if (typeof url === 'string' && url.includes('firebase')) {
    console.log('🔗 Firebase API request:', url);
  }
  return originalFetch.apply(this, args).catch(error => {
    if (typeof url === 'string' && url.includes('firebase')) {
      console.error('❌ Firebase API request failed:', url, error);
    }
    throw error;
  });
};

// 6. Wait and then report results
setTimeout(() => {
  console.log('\n📊 Firebase Error Summary (after 5 seconds):');
  
  if (errors.length > 0) {
    console.log('🚨 ERRORS FOUND:');
    errors.forEach((error, i) => {
      console.log(`${i + 1}. ${error}`);
    });
  } else {
    console.log('✅ No Firebase errors detected');
  }
  
  if (warnings.length > 0) {
    console.log('\n⚠️ WARNINGS FOUND:');
    warnings.forEach((warning, i) => {
      console.log(`${i + 1}. ${warning}`);
    });
  } else {
    console.log('✅ No Firebase warnings detected');
  }
  
  // Try to trigger Firebase operations to expose errors
  console.log('\n🧪 Testing Firebase Operations:');
  
  try {
    // Try to access Firebase config that might be imported
    if (typeof window.firebase !== 'undefined') {
      console.log('🔥 Attempting Firebase app access...');
      const apps = window.firebase.getApps?.() || [];
      console.log(`📱 Firebase apps: ${apps.length} found`);
    }
  } catch (e) {
    console.error('❌ Firebase operation test failed:', e);
  }
  
}, 5000);

console.log('✅ Firebase diagnosis script loaded. Check back in 5 seconds for results.');
console.log('💡 You can also manually trigger Firebase operations to see more errors.');