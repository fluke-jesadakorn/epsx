import { db } from '../lib/firebase';
import { collection, getDocs, doc, setDoc } from 'firebase/firestore';

/**
 * Firebase IAM Configuration Checker
 * Use this to debug Firebase setup issues
 */
export class FirebaseIAMChecker {
  
  /**
   * Check if Firebase is properly configured
   */
  static async checkFirebaseConnection(): Promise<{
    isConnected: boolean;
    error?: string;
    details: Record<string, any>;
  }> {
    const details: Record<string, any> = {};
    
    try {
      // Check if db is initialized
      if (!db) {
        return {
          isConnected: false,
          error: 'Firebase db is not initialized',
          details: { db: 'Not initialized' }
        };
      }
      
      details.db = 'Initialized';
      
      // Try to read from a collection
      try {
        const testCollection = collection(db, 'users');
        const snapshot = await getDocs(testCollection);
        details.readAccess = `Success (${snapshot.size} documents)`;
      } catch (readError) {
        details.readAccess = `Failed: ${readError}`;
      }
      
      // Try to write to a test collection
      try {
        const testDoc = doc(db, 'iam_test', 'connection_test');
        await setDoc(testDoc, {
          timestamp: new Date(),
          test: 'Firebase IAM connection test'
        }, { merge: true });
        details.writeAccess = 'Success';
      } catch (writeError) {
        details.writeAccess = `Failed: ${writeError}`;
      }
      
      return {
        isConnected: true,
        details
      };
      
    } catch (error) {
      return {
        isConnected: false,
        error: error instanceof Error ? error.message : 'Unknown error',
        details
      };
    }
  }
  
  /**
   * Check IAM collections structure
   */
  static async checkIAMCollections(): Promise<{
    collections: Record<string, { exists: boolean; count: number; error?: string }>;
  }> {
    const collections: Record<string, { exists: boolean; count: number; error?: string }> = {
      users: { exists: false, count: 0 },
      custom_permissions: { exists: false, count: 0 },
      effective_permissions: { exists: false, count: 0 },
      permission_audit_logs: { exists: false, count: 0 }
    };
    
    for (const collectionName of Object.keys(collections)) {
      try {
        const collectionRef = collection(db, collectionName);
        const snapshot = await getDocs(collectionRef);
        collections[collectionName] = {
          exists: true,
          count: snapshot.size
        };
      } catch (error) {
        collections[collectionName] = {
          exists: false,
          count: 0,
          error: error instanceof Error ? error.message : 'Unknown error'
        };
      }
    }
    
    return { collections };
  }
  
  /**
   * Initialize basic IAM collections with sample data
   */
  static async initializeBasicCollections(): Promise<void> {
    try {
      console.log('🚀 Initializing basic IAM collections...');
      
      // Create a test user
      const userRef = doc(db, 'users', 'test-user-001');
      await setDoc(userRef, {
        email: 'test@epsx.com',
        name: 'Test User',
        packageTier: 'FREE',
        subscriptionStatus: 'ACTIVE',
        createdAt: new Date(),
        updatedAt: new Date()
      });
      
      console.log('✅ Basic collections initialized');
      
    } catch (error) {
      console.error('❌ Failed to initialize collections:', error);
      throw error;
    }
  }
  
  /**
   * Run complete diagnostic
   */
  static async runDiagnostic(): Promise<void> {
    console.log('🔍 Running Firebase IAM Diagnostic...');
    
    // Check connection
    const connectionResult = await this.checkFirebaseConnection();
    console.log('📡 Connection Status:', connectionResult);
    
    if (!connectionResult.isConnected) {
      console.log('❌ Firebase connection failed. Please check:');
      console.log('   1. Firebase configuration in .env files');
      console.log('   2. Firebase project setup');
      console.log('   3. Firestore database creation');
      return;
    }
    
    // Check collections
    const collectionsResult = await this.checkIAMCollections();
    console.log('📚 Collections Status:', collectionsResult);
    
    // Check if collections are empty
    const hasData = Object.values(collectionsResult.collections).some(col => col.count > 0);
    
    if (!hasData) {
      console.log('⚠️  No data found in IAM collections.');
      console.log('💡 Consider running initializeBasicCollections() to create test data');
    }
    
    console.log('✅ Diagnostic complete');
  }
}

// Utility function to run diagnostic from console
export const runFirebaseIAMDiagnostic = () => FirebaseIAMChecker.runDiagnostic();
export const initFirebaseIAMCollections = () => FirebaseIAMChecker.initializeBasicCollections();
