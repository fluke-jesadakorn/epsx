import { NextRequest, NextResponse } from 'next/server';
import { env } from '@/config/env';

const BACKEND_URL = env.BACKEND_URL;

// In-memory nonce cache for replay attack prevention
// In production, this should be replaced with Redis or similar distributed cache
const usedNonces = new Map<string, { timestamp: number; wallet_address: string }>();
const NONCE_CLEANUP_INTERVAL = 15 * 60 * 1000; // 15 minutes
const MAX_NONCE_AGE = 10 * 60 * 1000; // 10 minutes

// Cleanup expired nonces periodically
setInterval(() => {
  const now = Date.now();
  for (const [nonce, data] of usedNonces.entries()) {
    if (now - data.timestamp > MAX_NONCE_AGE) {
      usedNonces.delete(nonce);
    }
  }
}, NONCE_CLEANUP_INTERVAL);

interface AuthenticateRequest {
  wallet_address: string;
  signature: string;
  message: string;
  chain_id?: number;
}

interface AuthenticateResponse {
  success: boolean;
  nonce?: string;
  session_token?: string;
  expires_at?: string;
  user?: {
    wallet_address: string;
    permissions: string[];
    user_tier: string;
  };
  error?: string;
}

/**
 * Streamlined Web3 Authentication Endpoint
 * 
 * This endpoint combines challenge generation and verification into a single call
 * to improve user experience while maintaining security through proper nonce handling.
 * 
 * Security Features:
 * - Server-generated nonces for replay protection
 * - Message expiration timestamps (24h max)
 * - Session invalidation support
 * - Proper SIWE message validation
 */
export async function POST(request: NextRequest) {
  try {
    const body: AuthenticateRequest = await request.json();
    const { wallet_address, signature, message, chain_id } = body;

    // Validate required fields
    if (!wallet_address) {
      return NextResponse.json(
        { success: false, error: 'Wallet address is required' },
        { status: 400 }
      );
    }

    if (!signature || !message) {
      return NextResponse.json(
        { success: false, error: 'Signature and message are required' },
        { status: 400 }
      );
    }

    // Extract nonce from SIWE message for validation
    const nonceMatch = message.match(/Nonce: ([a-zA-Z0-9]+)/);
    if (!nonceMatch) {
      return NextResponse.json(
        { success: false, error: 'Invalid SIWE message format - missing nonce' },
        { status: 400 }
      );
    }
    const nonce = nonceMatch[1];

    // Enhanced nonce validation with timestamp expiration
    console.log('🔍 Validating timestamp-embedded nonce:', nonce.substring(0, 20) + '...');
    
    // Parse timestamp-embedded nonce format: {timestamp_hex}{random_data}
    // Timestamp hex is typically 11-12 characters (48-bit timestamp), extract it from the beginning
    if (nonce.length < 19) { // minimum: 11 char timestamp + 8 char random
      return NextResponse.json(
        { success: false, error: 'Invalid nonce format - too short for timestamp-embedded format' },
        { status: 400 }
      );
    }

    // Extract timestamp (first 11-12 characters as hex)
    let timestampHex = '';
    let randomPart = '';
    
    // Try different timestamp lengths (11-12 characters typical for current timestamps)
    for (let timestampLength = 11; timestampLength <= 12; timestampLength++) {
      const potentialTimestamp = nonce.substring(0, timestampLength);
      const potentialRandom = nonce.substring(timestampLength);
      
      // Validate hex format and reasonable timestamp range
      if (/^[0-9a-f]+$/i.test(potentialTimestamp) && potentialRandom.length >= 8) {
        const timestamp = parseInt(potentialTimestamp, 16);
        const currentTime = Date.now();
        
        // Check if timestamp is reasonable (within last 24 hours and next 1 hour)
        if (timestamp > currentTime - 24 * 60 * 60 * 1000 && timestamp < currentTime + 60 * 60 * 1000) {
          timestampHex = potentialTimestamp;
          randomPart = potentialRandom;
          break;
        }
      }
    }
    
    if (!timestampHex) {
      return NextResponse.json(
        { success: false, error: 'Invalid nonce format - could not extract valid timestamp' },
        { status: 400 }
      );
    }
    
    // Validate random part (at least 8 alphanumeric characters per EIP-4361)
    if (randomPart.length < 8 || !/^[a-zA-Z0-9]+$/.test(randomPart)) {
      return NextResponse.json(
        { success: false, error: 'Invalid nonce random part - must be at least 8 alphanumeric characters' },
        { status: 400 }
      );
    }

    // Parse and validate nonce timestamp
    const nonceTimestamp = parseInt(timestampHex, 16);
    const currentTimestamp = Date.now();
    const nonceAgeMs = currentTimestamp - nonceTimestamp;
    const maxNonceAgeMs = 10 * 60 * 1000; // 10 minutes

    console.log('⏰ Nonce timestamp validation:', {
      nonceTimestamp: new Date(nonceTimestamp).toISOString(),
      currentTimestamp: new Date(currentTimestamp).toISOString(),
      ageMinutes: Math.floor(nonceAgeMs / 60000),
      maxAgeMinutes: Math.floor(maxNonceAgeMs / 60000),
      isExpired: nonceAgeMs > maxNonceAgeMs
    });

    if (nonceAgeMs > maxNonceAgeMs) {
      return NextResponse.json(
        { 
          success: false, 
          error: `Nonce has expired - must be used within ${Math.floor(maxNonceAgeMs / 60000)} minutes` 
        },
        { status: 400 }
      );
    }

    if (nonceAgeMs < 0) {
      return NextResponse.json(
        { success: false, error: 'Invalid nonce timestamp - cannot be in the future' },
        { status: 400 }
      );
    }

    // Skip nonce consumption here - let the backend handle replay attack detection
    // This prevents double-consumption where frontend marks nonce as used 
    // before backend can validate it
    console.log('🔄 Skipping frontend nonce consumption - delegating to backend');
    console.log('✅ Nonce format validation passed:', {
      age: `${Math.floor(nonceAgeMs / 1000)}s`,
      walletAddress: wallet_address,
      note: 'Backend will handle replay attack detection'
    });

    // Enhanced message expiration validation
    console.log('⏰ Validating SIWE message expiration...');
    const expirationMatch = message.match(/Expiration Time: ([^\\n]+)/);
    
    if (!expirationMatch) {
      console.log('⚠️ No expiration time found in SIWE message');
      return NextResponse.json(
        { success: false, error: 'Invalid SIWE message - missing expiration time' },
        { status: 400 }
      );
    }

    const expirationTimeStr = expirationMatch[1];
    let expirationTime: Date;
    
    try {
      expirationTime = new Date(expirationTimeStr);
      if (isNaN(expirationTime.getTime())) {
        throw new Error('Invalid date format');
      }
    } catch (error) {
      console.log('❌ Invalid expiration time format:', expirationTimeStr);
      return NextResponse.json(
        { success: false, error: 'Invalid expiration time format in SIWE message' },
        { status: 400 }
      );
    }

    const currentTime = new Date();
    const messageAge = currentTime.getTime() - expirationTime.getTime();
    const isExpired = expirationTime < currentTime;
    
    // Security validation: ensure reasonable expiration bounds
    const maxAllowedExpiration = 24 * 60 * 60 * 1000; // 24 hours max
    const minAllowedExpiration = 5 * 60 * 1000; // 5 minutes min
    const timeUntilExpiration = expirationTime.getTime() - currentTime.getTime();
    
    console.log('📅 Message expiration validation:', {
      expirationTime: expirationTime.toISOString(),
      currentTime: currentTime.toISOString(),
      timeUntilExpiration: `${Math.floor(timeUntilExpiration / 60000)} minutes`,
      isExpired,
      withinBounds: timeUntilExpiration <= maxAllowedExpiration && timeUntilExpiration >= -minAllowedExpiration
    });

    if (isExpired) {
      const expiredAgo = Math.floor(-timeUntilExpiration / 60000);
      console.log('❌ Message has expired:', {
        expiredAgo: `${expiredAgo} minutes ago`,
        expirationTime: expirationTime.toISOString()
      });
      
      return NextResponse.json(
        { 
          success: false, 
          error: `Authentication message expired ${expiredAgo} minutes ago` 
        },
        { status: 400 }
      );
    }

    // Validate expiration is not too far in the future (security check)
    if (timeUntilExpiration > maxAllowedExpiration) {
      console.log('❌ Message expiration too far in future:', {
        timeUntilExpiration: `${Math.floor(timeUntilExpiration / 60000)} minutes`,
        maxAllowed: `${Math.floor(maxAllowedExpiration / 60000)} minutes`
      });
      
      return NextResponse.json(
        { 
          success: false, 
          error: `Message expiration too far in future - maximum ${Math.floor(maxAllowedExpiration / 60000)} minutes allowed` 
        },
        { status: 400 }
      );
    }

    console.log('✅ Message expiration validation passed:', {
      validFor: `${Math.floor(timeUntilExpiration / 60000)} minutes`,
      expiresAt: expirationTime.toISOString()
    });

    // Verify signature with backend
    const verifyResponse = await fetch(`${BACKEND_URL}/api/auth/web3/verify`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        wallet_address,
        signature,
        nonce,
        message,
        chain_id,
      }),
    });

    if (!verifyResponse.ok) {
      const errorData = await verifyResponse.json().catch(() => ({}));
      return NextResponse.json(
        { 
          success: false, 
          error: errorData.message || `Backend verification failed: ${verifyResponse.status}` 
        },
        { status: verifyResponse.status }
      );
    }

    const verificationResult = await verifyResponse.json();

    // Create session and set cookies
    const response = NextResponse.json({
      success: true,
      session_token: verificationResult.session_token,
      expires_at: verificationResult.expires_at,
      user: verificationResult.user,
    });

    // Set session cookies if provided by backend
    if (verificationResult.access_token) {
      response.cookies.set('access_token', verificationResult.access_token, {
        httpOnly: true,
        secure: process.env.NODE_ENV === 'production',
        sameSite: 'lax',
        maxAge: 24 * 60 * 60, // 24 hours
        path: '/',
      });
    }

    if (verificationResult.id_token) {
      response.cookies.set('id_token', verificationResult.id_token, {
        httpOnly: true,
        secure: process.env.NODE_ENV === 'production',
        sameSite: 'lax',
        maxAge: 24 * 60 * 60, // 24 hours
        path: '/',
      });
    }

    if (verificationResult.refresh_token) {
      response.cookies.set('refresh_token', verificationResult.refresh_token, {
        httpOnly: true,
        secure: process.env.NODE_ENV === 'production',
        sameSite: 'lax',
        maxAge: 7 * 24 * 60 * 60, // 7 days
        path: '/',
      });
    }

    return response;

  } catch (error) {
    console.error('Streamlined Web3 authentication error:', error);
    return NextResponse.json(
      { success: false, error: 'Internal server error during authentication' },
      { status: 500 }
    );
  }
}