#!/usr/bin/env node
import admin from "firebase-admin";
import { config } from "dotenv";

// Load environment variables from .env file
config();

console.log("Starting unassign-admin script...");

async function unassignAdmin(email: string) {
  console.log(`Attempting to remove admin rights from ${email}...`);

  try {
    const { FIREBASE_PROJECT_ID, FIREBASE_PRIVATE_KEY, FIREBASE_CLIENT_EMAIL } =
      process.env;

    console.log(process.env);

    if (
      !FIREBASE_PROJECT_ID ||
      !FIREBASE_PRIVATE_KEY ||
      !FIREBASE_CLIENT_EMAIL
    ) {
      throw new Error("Missing required Firebase configuration in .env file");
    }

    console.log("Initializing Firebase Admin...");

    // Initialize Firebase Admin with service account
    if (!admin.apps.length) {
      admin.initializeApp({
        credential: admin.credential.cert({
          projectId: FIREBASE_PROJECT_ID,
          privateKey: FIREBASE_PRIVATE_KEY.replace(/\\n/g, "\n"),
          clientEmail: FIREBASE_CLIENT_EMAIL,
        }),
      });
    }
    console.log("Firebase Admin initialized successfully");

    console.log("Getting user by email...");
    try {
      const auth = admin.auth();

      // Get user by email
      console.log("Attempting to fetch user...");
      const user = await auth.getUserByEmail(email);
      console.log("User found:", user.uid);

      console.log("Removing admin claims...");
      await auth.setCustomUserClaims(user.uid, {
        admin: false,
        premium: true, // Keeping premium status
      });
      console.log(`Successfully removed admin rights from ${email}`);

      // Get updated user to verify claims
      const updatedUser = await auth.getUser(user.uid);
      console.log("Updated user claims:", updatedUser.customClaims);

      process.exit(0);
    } catch (error: any) {
      if (error?.code === "auth/user-not-found") {
        console.error(
          `User ${email} not found in Firebase Auth. Please ensure the user exists.`
        );
      } else {
        console.error("Firebase Auth error:", error?.message, error?.code);
      }
      throw error;
    }
  } catch (error: any) {
    console.error("Error removing admin rights:", error?.message || error);
    if (error?.code) {
      console.error("Error code:", error.code);
    }
    if (error?.stack) {
      console.error("Stack trace:", error.stack);
    }
    process.exit(1);
  } finally {
    try {
      await admin.app().delete();
    } catch (error) {
      // Ignore cleanup errors
    }
  }
}

// Get email from command line argument
const email = process.argv[2];
if (!email) {
  console.error("Please provide an email address");
  console.error("Usage: bun run unassign-admin.ts <email>");
  process.exit(1);
}

// Execute and handle top-level async
unassignAdmin(email);
