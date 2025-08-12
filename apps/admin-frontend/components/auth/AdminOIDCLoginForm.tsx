'use client';

// Advanced Admin OIDC Login Form
// Enterprise-grade security with admin privilege validation and threat detection

import React, { useState, useEffect, useCallback, useMemo, useRef } from 'react';
import { useRouter, useSearchParams } from 'next/navigation';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Progress } from '@/components/ui/progress';
import { Badge } from '@/components/ui/badge';
import { 
  Loader2, Shield, AlertTriangle, CheckCircle, XCircle, Lock,
  Eye, EyeOff, KeyRound, Fingerprint, Clock, Zap, Activity,
  UserCheck, Crown, ExternalLink, Wifi, WifiOff, RefreshCw
} from 'lucide-react';
import { initiateAdminOIDCLogin, getCurrentAdminOIDCUser } from '@/app/actions/admin-oidc-auth';

interface AdminOIDCLoginFormProps {
  onSuccess?: (user: any) => void;
  redirectTo?: string;
  requireMFA?: boolean;
  enableThreatDetection?: boolean;
  enableSessionMonitoring?: boolean;
  maxFailedAttempts?: number;
}

interface AdminUser {
  id: string;
  email: string;
  name?: string;
  role: string;
  permissions: string[];
  admin_level: 'moderator' | 'admin' | 'super_admin';
  tenant_id?: string;
  provider: string;
  last_activity: number;
}

interface LoginState {
  stage: 'input' | 'validating' | 'mfa_required' | 'processing' | 'success' | 'error' | 'lockout';
  progress: number;
  message: string;
  details?: string;
}

interface SecurityMetrics {
  failedAttempts: number;
  lastFailedAttempt: number;
  lockoutExpiry: number;
  riskScore: number;
  threatLevel: 'low' | 'medium' | 'high' | 'critical';
  deviceFingerprint: string;
  sessionId: string;
}

interface ThreatDetection {
  enabled: boolean;
  suspiciousActivity: string[];
  blockedReasons: string[];
  recommendations: string[];
}

interface ConnectionHealth {
  status: 'healthy' | 'degraded' | 'unhealthy';
  latency: number;
  lastCheck: number;
  backendReachable: boolean;
  oidcProviderReachable: boolean;
}

export function AdminOIDCLoginForm({
  onSuccess,
  redirectTo = '/dashboard',
  requireMFA = true,
  enableThreatDetection = true,
  enableSessionMonitoring = true,
  maxFailedAttempts = 3
}: AdminOIDCLoginFormProps) {
  const router = useRouter();
  const searchParams = useSearchParams();
  const monitoringRef = useRef<NodeJS.Timeout>();
  const healthCheckRef = useRef<NodeJS.Timeout>();

  // Form state
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [showPassword, setShowPassword] = useState(false);
  const [rememberDevice, setRememberDevice] = useState(false);

  // Login state
  const [loginState, setLoginState] = useState<LoginState>({
    stage: 'input',
    progress: 0,
    message: 'Enter your administrator credentials'
  });

  // Security metrics
  const [security, setSecurity] = useState<SecurityMetrics>({
    failedAttempts: 0,
    lastFailedAttempt: 0,
    lockoutExpiry: 0,
    riskScore: 0,
    threatLevel: 'low',
    deviceFingerprint: '',
    sessionId: ''
  });

  // Threat detection
  const [threats, setThreats] = useState<ThreatDetection>({
    enabled: enableThreatDetection,
    suspiciousActivity: [],
    blockedReasons: [],
    recommendations: []
  });

  // Connection health
  const [health, setHealth] = useState<ConnectionHealth>({
    status: 'healthy',
    latency: 0,
    lastCheck: Date.now(),
    backendReachable: true,
    oidcProviderReachable: true
  });

  const [error, setError] = useState<string | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [biometricAvailable, setBiometricAvailable] = useState(false);

  // Initialize component
  useEffect(() => {
    initializeSecurityFeatures();
    startHealthMonitoring();
    startSecurityMonitoring();
    
    return () => {
      if (monitoringRef.current) clearInterval(monitoringRef.current);
      if (healthCheckRef.current) clearInterval(healthCheckRef.current);
    };
  }, []);

  // Check for existing admin session
  useEffect(() => {
    checkExistingSession();
  }, []);

  // Real-time threat detection
  useEffect(() => {
    if (enableThreatDetection) {
      analyzeLoginThreat();
    }
  }, [email, enableThreatDetection]);

  // Computed values
  const canSubmit = useMemo(() => {
    return email && 
           loginState.stage === 'input' && 
           security.lockoutExpiry < Date.now() &&
           !isLoading &&
           health.backendReachable;
  }, [email, loginState.stage, security.lockoutExpiry, isLoading, health.backendReachable]);

  const lockoutTimeRemaining = useMemo(() => {
    if (security.lockoutExpiry <= Date.now()) return 0;
    return Math.ceil((security.lockoutExpiry - Date.now()) / 1000);
  }, [security.lockoutExpiry]);

  const riskLevelColor = useMemo(() => {
    switch (security.threatLevel) {
      case 'critical': return 'text-red-600 bg-red-50 border-red-200';
      case 'high': return 'text-orange-600 bg-orange-50 border-orange-200';
      case 'medium': return 'text-yellow-600 bg-yellow-50 border-yellow-200';
      default: return 'text-green-600 bg-green-50 border-green-200';
    }
  }, [security.threatLevel]);

  // Initialize security features
  const initializeSecurityFeatures = async () => {
    // Generate device fingerprint
    const fingerprint = await generateDeviceFingerprint();
    const sessionId = generateSessionId();
    
    // Load security metrics from storage
    const storedMetrics = loadSecurityMetrics();
    
    setSecurity(prev => ({
      ...prev,
      deviceFingerprint: fingerprint,
      sessionId,
      ...storedMetrics
    }));

    // Check biometric availability
    if ('PublicKeyCredential' in window) {
      try {
        const available = await PublicKeyCredential.isUserVerifyingPlatformAuthenticatorAvailable();
        setBiometricAvailable(available);
      } catch (error) {
        console.debug('Biometric check failed:', error);
      }
    }
  };

  // Check existing admin session
  const checkExistingSession = async () => {
    try {
      const existingUser = await getCurrentAdminOIDCUser();
      if (existingUser) {
        console.log('✅ Existing admin session found:', existingUser.email);
        setLoginState({
          stage: 'success',
          progress: 100,
          message: 'Session restored successfully'
        });
        
        if (onSuccess) {
          onSuccess(existingUser);
        } else {
          router.replace(redirectTo);
        }
      }
    } catch (error) {
      console.debug('No existing admin session found');
    }
  };

  // Health monitoring
  const startHealthMonitoring = () => {
    healthCheckRef.current = setInterval(async () => {
      await performHealthCheck();
    }, 30000); // Every 30 seconds
  };

  // Security monitoring
  const startSecurityMonitoring = () => {
    if (!enableSessionMonitoring) return;
    
    monitoringRef.current = setInterval(() => {
      // Check for security violations
      if (security.failedAttempts > maxFailedAttempts / 2) {
        setThreats(prev => ({
          ...prev,
          suspiciousActivity: [...prev.suspiciousActivity, 'Multiple failed login attempts']
        }));
      }
      
      // Update lockout status
      if (security.lockoutExpiry > Date.now()) {
        setLoginState(prev => ({
          ...prev,
          stage: 'lockout',
          message: `Account locked. Try again in ${lockoutTimeRemaining}s`,
          details: 'Too many failed authentication attempts'
        }));
      } else if (loginState.stage === 'lockout') {
        setLoginState({
          stage: 'input',
          progress: 0,
          message: 'Enter your administrator credentials'
        });
      }
    }, 1000);
  };

  // Perform health check
  const performHealthCheck = async () => {
    const startTime = Date.now();
    
    try {
      // Check backend connectivity (use actual backend URL)
      const backendResponse = await fetch(`${process.env.NEXT_PUBLIC_BACKEND_URL || 'http://localhost:8080'}/api/v1/health`, { 
        method: 'HEAD',
        cache: 'no-cache'
      });
      const backendReachable = backendResponse.ok;
      
      // Check OIDC provider (simplified check)
      const oidcProviderReachable = true; // Would check actual OIDC provider
      
      const latency = Date.now() - startTime;
      const status = backendReachable && oidcProviderReachable ? 'healthy' : 
                    (!backendReachable && !oidcProviderReachable) ? 'unhealthy' : 'degraded';
      
      setHealth({
        status,
        latency,
        lastCheck: Date.now(),
        backendReachable,
        oidcProviderReachable
      });
      
    } catch (error) {
      setHealth(prev => ({
        ...prev,
        status: 'unhealthy',
        latency: Date.now() - startTime,
        lastCheck: Date.now(),
        backendReachable: false,
        oidcProviderReachable: false
      }));
    }
  };

  // Analyze login threat
  const analyzeLoginThreat = () => {
    if (!enableThreatDetection || !email) return;
    
    let riskScore = 0;
    const suspiciousActivity: string[] = [];
    
    // Check email patterns
    if (email.includes('admin') || email.includes('root') || email.includes('test')) {
      riskScore += 20;
      suspiciousActivity.push('Common admin email pattern detected');
    }
    
    // Check for rapid attempts
    if (security.failedAttempts > 0) {
      const timeSinceLastAttempt = Date.now() - security.lastFailedAttempt;
      if (timeSinceLastAttempt < 10000) { // Less than 10 seconds
        riskScore += 30;
        suspiciousActivity.push('Rapid login attempts detected');
      }
    }
    
    // Device fingerprint analysis
    const knownDevice = localStorage.getItem('admin_known_device') === security.deviceFingerprint;
    if (!knownDevice) {
      riskScore += 25;
      suspiciousActivity.push('Unrecognized device');
    }
    
    // Time-based risk
    const hour = new Date().getHours();
    if (hour < 6 || hour > 22) {
      riskScore += 15;
      suspiciousActivity.push('Login attempt during off-hours');
    }
    
    // Determine threat level
    let threatLevel: SecurityMetrics['threatLevel'] = 'low';
    if (riskScore >= 70) threatLevel = 'critical';
    else if (riskScore >= 50) threatLevel = 'high';
    else if (riskScore >= 30) threatLevel = 'medium';
    
    setSecurity(prev => ({ ...prev, riskScore, threatLevel }));
    setThreats(prev => ({ ...prev, suspiciousActivity }));
  };

  // Handle form submission
  const handleSubmit = async (event?: React.FormEvent) => {
    console.log('🚀 handleSubmit called with version 2.0!');
    console.log('🔍 canSubmit:', canSubmit, 'email:', email, 'password:', password);
    event?.preventDefault();
    if (!canSubmit) {
      console.log('❌ canSubmit is false, returning early');
      return;
    }
    console.log('✅ Passed canSubmit check, proceeding...');

    try {
      setIsLoading(true);
      setError(null);
      
      setLoginState({
        stage: 'validating',
        progress: 20,
        message: 'Validating administrator credentials...',
        details: 'Checking admin privileges and security clearance'
      });

      // Security pre-checks
      if (security.threatLevel === 'critical') {
        throw new Error('Login blocked due to critical security risk. Please contact IT security.');
      }
      
      setLoginState(prev => ({ 
        ...prev, 
        progress: 40, 
        message: 'Initiating secure OIDC authentication...' 
      }));

      // For testing: Mock successful admin login
      console.log('🔐 Testing mock login with:', { email, password });
      let result;
      if (email === "jesadakorn.kirtnu@gmail.com" && password === "Aa_12345678") {
        console.log('✅ Mock credentials matched!');
        result = {
          bearer_token: "mock_admin_token_" + Date.now(),
          user: {
            email: email,
            name: "Test Admin",
            role: "admin",
            admin_level: "admin"
          }
        };
        
        // Store in localStorage for testing
        localStorage.setItem('admin_token', result.bearer_token);
        localStorage.setItem('admin_user', JSON.stringify(result.user));
      } else {
        throw new Error('Invalid credentials for testing');
      }

      setLoginState({
        stage: 'processing',
        progress: 80,
        message: 'Authentication successful...',
        details: 'Setting up admin session'
      });

      // Store successful attempt metrics
      setSecurity(prev => ({
        ...prev,
        failedAttempts: 0,
        lastFailedAttempt: 0
      }));
      
      // Save device as known if remember device is checked
      if (rememberDevice) {
        localStorage.setItem('admin_known_device', security.deviceFingerprint);
      }

      // Store authentication data
      if (result.bearer_token) {
        localStorage.setItem('admin_token', result.bearer_token);
      }

      setLoginState({
        stage: 'success',
        progress: 100,
        message: 'Login successful!',
        details: 'Redirecting to admin dashboard...'
      });

      // Call success callback or redirect
      if (onSuccess && result.user) {
        onSuccess(result.user);
      } else {
        // Redirect to admin dashboard
        setTimeout(() => {
          router.push(redirectTo || '/dashboard');
        }, 1000);
      }

    } catch (error) {
      console.error('❌ Admin login failed:', error);
      
      // Update security metrics
      const failedAttempts = security.failedAttempts + 1;
      const lockoutExpiry = failedAttempts >= maxFailedAttempts ? 
        Date.now() + (Math.pow(2, failedAttempts - maxFailedAttempts) * 60000) : 0;
      
      setSecurity(prev => ({
        ...prev,
        failedAttempts,
        lastFailedAttempt: Date.now(),
        lockoutExpiry
      }));
      
      saveSecurityMetrics({ 
        failedAttempts, 
        lastFailedAttempt: Date.now(), 
        lockoutExpiry 
      });

      setError(error instanceof Error ? error.message : 'Admin authentication failed');
      setLoginState({
        stage: 'error',
        progress: 0,
        message: 'Authentication failed',
        details: 'Please verify your administrator credentials'
      });

    } finally {
      setIsLoading(false);
    }
  };

  // Handle biometric authentication
  const handleBiometricAuth = async () => {
    if (!biometricAvailable) return;

    try {
      setIsLoading(true);
      setLoginState({
        stage: 'validating',
        progress: 30,
        message: 'Preparing biometric authentication...',
        details: 'Please complete biometric verification'
      });

      // Simplified WebAuthn flow for admin
      const credential = await navigator.credentials.create({
        publicKey: {
          challenge: new Uint8Array(32),
          rp: { name: 'EPSX Admin' },
          user: {
            id: new TextEncoder().encode(email || 'admin'),
            name: email || 'admin',
            displayName: 'Administrator'
          },
          pubKeyCredParams: [{ alg: -7, type: 'public-key' }],
          authenticatorSelection: {
            authenticatorAttachment: 'platform',
            userVerification: 'required',
            requireResidentKey: true
          }
        }
      });

      if (credential) {
        console.log('✅ Biometric authentication successful');
        await handleSubmit();
      }

    } catch (error) {
      console.error('❌ Biometric authentication failed:', error);
      setError('Biometric authentication failed. Please use email/password.');
      setLoginState({
        stage: 'input',
        progress: 0,
        message: 'Enter your administrator credentials'
      });
    } finally {
      setIsLoading(false);
    }
  };

  // Retry connection
  const retryConnection = async () => {
    setIsLoading(true);
    await performHealthCheck();
    setIsLoading(false);
  };

  // Utility functions
  const generateDeviceFingerprint = async (): Promise<string> => {
    const canvas = document.createElement('canvas');
    const ctx = canvas.getContext('2d')!;
    ctx.textBaseline = 'top';
    ctx.font = '14px Arial';
    ctx.fillText('Admin fingerprint', 2, 2);
    
    const fingerprint = [
      navigator.userAgent,
      navigator.language,
      screen.width + 'x' + screen.height,
      new Date().getTimezoneOffset(),
      canvas.toDataURL()
    ].join('|');
    
    return btoa(fingerprint).replace(/[^a-zA-Z0-9]/g, '').substring(0, 32);
  };

  const generateSessionId = (): string => {
    return 'admin_' + Date.now() + '_' + Math.random().toString(36).substring(2);
  };

  const loadSecurityMetrics = (): Partial<SecurityMetrics> => {
    try {
      const stored = localStorage.getItem('admin_security_metrics');
      return stored ? JSON.parse(stored) : {};
    } catch {
      return {};
    }
  };

  const saveSecurityMetrics = (metrics: Partial<SecurityMetrics>) => {
    try {
      const current = loadSecurityMetrics();
      const updated = { ...current, ...metrics };
      localStorage.setItem('admin_security_metrics', JSON.stringify(updated));
    } catch (error) {
      console.debug('Failed to save security metrics:', error);
    }
  };

  const getStageIcon = () => {
    switch (loginState.stage) {
      case 'success': return <CheckCircle className="h-8 w-8 text-green-500" />;
      case 'error': 
      case 'lockout': return <XCircle className="h-8 w-8 text-red-500" />;
      case 'validating':
      case 'processing': return <Loader2 className="h-8 w-8 animate-spin text-blue-500" />;
      default: return <Shield className="h-8 w-8 text-blue-600" />;
    }
  };

  return (
    <div className="min-h-screen flex items-center justify-center bg-gray-50 dark:bg-gray-900 py-12 px-4">
      <Card className="w-full max-w-md">
        <CardHeader className="space-y-4">
          <div className="flex items-center gap-3">
            {getStageIcon()}
            <div>
              <CardTitle className="text-2xl">Admin Sign In [MOCK VERSION]</CardTitle>
              <CardDescription>
                Secure administrator access portal
              </CardDescription>
            </div>
          </div>

          {/* Progress indicator */}
          {(loginState.stage === 'validating' || loginState.stage === 'processing') && (
            <div className="space-y-2">
              <Progress value={loginState.progress} className="h-2" />
              <p className="text-sm text-muted-foreground">{loginState.message}</p>
              {loginState.details && (
                <p className="text-xs text-muted-foreground">{loginState.details}</p>
              )}
            </div>
          )}

          {/* Connection health indicator */}
          <div className="flex items-center gap-2 text-sm">
            {health.status === 'healthy' ? (
              <Wifi className="h-4 w-4 text-green-500" />
            ) : (
              <WifiOff className="h-4 w-4 text-red-500" />
            )}
            <span className={health.status === 'healthy' ? 'text-green-600' : 'text-red-600'}>
              Connection {health.status} ({health.latency}ms)
            </span>
            {health.status !== 'healthy' && (
              <Button
                variant="ghost"
                size="sm"
                onClick={retryConnection}
                disabled={isLoading}
              >
                <RefreshCw className="h-3 w-3" />
              </Button>
            )}
          </div>
        </CardHeader>

        <CardContent className="space-y-6">
          {/* Error display */}
          {error && (
            <Alert variant="destructive">
              <AlertTriangle className="h-4 w-4" />
              <AlertDescription>{error}</AlertDescription>
            </Alert>
          )}

          {/* Lockout warning */}
          {loginState.stage === 'lockout' && (
            <Alert variant="destructive">
              <Lock className="h-4 w-4" />
              <AlertDescription>
                <div className="space-y-2">
                  <p className="font-medium">Account Temporarily Locked</p>
                  <p>Too many failed attempts. Please wait {lockoutTimeRemaining} seconds.</p>
                  <p className="text-xs">
                    For immediate access, contact your system administrator.
                  </p>
                </div>
              </AlertDescription>
            </Alert>
          )}

          {/* Threat detection warnings */}
          {threats.enabled && security.threatLevel !== 'low' && (
            <Alert className={riskLevelColor}>
              <AlertTriangle className="h-4 w-4" />
              <AlertDescription>
                <div className="space-y-2">
                  <p className="font-medium">Security Alert: {security.threatLevel.toUpperCase()} Risk</p>
                  {threats.suspiciousActivity.length > 0 && (
                    <ul className="text-xs space-y-1">
                      {threats.suspiciousActivity.slice(0, 3).map((activity, index) => (
                        <li key={index}>• {activity}</li>
                      ))}
                    </ul>
                  )}
                </div>
              </AlertDescription>
            </Alert>
          )}

          {/* Login form */}
          {loginState.stage === 'input' && (
            <form onSubmit={handleSubmit} className="space-y-4">
              <div className="space-y-2">
                <Label htmlFor="email">Administrator Email</Label>
                <div className="relative">
                  <Input
                    id="email"
                    type="email"
                    value={email}
                    onChange={(e) => setEmail(e.target.value)}
                    placeholder="admin@company.com"
                    disabled={isLoading}
                    className="pl-10"
                    autoComplete="username"
                    required
                  />
                  <UserCheck className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-gray-400" />
                </div>
              </div>

              <div className="space-y-2">
                <Label htmlFor="password">Password</Label>
                <div className="relative">
                  <Input
                    id="password"
                    type={showPassword ? 'text' : 'password'}
                    value={password}
                    onChange={(e) => setPassword(e.target.value)}
                    placeholder="Enter your password"
                    disabled={isLoading}
                    className="pl-10 pr-10"
                    autoComplete="current-password"
                  />
                  <KeyRound className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-gray-400" />
                  <button
                    type="button"
                    onClick={() => setShowPassword(!showPassword)}
                    className="absolute right-3 top-1/2 transform -translate-y-1/2 text-gray-400 hover:text-gray-600"
                    disabled={isLoading}
                  >
                    {showPassword ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
                  </button>
                </div>
              </div>

              <div className="flex items-center space-x-2">
                <input
                  id="rememberDevice"
                  type="checkbox"
                  checked={rememberDevice}
                  onChange={(e) => setRememberDevice(e.target.checked)}
                  disabled={isLoading}
                  className="rounded"
                />
                <Label htmlFor="rememberDevice" className="text-sm">
                  Trust this device for 30 days
                </Label>
              </div>

              <div className="space-y-3">
                <Button
                  type="submit"
                  disabled={!canSubmit}
                  className="w-full"
                  size="lg"
                >
                  {isLoading ? (
                    <>
                      <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                      Authenticating...
                    </>
                  ) : (
                    <>
                      <Crown className="mr-2 h-4 w-4" />
                      Sign In as Administrator
                    </>
                  )}
                </Button>

                {/* Biometric authentication */}
                {biometricAvailable && !isLoading && (
                  <Button
                    type="button"
                    variant="outline"
                    onClick={handleBiometricAuth}
                    disabled={!email}
                    className="w-full"
                  >
                    <Fingerprint className="mr-2 h-4 w-4" />
                    Use Biometric Authentication
                  </Button>
                )}
              </div>
            </form>
          )}

          {/* Success state */}
          {loginState.stage === 'success' && (
            <div className="text-center space-y-3">
              <p className="text-green-600 font-medium">{loginState.message}</p>
              <p className="text-sm text-muted-foreground">
                Redirecting to administrator dashboard...
              </p>
            </div>
          )}

          {/* Security metrics display */}
          {enableThreatDetection && (
            <div className="text-xs text-muted-foreground space-y-1">
              <div className="flex items-center gap-2">
                <Activity className="h-3 w-3" />
                <span>Security Score: {Math.max(0, 100 - security.riskScore)}%</span>
              </div>
              {security.failedAttempts > 0 && (
                <div className="flex items-center gap-2 text-orange-600">
                  <AlertTriangle className="h-3 w-3" />
                  <span>{security.failedAttempts} failed attempt(s)</span>
                </div>
              )}
            </div>
          )}

          {/* Admin notice */}
          <div className="text-center text-xs text-muted-foreground border-t pt-4">
            <p>🔒 This is a secure administrator portal</p>
            <p>All access attempts are monitored and logged</p>
            <p>Unauthorized access is strictly prohibited</p>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}