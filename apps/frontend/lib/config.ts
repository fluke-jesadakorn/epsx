// Server-side configuration utility
export const config = {
  // Backend API URL - only available server-side
  getBackendUrl(): string {
    const url = process.env.API_URL || process.env.BACKEND_URL || 'http://localhost:8080';
    return url;
  },

  // Site URL - server-side only
  getSiteUrl(): string {
    const url = process.env.SITE_URL || process.env.NEXTAUTH_URL || 'http://localhost:3000';
    return url;
  },

  // Client-side should use relative URLs or API routes
  getApiUrl(): string {
    // For client-side, always use relative paths or Next.js API routes
    return '/api';
  },

  // Check if we're on the server side
  isServer(): boolean {
    return typeof window === 'undefined';
  },

  // Get MusePay configuration (server-side only)
  getMusePayConfig(): {
    partnerId: string;
    privateKey: string;
    publicKey: string;
    apiUrl: string;
    notifyUrl: string;
  } {
    if (!this.isServer()) {
      throw new Error('MusePay config can only be accessed server-side');
    }
    return {
      partnerId: process.env.MUSEPAY_PARTNER_ID || '',
      privateKey: process.env.MUSEPAY_PRIVATE_KEY || '',
      publicKey: process.env.MUSEPAY_PUBLIC_KEY || '',
      apiUrl: process.env.MUSEPAY_API_URL || '',
      notifyUrl: process.env.MUSEPAY_NOTIFY_URL || '',
    };
  },

  // Get Firebase configuration (server-side only for security)
  getFirebaseConfig() {
    if (!this.isServer()) {
      throw new Error('Firebase config can only be accessed server-side');
    }
    return {
      // Server-side admin config
      type: process.env.FIREBASE_TYPE,
      projectId: process.env.FIREBASE_PROJECT_ID,
      privateKeyId: process.env.FIREBASE_PRIVATE_KEY_ID,
      privateKey: process.env.FIREBASE_PRIVATE_KEY,
      clientEmail: process.env.FIREBASE_CLIENT_EMAIL,
      clientId: process.env.FIREBASE_CLIENT_ID,
      authUri: process.env.FIREBASE_AUTH_URI,
      tokenUri: process.env.FIREBASE_TOKEN_URI,
      authProviderCertUrl: process.env.FIREBASE_AUTH_PROVIDER_CERT_URL,
      clientCertUrl: process.env.FIREBASE_CLIENT_CERT_URL,
      universeDomain: process.env.FIREBASE_UNIVERSE_DOMAIN,
    };
  }
};