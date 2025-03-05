#!/usr/bin/env node
import { auth } from '../shared';

async function makeAdmin(email: string) {
  try {
    // Initialize Firebase Admin
    const user = await auth.getUserByEmail(email);
    
    // Set custom claims for admin
    await auth.setCustomUserClaims(user.uid, {
      admin: true,
      premium: true // Admins get premium access too
    });

    console.log(`Successfully made ${email} an admin`);
  } catch (error: any) {
    console.error('Error making user admin:', error?.message || error);
    process.exit(1);
  }
}

// Get email from command line argument
const email = process.argv[2];
if (!email) {
  console.error('Please provide an email address');
  console.error('Usage: bun run make-admin.ts <email>');
  process.exit(1);
}

makeAdmin(email);
