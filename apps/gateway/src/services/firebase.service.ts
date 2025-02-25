import { Injectable, Inject, OnModuleInit } from '@nestjs/common';
import * as admin from 'firebase-admin';

@Injectable()
export class FirebaseService implements OnModuleInit {
  constructor(
    @Inject('FIREBASE_ADMIN') private readonly firebaseAdmin: admin.app.App
  ) {}

  onModuleInit() {
    // Validate required environment variables
    const requiredEnvVars = [
      'FIREBASE_PROJECT_ID',
      'FIREBASE_PRIVATE_KEY',
      'FIREBASE_CLIENT_EMAIL',
    ];

    const missingEnvVars = requiredEnvVars.filter(
      (envVar) => !process.env[envVar]
    );

    if (missingEnvVars.length > 0) {
      throw new Error(
        `Missing required Firebase environment variables: ${missingEnvVars.join(
          ', '
        )}`
      );
    }
  }

  // Common notification templates
  async sendTopicNotification(
    topic: string,
    title: string,
    body: string,
    data?: { [key: string]: string }
  ) {
    try {
      const message: admin.messaging.Message = {
        topic,
        notification: {
          title,
          body,
        },
        data,
      };
      return await this.firebaseAdmin.messaging().send(message);
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error occurred';
      throw new Error(`Failed to send topic notification: ${errorMessage}`);
    }
  }

  async sendMulticastNotification(
    tokens: string[],
    title: string,
    body: string,
    data?: { [key: string]: string }
  ) {
    try {
      const messages = tokens.map((token) => ({
        token,
        notification: {
          title,
          body,
        },
        data,
      }));
      return await this.firebaseAdmin.messaging().sendEach(messages);
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error occurred';
      throw new Error(`Failed to send multicast notification: ${errorMessage}`);
    }
  }

  // Firebase Cloud Messaging (FCM)
  async sendPushNotification(
    token: string,
    notification: admin.messaging.Notification,
    data?: { [key: string]: string }
  ) {
    try {
      const message: admin.messaging.Message = {
        token,
        notification,
        data,
      };
      return await this.firebaseAdmin.messaging().send(message);
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error occurred';
      throw new Error(`Failed to send push notification: ${errorMessage}`);
    }
  }

  // Firebase Authentication
  async verifyIdToken(idToken: string) {
    try {
      return await this.firebaseAdmin.auth().verifyIdToken(idToken);
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error occurred';
      throw new Error(`Failed to verify ID token: ${errorMessage}`);
    }
  }

  async getUser(uid: string) {
    try {
      return await this.firebaseAdmin.auth().getUser(uid);
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error occurred';
      throw new Error(`Failed to get user: ${errorMessage}`);
    }
  }

  async createCustomToken(uid: string, claims?: object) {
    try {
      return await this.firebaseAdmin.auth().createCustomToken(uid, claims);
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error occurred';
      throw new Error(`Failed to create custom token: ${errorMessage}`);
    }
  }

  // Firebase Analytics
  async logEvent(
    userId: string,
    eventName: string,
    eventParams?: { [key: string]: any }
  ) {
    try {
      // Note: Server-side Firebase Analytics requires Google Analytics 4
      // This is a placeholder for GA4 implementation
      // You'll need to use the Google Analytics Data API or Measurement Protocol
      console.log('Analytics event logged:', { userId, eventName, eventParams });
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error occurred';
      throw new Error(`Failed to log analytics event: ${errorMessage}`);
    }
  }

  // Firebase AdMob (Note: AdMob is primarily client-side)
  // Server can handle ad-related analytics and reporting
  async getAdPerformanceMetrics(adUnitId: string) {
    // This is a placeholder for AdMob reporting implementation
    // You'll need to use the AdMob API for actual implementation
    console.log('Getting ad performance metrics for:', adUnitId);
    return { adUnitId, impressions: 0, clicks: 0 };
  }
}
