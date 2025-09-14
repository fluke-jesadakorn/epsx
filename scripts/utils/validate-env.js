#!/usr/bin/env node
/**
 * EPSX Environment Validation Script
 * Validates all required environment variables from unified schema
 */

const fs = require('fs');
const path = require('path');

// Required environment variables (from unified schema)
const REQUIRED_VARS = [
  'DATABASE_URL',
  'NEXTAUTH_SECRET',
  'OIDC_CLIENT_SECRET', 
  'OIDC_ADMIN_CLIENT_SECRET',
  'FIREBASE_PROJECT_ID',
  'FIREBASE_PRIVATE_KEY',
  'FIREBASE_CLIENT_EMAIL'
];

// Optional variables with defaults
const OPTIONAL_VARS = [
  'BACKEND_URL',
  'FRONTEND_URL',
  'ADMIN_FRONTEND_URL',
  'OIDC_CLIENT_ID',
  'OIDC_ADMIN_CLIENT_ID',
  'MUSEPAY_PARTNER_ID',
  'MUSEPAY_PRIVATE_KEY',
  'REDIS_URL',
  'LOG_LEVEL'
];

// Load environment variables
require('dotenv').config();

function validateEnvironment() {
  let hasErrors = false;
  let warnings = [];

  console.log('🔍 EPSX Environment Validation');
  console.log('=====================================\n');

  // Check required variables
  console.log('✅ Required Variables:');
  for (const varName of REQUIRED_VARS) {
    const value = process.env[varName];
    if (!value) {
      console.log(`❌ ${varName}: Missing (REQUIRED)`);
      hasErrors = true;
    } else {
      // Validate specific formats
      if (varName === 'DATABASE_URL' && !value.startsWith('postgresql://')) {
        console.log(`❌ ${varName}: Must be a PostgreSQL connection string`);
        hasErrors = true;
      } else if (varName === 'NEXTAUTH_SECRET' && value.length < 32) {
        console.log(`❌ ${varName}: Must be at least 32 characters`);
        hasErrors = true;
      } else if (varName === 'FIREBASE_PRIVATE_KEY' && !value.includes('BEGIN PRIVATE KEY')) {
        console.log(`❌ ${varName}: Must be a valid PEM private key`);
        hasErrors = true;
      } else if (varName === 'FIREBASE_CLIENT_EMAIL' && !value.includes('@')) {
        console.log(`❌ ${varName}: Must be a valid email address`);
        hasErrors = true;
      } else {
        console.log(`✅ ${varName}: Set`);
      }
    }
  }

  console.log('\n🔧 Optional Variables:');
  for (const varName of OPTIONAL_VARS) {
    const value = process.env[varName];
    if (value) {
      console.log(`✅ ${varName}: ${value}`);
    } else {
      console.log(`⚪ ${varName}: Using default`);
    }
  }

  // Check for potential security issues
  console.log('\n🔒 Security Check:');
  if (process.env.NEXTAUTH_SECRET && process.env.NEXTAUTH_SECRET.includes('dev-secret')) {
    warnings.push('NEXTAUTH_SECRET appears to be using development default - update for production');
  }
  if (process.env.OIDC_CLIENT_SECRET && process.env.OIDC_CLIENT_SECRET.includes('dev-')) {
    warnings.push('OIDC_CLIENT_SECRET appears to be using development default');
  }

  // Display results
  console.log('\n=====================================');
  if (hasErrors) {
    console.log('❌ Environment validation FAILED');
    console.log('\n💡 Quick Setup:');
    console.log('1. cp .env.example .env');
    console.log('2. Edit .env with your values');
    console.log('3. Generate secure secrets: openssl rand -base64 32');
    console.log('4. Configure Firebase project credentials');
    console.log('\nSee CLAUDE.md - Environment Architecture for details');
    process.exit(1);
  }

  if (warnings.length > 0) {
    console.log('⚠️  Environment validation passed with warnings:');
    warnings.forEach(warning => console.log(`   ${warning}`));
  } else {
    console.log('✅ Environment validation passed');
  }

  console.log('\n🚀 All required variables are set. Ready to start EPSX!');
}

// Check if .env exists
const envPath = path.join(process.cwd(), '.env');
if (!fs.existsSync(envPath)) {
  console.log('❌ .env file not found');
  console.log('💡 Run: cp .env.example .env');
  console.log('Then edit .env with your values');
  process.exit(1);
}

validateEnvironment();
