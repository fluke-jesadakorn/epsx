// Firebase Authentication Module Exports

export {
  FirebaseAuthManager,
  FirebaseAuthError,
  getFirebaseAuth,
  initializeFirebaseAuth,
  type FirebaseAuthResult,
  type FirebaseUserProfile,
  type FirebaseAuthConfig
} from './firebase-auth';

export { 
  FirebaseTokenValidator,
  getFirebaseTokenValidator,
  initializeFirebaseTokenValidator,
  TokenValidationError,
  type UnifiedJWT
} from './token-validator';