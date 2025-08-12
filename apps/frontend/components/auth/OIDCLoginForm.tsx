'use client';

// Advanced OIDC Login Form with Intelligent Features
// Multi-tenant detection, biometric support, adaptive UI, real-time validation

import React, { useState, useEffect, useCallback, useMemo, useRef } from 'react';
import { useRouter } from 'next/navigation';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Progress } from '@/components/ui/progress';
import { Badge } from '@/components/ui/badge';
import { Loader2, Eye, EyeOff, Fingerprint, Shield, AlertTriangle, CheckCircle, XCircle, Zap, Globe, Building2 } from 'lucide-react';
import { getOIDCClient, type AuthenticationMethod, type AuthenticationRisk } from '@/lib/auth/oidc-client-wrapper';
import { getTenantDetectionService, type TenantDetectionResult, type TenantInfo } from '@/lib/auth/tenant-detection-service';
import { calculatePasswordStrength } from '@/lib/password-strength';

interface OIDCLoginFormProps {
  onSuccess?: (user: any) => void;
  redirectTo?: string;
  allowRegister?: boolean;
  enableBiometric?: boolean;
  enableSmartDetection?: boolean;
  preferredAuthMethod?: AuthenticationMethod;
}

interface ValidationState {
  email: string | null;
  password: string | null;
  overall: 'valid' | 'invalid' | 'pending' | 'unknown';
}

interface TenantSuggestion extends TenantInfo {
  confidence: number;
  reason: string;
}

interface BiometricSupport {
  available: boolean;
  types: string[];
  enrolled: boolean;
}

interface SmartDetectionState {
  isAnalyzing: boolean;
  detectedTenant: TenantInfo | null;
  suggestions: TenantSuggestion[];
  confidence: number;
  riskLevel: AuthenticationRisk;
  recommendation: string;
}

interface FormState {
  email: string;
  password: string;
  showPassword: boolean;
  rememberMe: boolean;
  selectedTenant: TenantInfo | null;
  authMethod: AuthenticationMethod;
}

interface UIState {
  loading: boolean;
  error: string | null;
  success: string | null;
  step: 'input' | 'tenant_selection' | 'mfa_challenge' | 'processing' | 'success';
  progress: number;
  showAdvanced: boolean;
}

export function OIDCLoginForm({
  onSuccess,
  redirectTo = '/dashboard',
  allowRegister = true,
  enableBiometric = true,
  enableSmartDetection = true,
  preferredAuthMethod = 'redirect'
}: OIDCLoginFormProps) {
  const router = useRouter();
  const oidcClient = getOIDCClient();
  const tenantService = getTenantDetectionService();
  const inputRef = useRef<HTMLInputElement>(null);
  const debounceRef = useRef<NodeJS.Timeout>();

  // Form state
  const [form, setForm] = useState<FormState>({
    email: '',
    password: '',
    showPassword: false,
    rememberMe: false,
    selectedTenant: null,
    authMethod: preferredAuthMethod
  });

  // UI state
  const [ui, setUI] = useState<UIState>({
    loading: false,
    error: null,
    success: null,
    step: 'input',
    progress: 0,
    showAdvanced: false
  });

  // Validation state
  const [validation, setValidation] = useState<ValidationState>({
    email: null,
    password: null,
    overall: 'unknown'
  });

  // Smart detection state
  const [smartDetection, setSmartDetection] = useState<SmartDetectionState>({
    isAnalyzing: false,
    detectedTenant: null,
    suggestions: [],
    confidence: 0,
    riskLevel: 'low',
    recommendation: 'Standard authentication recommended'
  });

  // Biometric support state
  const [biometric, setBiometric] = useState<BiometricSupport>({
    available: false,
    types: [],
    enrolled: false
  });

  // Initialize component
  useEffect(() => {
    initializeComponent();
    return () => {
      if (debounceRef.current) {
        clearTimeout(debounceRef.current);
      }
    };
  }, []);

  // Real-time email validation and tenant detection
  useEffect(() => {
    if (form.email && enableSmartDetection) {
      if (debounceRef.current) {
        clearTimeout(debounceRef.current);
      }
      
      debounceRef.current = setTimeout(() => {
        performSmartDetection(form.email);
      }, 500); // Debounce for 500ms
    }
  }, [form.email, enableSmartDetection]);

  // Computed values
  const passwordStrength = useMemo(() => {
    return form.password ? calculatePasswordStrength(form.password) : null;
  }, [form.password]);

  const canSubmit = useMemo(() => {
    return validation.overall === 'valid' && !ui.loading;
  }, [validation.overall, ui.loading]);

  const tenantDisplayName = useMemo(() => {
    if (form.selectedTenant) {
      return form.selectedTenant.name || form.selectedTenant.domain;
    }
    if (smartDetection.detectedTenant) {
      return smartDetection.detectedTenant.name || smartDetection.detectedTenant.domain;
    }
    return null;
  }, [form.selectedTenant, smartDetection.detectedTenant]);

  // Initialize component features
  const initializeComponent = async () => {
    // Check biometric availability
    if (enableBiometric && 'PublicKeyCredential' in window) {
      try {
        const available = await PublicKeyCredential.isUserVerifyingPlatformAuthenticatorAvailable();
        setBiometric(prev => ({ ...prev, available }));
        
        if (available) {
          // Check for enrolled credentials
          // This is a simplified check - real implementation would verify specific credentials
          const hasCredentials = localStorage.getItem('biometric_enrolled') === 'true';
          setBiometric(prev => ({ ...prev, enrolled: hasCredentials }));
        }
      } catch (error) {
        console.log('Biometric check failed:', error);
      }
    }

    // Focus email input
    inputRef.current?.focus();
  };

  // Smart tenant detection and risk assessment
  const performSmartDetection = async (email: string) => {
    if (!enableSmartDetection) return;

    setSmartDetection(prev => ({ ...prev, isAnalyzing: true }));

    try {
      // Detect tenant
      const detection: TenantDetectionResult = await tenantService.detectTenant(email);
      
      // Assess risk (simplified implementation)
      const riskLevel = assessRiskLevel(email, detection);
      
      // Generate recommendation
      const recommendation = generateRecommendation(detection, riskLevel);
      
      // Create suggestions with confidence scores
      const suggestions = createTenantSuggestions(detection);

      setSmartDetection({
        isAnalyzing: false,
        detectedTenant: detection.tenant,
        suggestions,
        confidence: getConfidenceScore(detection.confidence),
        riskLevel,
        recommendation
      });

      // Auto-select high-confidence tenant
      if (detection.confidence === 'high' && detection.tenant) {
        setForm(prev => ({ ...prev, selectedTenant: detection.tenant }));
      }

    } catch (error) {
      console.error('Smart detection failed:', error);
      setSmartDetection(prev => ({ 
        ...prev, 
        isAnalyzing: false,
        recommendation: 'Smart detection unavailable - proceeding with standard flow'
      }));
    }
  };

  // Validate form fields
  const validateField = useCallback((field: keyof FormState, value: string) => {
    switch (field) {
      case 'email':
        const emailValid = /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(value);
        setValidation(prev => ({ 
          ...prev, 
          email: emailValid ? null : 'Invalid email format'
        }));
        return emailValid;
        
      case 'password':
        const passwordValid = value.length >= 1; // We're using OIDC, so password validation is less critical
        setValidation(prev => ({ 
          ...prev, 
          password: passwordValid ? null : 'Password required'
        }));
        return passwordValid;
        
      default:
        return true;
    }
  }, []);

  // Update form field with validation
  const updateField = useCallback((field: keyof FormState, value: any) => {
    setForm(prev => ({ ...prev, [field]: value }));
    
    if (typeof value === 'string') {
      validateField(field, value);
    }
    
    // Update overall validation
    setValidation(prev => {
      const hasErrors = Object.values(prev).some(error => error !== null);
      const hasValues = form.email && (form.authMethod === 'redirect' || form.password);
      
      return {
        ...prev,
        overall: hasErrors ? 'invalid' : hasValues ? 'valid' : 'pending'
      };
    });
  }, [validateField, form.email, form.password, form.authMethod]);

  // Handle biometric authentication
  const handleBiometricAuth = async () => {
    if (!biometric.available) return;

    try {
      setUI(prev => ({ ...prev, loading: true, error: null }));
      
      // This is a simplified WebAuthn implementation
      const credential = await navigator.credentials.create({
        publicKey: {
          challenge: new Uint8Array(32),
          rp: { name: 'EPSX' },
          user: {
            id: new TextEncoder().encode(form.email),
            name: form.email,
            displayName: form.email
          },
          pubKeyCredParams: [{ alg: -7, type: 'public-key' }],
          authenticatorSelection: {
            authenticatorAttachment: 'platform',
            userVerification: 'required'
          }
        }
      });

      if (credential) {
        console.log('Biometric authentication successful');
        // Proceed with OIDC flow
        await handleSubmit();
      }

    } catch (error) {
      console.error('Biometric authentication failed:', error);
      setUI(prev => ({ 
        ...prev, 
        loading: false,
        error: 'Biometric authentication failed. Please use email/password.'
      }));
    }
  };

  // Handle form submission
  const handleSubmit = async (event?: React.FormEvent) => {
    if (event) {
      event.preventDefault();
    }

    if (!canSubmit && !event) return;

    try {
      setUI(prev => ({ ...prev, loading: true, error: null, step: 'processing', progress: 20 }));

      // Initialize OIDC client if needed
      await oidcClient.initialize(form.email);
      setUI(prev => ({ ...prev, progress: 40 }));

      // Perform adaptive authentication
      if (form.authMethod === 'popup') {
        const user = await oidcClient.signInPopup(form.email);
        setUI(prev => ({ ...prev, progress: 80, step: 'success' }));
        handleAuthSuccess(user);
      } else if (form.authMethod === 'silent') {
        const user = await oidcClient.signInSilent();
        if (user) {
          setUI(prev => ({ ...prev, progress: 80, step: 'success' }));
          handleAuthSuccess(user);
        } else {
          throw new Error('Silent authentication failed');
        }
      } else {
        // Redirect authentication
        setUI(prev => ({ ...prev, progress: 60 }));
        await oidcClient.signInRedirect(form.email);
        // Redirect is handled by the client, this won't continue
      }

    } catch (error) {
      console.error('Authentication failed:', error);
      setUI(prev => ({
        ...prev,
        loading: false,
        error: error instanceof Error ? error.message : 'Authentication failed',
        step: 'input',
        progress: 0
      }));
    }
  };

  // Handle authentication success
  const handleAuthSuccess = (user: any) => {
    console.log('Authentication successful:', user.profile.email);
    
    setUI(prev => ({ 
      ...prev, 
      loading: false, 
      success: 'Authentication successful!',
      progress: 100
    }));

    // Remember successful tenant
    if (form.selectedTenant || smartDetection.detectedTenant) {
      const tenant = form.selectedTenant || smartDetection.detectedTenant;
      tenantService.setPreferredTenant(tenant!);
    }

    // Call success callback or redirect
    if (onSuccess) {
      onSuccess(user);
    } else {
      setTimeout(() => {
        router.push(redirectTo);
      }, 1000);
    }
  };

  // Utility functions
  const assessRiskLevel = (email: string, detection: TenantDetectionResult): AuthenticationRisk => {
    let risk: AuthenticationRisk = 'low';
    
    if (email.includes('admin') || email.includes('root')) risk = 'medium';
    if (detection.confidence === 'none') risk = 'medium';
    if (!detection.tenant) risk = 'high';
    
    return risk;
  };

  const generateRecommendation = (detection: TenantDetectionResult, risk: AuthenticationRisk): string => {
    if (risk === 'high') return 'High-risk authentication - additional verification may be required';
    if (risk === 'medium') return 'Medium-risk authentication - consider enabling MFA';
    if (detection.confidence === 'high') return 'Tenant automatically detected - secure authentication enabled';
    return 'Standard authentication flow recommended';
  };

  const createTenantSuggestions = (detection: TenantDetectionResult): TenantSuggestion[] => {
    if (!detection.suggestions) return [];
    
    return detection.suggestions.map((tenant, index) => ({
      ...tenant,
      confidence: Math.max(0.1, 0.8 - (index * 0.2)),
      reason: `Matched by ${detection.method} detection`
    }));
  };

  const getConfidenceScore = (confidence: string): number => {
    switch (confidence) {
      case 'high': return 85;
      case 'medium': return 65;
      case 'low': return 40;
      default: return 10;
    }
  };

  const getRiskColor = (risk: AuthenticationRisk): string => {
    switch (risk) {
      case 'critical': return 'text-red-600';
      case 'high': return 'text-red-500';
      case 'medium': return 'text-yellow-500';
      case 'low': return 'text-green-500';
      default: return 'text-gray-500';
    }
  };

  const getStepIcon = () => {
    switch (ui.step) {
      case 'processing': return <Loader2 className="h-5 w-5 animate-spin" />;
      case 'success': return <CheckCircle className="h-5 w-5 text-green-500" />;
      default: return <Shield className="h-5 w-5" />;
    }
  };

  return (
    <Card className="w-full max-w-lg mx-auto">
      <CardHeader className="space-y-4">
        <div className="flex items-center gap-3">
          {getStepIcon()}
          <div>
            <CardTitle className="text-2xl">Secure Sign In</CardTitle>
            <CardDescription>
              {ui.step === 'processing' ? 'Authenticating...' : 
               ui.step === 'success' ? 'Authentication successful!' :
               'Access your EPSX analytics dashboard'}
            </CardDescription>
          </div>
        </div>

        {/* Progress bar during authentication */}
        {ui.loading && (
          <div className="space-y-2">
            <Progress value={ui.progress} className="h-2" />
            <p className="text-sm text-muted-foreground text-center">
              {ui.progress < 40 ? 'Initializing secure connection...' :
               ui.progress < 60 ? 'Detecting authentication provider...' :
               ui.progress < 80 ? 'Processing credentials...' :
               'Finalizing authentication...'}
            </p>
          </div>
        )}
      </CardHeader>

      <CardContent className="space-y-6">
        {/* Error display */}
        {ui.error && (
          <Alert variant="destructive">
            <XCircle className="h-4 w-4" />
            <AlertDescription>{ui.error}</AlertDescription>
          </Alert>
        )}

        {/* Success display */}
        {ui.success && (
          <Alert>
            <CheckCircle className="h-4 w-4" />
            <AlertDescription className="text-green-600">{ui.success}</AlertDescription>
          </Alert>
        )}

        {/* Smart detection results */}
        {enableSmartDetection && (smartDetection.detectedTenant || smartDetection.suggestions.length > 0) && (
          <div className="space-y-3 p-4 bg-blue-50 rounded-lg border border-blue-200">
            <div className="flex items-center gap-2">
              <Zap className="h-4 w-4 text-blue-600" />
              <span className="font-medium text-sm text-blue-900">Smart Detection Results</span>
            </div>
            
            {smartDetection.detectedTenant && (
              <div className="flex items-center gap-3">
                <Building2 className="h-4 w-4 text-blue-600" />
                <div>
                  <p className="font-medium text-sm">{smartDetection.detectedTenant.name}</p>
                  <p className="text-xs text-blue-700">
                    {smartDetection.detectedTenant.domain} • {smartDetection.confidence}% confidence
                  </p>
                </div>
                <Badge variant="outline" className="ml-auto">Auto-detected</Badge>
              </div>
            )}

            <div className={`text-xs ${getRiskColor(smartDetection.riskLevel)}`}>
              <Shield className="h-3 w-3 inline mr-1" />
              {smartDetection.recommendation}
            </div>
          </div>
        )}

        {/* Authentication form */}
        <form onSubmit={handleSubmit} className="space-y-4">
          {/* Email field with real-time validation */}
          <div className="space-y-2">
            <Label htmlFor="email">
              Email Address
              {smartDetection.isAnalyzing && <Loader2 className="h-3 w-3 inline ml-1 animate-spin" />}
            </Label>
            <Input
              ref={inputRef}
              id="email"
              type="email"
              value={form.email}
              onChange={(e) => updateField('email', e.target.value)}
              placeholder="Enter your email address"
              className={validation.email ? 'border-red-500' : 'border-gray-300'}
              disabled={ui.loading}
              autoComplete="email"
              required
            />
            {validation.email && (
              <p className="text-sm text-red-600">{validation.email}</p>
            )}
          </div>

          {/* Password field (conditional) */}
          {form.authMethod !== 'redirect' && (
            <div className="space-y-2">
              <Label htmlFor="password">Password</Label>
              <div className="relative">
                <Input
                  id="password"
                  type={form.showPassword ? 'text' : 'password'}
                  value={form.password}
                  onChange={(e) => updateField('password', e.target.value)}
                  placeholder="Enter your password"
                  className="pr-10"
                  disabled={ui.loading}
                  autoComplete="current-password"
                  required
                />
                <button
                  type="button"
                  onClick={() => updateField('showPassword', !form.showPassword)}
                  className="absolute right-3 top-1/2 transform -translate-y-1/2 text-gray-400 hover:text-gray-600"
                  disabled={ui.loading}
                >
                  {form.showPassword ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
                </button>
              </div>
              
              {/* Password strength indicator */}
              {passwordStrength && form.password && (
                <div className="space-y-1">
                  <div className="flex items-center gap-2">
                    <span className="text-xs">{passwordStrength.symbol}</span>
                    <span className={`text-xs ${passwordStrength.color}`}>
                      {passwordStrength.text}
                    </span>
                  </div>
                </div>
              )}
            </div>
          )}

          {/* Authentication method selection */}
          {ui.showAdvanced && (
            <div className="space-y-2">
              <Label>Authentication Method</Label>
              <div className="flex gap-2">
                {['redirect', 'popup', 'silent'].map((method) => (
                  <Button
                    key={method}
                    type="button"
                    variant={form.authMethod === method ? 'default' : 'outline'}
                    size="sm"
                    onClick={() => updateField('authMethod', method)}
                    disabled={ui.loading}
                  >
                    {method.charAt(0).toUpperCase() + method.slice(1)}
                  </Button>
                ))}
              </div>
            </div>
          )}

          {/* Submit button */}
          <div className="space-y-3">
            <Button
              type="submit"
              className="w-full"
              disabled={!canSubmit}
              size="lg"
            >
              {ui.loading ? (
                <>
                  <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                  {ui.step === 'processing' ? 'Authenticating...' : 'Please wait...'}
                </>
              ) : (
                <>
                  <Shield className="mr-2 h-4 w-4" />
                  Sign In Securely
                </>
              )}
            </Button>

            {/* Biometric authentication option */}
            {enableBiometric && biometric.available && (
              <Button
                type="button"
                variant="outline"
                className="w-full"
                onClick={handleBiometricAuth}
                disabled={ui.loading || !form.email}
              >
                <Fingerprint className="mr-2 h-4 w-4" />
                {biometric.enrolled ? 'Use Biometric Authentication' : 'Set up Biometric Authentication'}
              </Button>
            )}
          </div>

          {/* Additional options */}
          <div className="flex items-center justify-between text-sm">
            <button
              type="button"
              onClick={() => setUI(prev => ({ ...prev, showAdvanced: !prev.showAdvanced }))}
              className="text-blue-600 hover:underline"
              disabled={ui.loading}
            >
              {ui.showAdvanced ? 'Hide' : 'Show'} Advanced Options
            </button>
            
            {allowRegister && (
              <button
                type="button"
                onClick={() => router.push('/register')}
                className="text-blue-600 hover:underline"
                disabled={ui.loading}
              >
                Create Account
              </button>
            )}
          </div>
        </form>

        {/* Tenant information */}
        {tenantDisplayName && (
          <div className="text-center text-sm text-muted-foreground">
            <Globe className="h-3 w-3 inline mr-1" />
            Authenticating with {tenantDisplayName}
          </div>
        )}
      </CardContent>
    </Card>
  );
}