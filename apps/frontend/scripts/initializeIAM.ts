#!/usr/bin/env node

/**
 * IAM Initialization Script for Frontend
 * 
 * This script helps set up the IAM system for the frontend application.
 * It creates default roles and permissions if they don't exist.
 */

import { iamService } from '../services/iamService';

async function initializeIAM() {
  console.log('🚀 Initializing IAM system for frontend...');
  
  try {
    await iamService.initializeIAM();
    console.log('✅ IAM system initialized successfully');
    
    // Create default admin user if needed
    console.log('📋 IAM setup complete');
  } catch (error) {
    console.error('❌ Failed to initialize IAM:', error);
    process.exit(1);
  }
}

// Run initialization
if (require.main === module) {
  initializeIAM();
}

export { initializeIAM };
