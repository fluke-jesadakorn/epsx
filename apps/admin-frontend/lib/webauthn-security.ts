'use client';

// WebAuthn/FIDO2 Hardware Security Key Implementation
// The most secure authentication method with hardware-based cryptographic verification

/**
 * WebAuthn credential creation options
 */
export interface WebAuthnCredentialOptions {
  userId: string;
  userEmail: string;
  userName: string;
  userDisplayName: string;
  challenge: string;
  rpId: string;
  rpName: string;
  timeout: number;
  attestation: AttestationConveyancePreference;
  userVerification: UserVerificationRequirement;
  authenticatorSelection: AuthenticatorSelectionCriteria;
  excludeCredentials?: PublicKeyCredentialDescriptor[];
}

/**
 * WebAuthn assertion options
 */
export interface WebAuthnAssertionOptions {
  challenge: string;
  rpId: string;
  timeout: number;
  userVerification: UserVerificationRequirement;
  allowCredentials?: PublicKeyCredentialDescriptor[];
}

/**
 * Biometric verification result
 */
export interface BiometricVerificationResult {
  isSupported: boolean;
  isAvailable: boolean;
  verificationSuccessful: boolean;
  biometricType: 'fingerprint' | 'faceId' | 'voiceId' | 'unknown';
  confidenceScore: number;
  errorMessage?: string;
}

/**
 * Behavioral biometrics data
 */
export interface BehavioralBiometricsData {
  keystrokeDynamics: KeystrokeDynamics;
  mouseMovement: MouseMovement;
  touchPatterns: TouchPattern[];
  deviceOrientation: DeviceOrientation;
  networkLatency: number;
  timeZoneOffset: number;
  sessionId: string;
  timestamp: Date;
}

export interface KeystrokeDynamics {
  dwellTimes: number[]; // Time each key is held down
  flightTimes: number[]; // Time between key presses
  typingRhythm: number;
  averageSpeed: number; // Words per minute
  pressureLevels: number[]; // Key pressure if available
}

export interface MouseMovement {
  movementSpeed: number;
  clickPressure: number;
  trajectoryPattern: { x: number; y: number; timestamp: number }[];
  scrollSpeed: number;
  hoverTime: number;
}

export interface TouchPattern {
  touchPressure: number;
  touchSize: number;
  swipeVelocity: number;
  multiTouchCoordination: number;
  gestureComplexity: number;
}

export interface DeviceOrientation {
  alpha: number; // Z-axis rotation
  beta: number;  // X-axis rotation  
  gamma: number; // Y-axis rotation
  accelerometerData: { x: number; y: number; z: number };
  gyroscopeData: { x: number; y: number; z: number };
}

/**
 * Advanced WebAuthn Security Manager
 * Implements hardware security keys, biometric authentication, and behavioral analysis
 */
export class WebAuthnSecurityManager {
  private behavioralData: BehavioralBiometricsData | null = null;
  private keystrokeBuffer: { key: string; timestamp: number; type: 'down' | 'up' }[] = [];
  private mouseTrackingBuffer: { x: number; y: number; timestamp: number }[] = [];
  private touchTrackingBuffer: TouchPattern[] = [];
  private deviceOrientationData: DeviceOrientation | null = null;
  
  private isTracking = false;
  private trackingStartTime: number = 0;

  constructor(private baseApiUrl: string) {
    this.initializeBehavioralTracking();
    this.initializeDeviceOrientationTracking();
  }

  /**
   * Check if WebAuthn is supported in the current environment
   */
  public isWebAuthnSupported(): boolean {
    return !!(window.PublicKeyCredential && 
              navigator.credentials && 
              navigator.credentials.create &&
              navigator.credentials.get);
  }

  /**
   * Check if biometric authentication is available
   */
  public async isBiometricAvailable(): Promise<BiometricVerificationResult> {
    const result: BiometricVerificationResult = {
      isSupported: false,
      isAvailable: false,
      verificationSuccessful: false,
      biometricType: 'unknown',
      confidenceScore: 0,
    };

    if (!this.isWebAuthnSupported()) {
      result.errorMessage = 'WebAuthn not supported';
      return result;
    }

    try {
      // Check if platform authenticator (biometric) is available
      const available = await PublicKeyCredential.isUserVerifyingPlatformAuthenticatorAvailable();
      result.isSupported = true;
      result.isAvailable = available;

      if (available) {
        // Detect biometric type based on platform
        if (navigator.userAgent.includes('iPhone') || navigator.userAgent.includes('iPad')) {
          result.biometricType = 'faceId';
        } else if (navigator.userAgent.includes('Android')) {
          result.biometricType = 'fingerprint';
        } else if (navigator.userAgent.includes('Windows')) {
          result.biometricType = 'fingerprint'; // Windows Hello
        }
      }

      return result;
    } catch (error) {
      result.errorMessage = error instanceof Error ? error.message : 'Biometric check failed';
      return result;
    }
  }

  /**
   * Register a new WebAuthn credential (hardware security key)
   */
  public async registerWebAuthnCredential(
    userId: string,
    userEmail: string,
    sessionToken: string
  ): Promise<{ success: boolean; credentialId?: string; error?: string }> {
    if (!this.isWebAuthnSupported()) {
      return { success: false, error: 'WebAuthn not supported' };
    }

    try {
      // Get registration options from backend
      const optionsResponse = await fetch(`${this.baseApiUrl}/api/v1/webauthn/register/begin`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${sessionToken}`,
        },
        body: JSON.stringify({
          user_id: userId,
          user_email: userEmail,
          behavioral_data: this.behavioralData,
        }),
      });

      if (!optionsResponse.ok) {
        throw new Error('Failed to get WebAuthn registration options');
      }

      const options: WebAuthnCredentialOptions = await optionsResponse.json();

      // Create credential using WebAuthn API
      const credentialCreationOptions: CredentialCreationOptions = {
        publicKey: {
          challenge: this.base64urlDecode(options.challenge),
          rp: {
            name: options.rpName,
            id: options.rpId,
          },
          user: {
            id: this.stringToArrayBuffer(options.userId),
            name: options.userEmail,
            displayName: options.userDisplayName,
          },
          pubKeyCredParams: [
            { alg: -7, type: 'public-key' },  // ES256
            { alg: -257, type: 'public-key' }, // RS256
          ],
          authenticatorSelection: {
            authenticatorAttachment: 'cross-platform', // Hardware security key
            userVerification: options.userVerification,
            requireResidentKey: false,
          },
          timeout: options.timeout,
          attestation: options.attestation,
          excludeCredentials: options.excludeCredentials?.map(cred => ({
            ...cred,
            id: this.base64urlDecode(cred.id as string),
          })),
        },
      };

      const credential = await navigator.credentials.create(credentialCreationOptions) as PublicKeyCredential;

      if (!credential) {
        throw new Error('Failed to create WebAuthn credential');
      }

      const response = credential.response as AuthenticatorAttestationResponse;

      // Complete registration with backend
      const finishResponse = await fetch(`${this.baseApiUrl}/api/v1/webauthn/register/finish`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${sessionToken}`,
        },
        body: JSON.stringify({
          user_id: userId,
          credential_id: this.arrayBufferToBase64url(credential.rawId),
          client_data_json: this.arrayBufferToBase64url(response.clientDataJSON),
          attestation_object: this.arrayBufferToBase64url(response.attestationObject),
          behavioral_data: this.behavioralData,
        }),
      });

      if (!finishResponse.ok) {
        throw new Error('Failed to complete WebAuthn registration');
      }

      const result = await finishResponse.json();

      return {
        success: true,
        credentialId: result.credential_id,
      };

    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : 'WebAuthn registration failed',
      };
    }
  }

  /**
   * Authenticate using WebAuthn credential
   */
  public async authenticateWebAuthn(
    userEmail: string,
    sessionToken?: string
  ): Promise<{ success: boolean; authToken?: string; authLevel?: number; error?: string }> {
    if (!this.isWebAuthnSupported()) {
      return { success: false, error: 'WebAuthn not supported' };
    }

    try {
      // Get authentication options from backend
      const optionsResponse = await fetch(`${this.baseApiUrl}/api/v1/webauthn/authenticate/begin`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          ...(sessionToken && { 'Authorization': `Bearer ${sessionToken}` }),
        },
        body: JSON.stringify({
          user_email: userEmail,
          behavioral_data: this.behavioralData,
        }),
      });

      if (!optionsResponse.ok) {
        throw new Error('Failed to get WebAuthn authentication options');
      }

      const options: WebAuthnAssertionOptions = await optionsResponse.json();

      // Create assertion using WebAuthn API
      const credentialRequestOptions: CredentialRequestOptions = {
        publicKey: {
          challenge: this.base64urlDecode(options.challenge),
          rpId: options.rpId,
          timeout: options.timeout,
          userVerification: options.userVerification,
          allowCredentials: options.allowCredentials?.map(cred => ({
            ...cred,
            id: this.base64urlDecode(cred.id as string),
          })),
        },
      };

      const assertion = await navigator.credentials.get(credentialRequestOptions) as PublicKeyCredential;

      if (!assertion) {
        throw new Error('Failed to get WebAuthn assertion');
      }

      const response = assertion.response as AuthenticatorAssertionResponse;

      // Complete authentication with backend
      const finishResponse = await fetch(`${this.baseApiUrl}/api/v1/webauthn/authenticate/finish`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          user_email: userEmail,
          credential_id: this.arrayBufferToBase64url(assertion.rawId),
          client_data_json: this.arrayBufferToBase64url(response.clientDataJSON),
          authenticator_data: this.arrayBufferToBase64url(response.authenticatorData),
          signature: this.arrayBufferToBase64url(response.signature),
          user_handle: response.userHandle ? this.arrayBufferToBase64url(response.userHandle) : null,
          behavioral_data: this.behavioralData,
        }),
      });

      if (!finishResponse.ok) {
        throw new Error('Failed to complete WebAuthn authentication');
      }

      const result = await finishResponse.json();

      return {
        success: true,
        authToken: result.access_token,
        authLevel: result.auth_level,
      };

    } catch (error) {
      return {
        success: false,
        error: error instanceof Error ? error.message : 'WebAuthn authentication failed',
      };
    }
  }

  /**
   * Start behavioral biometrics tracking
   */
  public startBehavioralTracking(): void {
    if (this.isTracking) return;

    this.isTracking = true;
    this.trackingStartTime = Date.now();
    this.keystrokeBuffer = [];
    this.mouseTrackingBuffer = [];
    this.touchTrackingBuffer = [];

    console.log('Started behavioral biometrics tracking');
  }

  /**
   * Stop behavioral biometrics tracking and generate data
   */
  public stopBehavioralTracking(): BehavioralBiometricsData | null {
    if (!this.isTracking) return null;

    this.isTracking = false;

    const sessionDuration = Date.now() - this.trackingStartTime;

    this.behavioralData = {
      keystrokeDynamics: this.analyzeKeystrokeDynamics(),
      mouseMovement: this.analyzeMouseMovement(),
      touchPatterns: this.touchTrackingBuffer,
      deviceOrientation: this.deviceOrientationData || {
        alpha: 0, beta: 0, gamma: 0,
        accelerometerData: { x: 0, y: 0, z: 0 },
        gyroscopeData: { x: 0, y: 0, z: 0 },
      },
      networkLatency: this.measureNetworkLatency(),
      timeZoneOffset: new Date().getTimezoneOffset(),
      sessionId: crypto.randomUUID(),
      timestamp: new Date(),
    };

    console.log('Stopped behavioral tracking, session duration:', sessionDuration, 'ms');
    return this.behavioralData;
  }

  /**
   * Verify user identity using behavioral biometrics
   */
  public async verifyBehavioralBiometrics(
    userId: string,
    sessionToken: string
  ): Promise<{ verified: boolean; confidenceScore: number; riskLevel: 'low' | 'medium' | 'high' }> {
    if (!this.behavioralData) {
      return { verified: false, confidenceScore: 0, riskLevel: 'high' };
    }

    try {
      const response = await fetch(`${this.baseApiUrl}/api/v1/biometrics/verify`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${sessionToken}`,
        },
        body: JSON.stringify({
          user_id: userId,
          behavioral_data: this.behavioralData,
        }),
      });

      if (!response.ok) {
        throw new Error('Behavioral biometrics verification failed');
      }

      const result = await response.json();

      return {
        verified: result.verification_successful,
        confidenceScore: result.confidence_score,
        riskLevel: result.risk_level,
      };

    } catch (error) {
      console.error('Behavioral biometrics verification error:', error);
      return { verified: false, confidenceScore: 0, riskLevel: 'high' };
    }
  }

  /**
   * Initialize behavioral tracking event listeners
   */
  private initializeBehavioralTracking(): void {
    // Keystroke dynamics tracking
    document.addEventListener('keydown', (event) => {
      if (this.isTracking) {
        this.keystrokeBuffer.push({
          key: event.key,
          timestamp: event.timeStamp,
          type: 'down',
        });
      }
    });

    document.addEventListener('keyup', (event) => {
      if (this.isTracking) {
        this.keystrokeBuffer.push({
          key: event.key,
          timestamp: event.timeStamp,
          type: 'up',
        });
      }
    });

    // Mouse movement tracking
    document.addEventListener('mousemove', (event) => {
      if (this.isTracking && this.mouseTrackingBuffer.length < 1000) {
        this.mouseTrackingBuffer.push({
          x: event.clientX,
          y: event.clientY,
          timestamp: event.timeStamp,
        });
      }
    });

    // Touch pattern tracking
    document.addEventListener('touchstart', (event) => {
      if (this.isTracking) {
        const touch = event.touches[0];
        this.touchTrackingBuffer.push({
          touchPressure: (touch as any).force || 0,
          touchSize: (touch as any).radiusX || 0,
          swipeVelocity: 0,
          multiTouchCoordination: event.touches.length > 1 ? 1 : 0,
          gestureComplexity: event.touches.length,
        });
      }
    });
  }

  /**
   * Initialize device orientation tracking
   */
  private initializeDeviceOrientationTracking(): void {
    if (window.DeviceOrientationEvent) {
      window.addEventListener('deviceorientation', (event) => {
        this.deviceOrientationData = {
          alpha: event.alpha || 0,
          beta: event.beta || 0,
          gamma: event.gamma || 0,
          accelerometerData: { x: 0, y: 0, z: 0 }, // Would need DeviceMotionEvent
          gyroscopeData: { x: 0, y: 0, z: 0 },
        };
      });
    }

    if (window.DeviceMotionEvent) {
      window.addEventListener('devicemotion', (event) => {
        if (this.deviceOrientationData) {
          this.deviceOrientationData.accelerometerData = {
            x: event.acceleration?.x || 0,
            y: event.acceleration?.y || 0,
            z: event.acceleration?.z || 0,
          };
          this.deviceOrientationData.gyroscopeData = {
            x: event.rotationRate?.alpha || 0,
            y: event.rotationRate?.beta || 0,
            z: event.rotationRate?.gamma || 0,
          };
        }
      });
    }
  }

  /**
   * Analyze keystroke dynamics patterns
   */
  private analyzeKeystrokeDynamics(): KeystrokeDynamics {
    const dwellTimes: number[] = [];
    const flightTimes: number[] = [];
    
    let totalChars = 0;
    let totalTime = 0;

    for (let i = 0; i < this.keystrokeBuffer.length - 1; i++) {
      const current = this.keystrokeBuffer[i];
      const next = this.keystrokeBuffer[i + 1];

      if (current.type === 'down' && next.type === 'up' && current.key === next.key) {
        // Dwell time (key held down)
        dwellTimes.push(next.timestamp - current.timestamp);
      }

      if (current.type === 'up' && next.type === 'down') {
        // Flight time (between key presses)
        flightTimes.push(next.timestamp - current.timestamp);
        totalChars++;
        totalTime += next.timestamp - current.timestamp;
      }
    }

    const averageSpeed = totalTime > 0 ? (totalChars / totalTime) * 60000 : 0; // WPM
    const typingRhythm = this.calculateTypingRhythm(flightTimes);

    return {
      dwellTimes,
      flightTimes,
      typingRhythm,
      averageSpeed,
      pressureLevels: [], // Would need hardware support
    };
  }

  /**
   * Analyze mouse movement patterns
   */
  private analyzeMouseMovement(): MouseMovement {
    if (this.mouseTrackingBuffer.length < 2) {
      return {
        movementSpeed: 0,
        clickPressure: 0,
        trajectoryPattern: [],
        scrollSpeed: 0,
        hoverTime: 0,
      };
    }

    let totalDistance = 0;
    let totalTime = 0;

    for (let i = 1; i < this.mouseTrackingBuffer.length; i++) {
      const prev = this.mouseTrackingBuffer[i - 1];
      const curr = this.mouseTrackingBuffer[i];

      const distance = Math.sqrt(
        Math.pow(curr.x - prev.x, 2) + Math.pow(curr.y - prev.y, 2)
      );
      const timeDiff = curr.timestamp - prev.timestamp;

      totalDistance += distance;
      totalTime += timeDiff;
    }

    const averageSpeed = totalTime > 0 ? totalDistance / totalTime : 0;

    return {
      movementSpeed: averageSpeed,
      clickPressure: 0, // Would need hardware support
      trajectoryPattern: this.mouseTrackingBuffer,
      scrollSpeed: 0, // Would need scroll event tracking
      hoverTime: 0, // Would need hover time calculation
    };
  }

  /**
   * Calculate typing rhythm variance
   */
  private calculateTypingRhythm(flightTimes: number[]): number {
    if (flightTimes.length === 0) return 0;

    const mean = flightTimes.reduce((sum, time) => sum + time, 0) / flightTimes.length;
    const variance = flightTimes.reduce((sum, time) => sum + Math.pow(time - mean, 2), 0) / flightTimes.length;
    
    return Math.sqrt(variance);
  }

  /**
   * Measure network latency
   */
  private measureNetworkLatency(): number {
    // This would typically involve a ping to the server
    // For now, return a mock value
    return Math.random() * 100 + 50; // 50-150ms mock latency
  }

  /**
   * Utility functions for WebAuthn data conversion
   */
  private arrayBufferToBase64url(buffer: ArrayBuffer): string {
    const bytes = new Uint8Array(buffer);
    let binary = '';
    for (let i = 0; i < bytes.byteLength; i++) {
      binary += String.fromCharCode(bytes[i]);
    }
    return btoa(binary)
      .replace(/\+/g, '-')
      .replace(/\//g, '_')
      .replace(/=/g, '');
  }

  private base64urlDecode(input: string): ArrayBuffer {
    input = input.replace(/-/g, '+').replace(/_/g, '/');
    while (input.length % 4) {
      input += '=';
    }
    const binary = atob(input);
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i++) {
      bytes[i] = binary.charCodeAt(i);
    }
    return bytes.buffer;
  }

  private stringToArrayBuffer(str: string): ArrayBuffer {
    const encoder = new TextEncoder();
    return encoder.encode(str);
  }
}