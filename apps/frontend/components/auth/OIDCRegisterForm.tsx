'use client';

// Advanced OIDC Registration Form
// Intelligent account creation with organizational detection and privacy controls

import React, { useState, useEffect, useCallback, useMemo, useRef } from 'react';
import { useRouter } from 'next/navigation';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Progress } from '@/components/ui/progress';
import { Badge } from '@/components/ui/badge';
import { Checkbox } from '@/components/ui/checkbox';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Textarea } from '@/components/ui/textarea';
import { 
  Loader2, Eye, EyeOff, Shield, CheckCircle, XCircle, AlertTriangle,
  Building2, Globe, UserPlus, Lock, Zap, Info, ExternalLink, Crown
} from 'lucide-react';
import { getOIDCClient } from '@/lib/auth/oidc-client-wrapper';
import { getTenantDetectionService, type TenantInfo } from '@/lib/auth/tenant-detection-service';
import { calculatePasswordStrength, isPasswordWeak } from '@/lib/password-strength';

interface OIDCRegisterFormProps {
  onSuccess?: (user: any) => void;
  redirectTo?: string;
  enableOrgDetection?: boolean;
  enablePrivacyControls?: boolean;
  allowCustomTenant?: boolean;
  requireEmailVerification?: boolean;
}

interface RegistrationData {
  email: string;
  password: string;
  confirmPassword: string;
  firstName: string;
  lastName: string;
  organization?: string;
  jobTitle?: string;
  phone?: string;
  selectedTenant?: TenantInfo;
  agreedToTerms: boolean;
  agreedToPrivacy: boolean;
  acceptMarketing: boolean;
}

interface OrganizationInfo {
  domain: string;
  name: string;
  type: 'enterprise' | 'educational' | 'government' | 'nonprofit' | 'other';
  size: 'startup' | 'small' | 'medium' | 'large' | 'enterprise';
  industry: string;
  hasExistingTenant: boolean;
  suggestedTenant?: TenantInfo;
}

interface PrivacySettings {
  dataRetention: 'minimal' | 'standard' | 'extended';
  analyticsConsent: boolean;
  thirdPartySharing: boolean;
  marketingConsent: boolean;
  profileVisibility: 'private' | 'organization' | 'public';
}

interface RegistrationState {
  step: 'personal' | 'organization' | 'security' | 'privacy' | 'review' | 'processing' | 'success' | 'error';
  progress: number;
  loading: boolean;
  error: string | null;
  validationErrors: Record<string, string>;
}

interface SecurityRecommendations {
  passwordStrength: 'weak' | 'fair' | 'good' | 'strong' | 'excellent';
  mfaRecommended: boolean;
  biometricAvailable: boolean;
  securityScore: number;
  improvements: string[];
}

const INDUSTRIES = [
  'Technology', 'Finance', 'Healthcare', 'Education', 'Government',
  'Manufacturing', 'Retail', 'Energy', 'Transportation', 'Media',
  'Real Estate', 'Legal', 'Consulting', 'Non-profit', 'Other'
];

const ORGANIZATION_SIZES = [
  { value: 'startup', label: '1-10 employees' },
  { value: 'small', label: '11-50 employees' },
  { value: 'medium', label: '51-200 employees' },
  { value: 'large', label: '201-1000 employees' },
  { value: 'enterprise', label: '1000+ employees' }
];

export function OIDCRegisterForm({
  onSuccess,
  redirectTo = '/dashboard',
  enableOrgDetection = true,
  enablePrivacyControls = true,
  allowCustomTenant = false,
  requireEmailVerification = true
}: OIDCRegisterFormProps) {
  const router = useRouter();
  const oidcClient = getOIDCClient();
  const tenantService = getTenantDetectionService();
  const debounceRef = useRef<NodeJS.Timeout>();

  // Form data
  const [data, setData] = useState<RegistrationData>({
    email: '',
    password: '',
    confirmPassword: '',
    firstName: '',
    lastName: '',
    organization: '',
    jobTitle: '',
    phone: '',
    agreedToTerms: false,
    agreedToPrivacy: false,
    acceptMarketing: false
  });

  // State management
  const [state, setState] = useState<RegistrationState>({
    step: 'personal',
    progress: 20,
    loading: false,
    error: null,
    validationErrors: {}
  });

  // Organization detection
  const [orgInfo, setOrgInfo] = useState<OrganizationInfo | null>(null);
  const [orgDetecting, setOrgDetecting] = useState(false);

  // Privacy settings
  const [privacy, setPrivacy] = useState<PrivacySettings>({
    dataRetention: 'standard',
    analyticsConsent: false,
    thirdPartySharing: false,
    marketingConsent: false,
    profileVisibility: 'private'
  });

  // Security recommendations
  const [security, setSecurity] = useState<SecurityRecommendations>({
    passwordStrength: 'weak',
    mfaRecommended: false,
    biometricAvailable: false,
    securityScore: 0,
    improvements: []
  });

  const [showPassword, setShowPassword] = useState(false);
  const [showConfirmPassword, setShowConfirmPassword] = useState(false);

  // Real-time organization detection
  useEffect(() => {
    if (data.email && enableOrgDetection) {
      if (debounceRef.current) {
        clearTimeout(debounceRef.current);
      }
      
      debounceRef.current = setTimeout(() => {
        detectOrganization(data.email);
      }, 1000);
    }
  }, [data.email, enableOrgDetection]);

  // Security analysis
  useEffect(() => {
    if (data.password) {
      analyzeSecurityProfile();
    }
  }, [data.password, data.email]);

  // Computed values
  const passwordStrength = useMemo(() => {
    return data.password ? calculatePasswordStrength(data.password) : null;
  }, [data.password]);

  const canProceedToNext = useMemo(() => {
    switch (state.step) {
      case 'personal':
        return data.email && data.firstName && data.lastName && 
               Object.keys(state.validationErrors).length === 0;
      case 'organization':
        return true; // Optional step
      case 'security':
        return data.password && data.confirmPassword && 
               data.password === data.confirmPassword &&
               !isPasswordWeak(data.password);
      case 'privacy':
        return data.agreedToTerms && data.agreedToPrivacy;
      case 'review':
        return true;
      default:
        return false;
    }
  }, [state.step, state.validationErrors, data]);

  const completionPercentage = useMemo(() => {
    const steps = ['personal', 'organization', 'security', 'privacy', 'review'];
    const currentIndex = steps.indexOf(state.step);
    return ((currentIndex + 1) / steps.length) * 100;
  }, [state.step]);

  // Organization detection
  const detectOrganization = async (email: string) => {
    if (!email.includes('@')) return;

    setOrgDetecting(true);
    try {
      const domain = email.split('@')[1].toLowerCase();
      
      // Detect tenant
      const detection = await tenantService.detectTenant(email);
      
      // Analyze organization info (simplified)
      const orgType = determineOrganizationType(domain);
      const orgSize = 'medium'; // Would be determined by API lookup
      const industry = 'Technology'; // Would be determined by API lookup

      const organizationInfo: OrganizationInfo = {
        domain,
        name: detection.tenant?.name || domain.split('.')[0],
        type: orgType,
        size: orgSize as any,
        industry,
        hasExistingTenant: !!detection.tenant,
        suggestedTenant: detection.tenant || undefined
      };

      setOrgInfo(organizationInfo);

      // Auto-fill organization name if detected
      if (organizationInfo.name && !data.organization) {
        updateField('organization', organizationInfo.name);
      }

    } catch (error) {
      console.error('Organization detection failed:', error);
    } finally {
      setOrgDetecting(false);
    }
  };

  // Security analysis
  const analyzeSecurityProfile = async () => {
    const strength = passwordStrength;
    if (!strength) return;

    const recommendations: SecurityRecommendations = {
      passwordStrength: getPasswordStrengthLevel(strength.score),
      mfaRecommended: strength.score < 4 || data.email.includes('admin'),
      biometricAvailable: 'PublicKeyCredential' in window,
      securityScore: calculateSecurityScore(strength, data),
      improvements: generateSecurityImprovements(strength, data)
    };

    setSecurity(recommendations);
  };

  // Form field updates
  const updateField = useCallback((field: keyof RegistrationData, value: any) => {
    setData(prev => ({ ...prev, [field]: value }));
    
    // Clear validation error for this field
    if (state.validationErrors[field]) {
      setState(prev => ({
        ...prev,
        validationErrors: { ...prev.validationErrors, [field]: undefined }
      }));
    }
    
    // Real-time validation
    validateField(field, value);
  }, [state.validationErrors]);

  // Field validation
  const validateField = (field: keyof RegistrationData, value: any) => {
    let error = '';
    
    switch (field) {
      case 'email':
        if (!value || !/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(value)) {
          error = 'Please enter a valid email address';
        }
        break;
      case 'password':
        if (!value || value.length < 8) {
          error = 'Password must be at least 8 characters long';
        } else if (isPasswordWeak(value)) {
          error = 'Password is too weak. Please choose a stronger password';
        }
        break;
      case 'confirmPassword':
        if (value !== data.password) {
          error = 'Passwords do not match';
        }
        break;
      case 'firstName':
      case 'lastName':
        if (!value || value.trim().length < 2) {
          error = 'This field is required';
        }
        break;
    }
    
    if (error) {
      setState(prev => ({
        ...prev,
        validationErrors: { ...prev.validationErrors, [field]: error }
      }));
    }
  };

  // Navigate between steps
  const nextStep = () => {
    const steps = ['personal', 'organization', 'security', 'privacy', 'review'];
    const currentIndex = steps.indexOf(state.step);
    
    if (currentIndex < steps.length - 1) {
      setState(prev => ({
        ...prev,
        step: steps[currentIndex + 1] as any,
        progress: ((currentIndex + 2) / steps.length) * 80 // Leave 20% for processing
      }));
    }
  };

  const prevStep = () => {
    const steps = ['personal', 'organization', 'security', 'privacy', 'review'];
    const currentIndex = steps.indexOf(state.step);
    
    if (currentIndex > 0) {
      setState(prev => ({
        ...prev,
        step: steps[currentIndex - 1] as any,
        progress: (currentIndex / steps.length) * 80
      }));
    }
  };

  // Handle registration
  const handleRegister = async () => {
    try {
      setState(prev => ({ 
        ...prev, 
        loading: true, 
        error: null, 
        step: 'processing',
        progress: 90
      }));

      // Initialize OIDC client
      await oidcClient.initialize(data.email);

      // Create registration data
      const registrationPayload = {
        email: data.email,
        password: data.password,
        firstName: data.firstName,
        lastName: data.lastName,
        organization: data.organization,
        jobTitle: data.jobTitle,
        phone: data.phone,
        tenant_id: data.selectedTenant?.tenant_id || orgInfo?.suggestedTenant?.tenant_id,
        privacy_settings: privacy,
        terms_accepted: data.agreedToTerms,
        privacy_policy_accepted: data.agreedToPrivacy,
        marketing_consent: data.acceptMarketing
      };

      // TODO: Implement registration API call
      console.log('Registration payload:', registrationPayload);

      // For now, simulate registration success and proceed with OIDC login
      await new Promise(resolve => setTimeout(resolve, 2000));

      setState(prev => ({ ...prev, progress: 95 }));

      // Attempt automatic login after registration
      const user = await oidcClient.signInPopup(data.email);
      
      setState(prev => ({ 
        ...prev, 
        step: 'success',
        progress: 100,
        loading: false
      }));

      // Handle success
      if (onSuccess) {
        onSuccess(user);
      } else {
        setTimeout(() => {
          router.push(redirectTo);
        }, 2000);
      }

    } catch (error) {
      console.error('Registration failed:', error);
      setState(prev => ({
        ...prev,
        loading: false,
        error: error instanceof Error ? error.message : 'Registration failed',
        step: 'error'
      }));
    }
  };

  // Utility functions
  const determineOrganizationType = (domain: string): OrganizationInfo['type'] => {
    if (domain.endsWith('.edu')) return 'educational';
    if (domain.endsWith('.gov') || domain.endsWith('.mil')) return 'government';
    if (domain.endsWith('.org')) return 'nonprofit';
    return 'enterprise';
  };

  const getPasswordStrengthLevel = (score: number): SecurityRecommendations['passwordStrength'] => {
    if (score <= 1) return 'weak';
    if (score <= 2) return 'fair';
    if (score <= 3) return 'good';
    if (score <= 4) return 'strong';
    return 'excellent';
  };

  const calculateSecurityScore = (strength: any, formData: RegistrationData): number => {
    let score = strength.score * 20; // 0-100 scale
    
    // Bonus for organizational email
    if (orgInfo?.hasExistingTenant) score += 10;
    
    // Penalty for common passwords
    if (formData.password.toLowerCase().includes(formData.firstName.toLowerCase())) score -= 20;
    
    return Math.max(0, Math.min(100, score));
  };

  const generateSecurityImprovements = (strength: any, formData: RegistrationData): string[] => {
    const improvements: string[] = [];
    
    if (strength.score < 3) {
      improvements.push('Use a longer password with mixed characters');
    }
    
    if (!strength.requirements.symbols) {
      improvements.push('Add special characters to your password');
    }
    
    if (formData.password.toLowerCase().includes(formData.firstName.toLowerCase())) {
      improvements.push('Avoid using your name in the password');
    }
    
    if (security.mfaRecommended) {
      improvements.push('Enable two-factor authentication after registration');
    }
    
    return improvements;
  };

  // Step rendering functions
  const renderPersonalStep = () => (
    <div className="space-y-4">
      <div className="grid grid-cols-2 gap-4">
        <div className="space-y-2">
          <Label htmlFor="firstName">First Name *</Label>
          <Input
            id="firstName"
            value={data.firstName}
            onChange={(e) => updateField('firstName', e.target.value)}
            placeholder="John"
            disabled={state.loading}
            className={state.validationErrors.firstName ? 'border-red-500' : ''}
          />
          {state.validationErrors.firstName && (
            <p className="text-sm text-red-600">{state.validationErrors.firstName}</p>
          )}
        </div>
        
        <div className="space-y-2">
          <Label htmlFor="lastName">Last Name *</Label>
          <Input
            id="lastName"
            value={data.lastName}
            onChange={(e) => updateField('lastName', e.target.value)}
            placeholder="Doe"
            disabled={state.loading}
            className={state.validationErrors.lastName ? 'border-red-500' : ''}
          />
          {state.validationErrors.lastName && (
            <p className="text-sm text-red-600">{state.validationErrors.lastName}</p>
          )}
        </div>
      </div>

      <div className="space-y-2">
        <Label htmlFor="email">
          Email Address *
          {orgDetecting && <Loader2 className="h-3 w-3 inline ml-1 animate-spin" />}
        </Label>
        <Input
          id="email"
          type="email"
          value={data.email}
          onChange={(e) => updateField('email', e.target.value)}
          placeholder="john.doe@company.com"
          disabled={state.loading}
          className={state.validationErrors.email ? 'border-red-500' : ''}
        />
        {state.validationErrors.email && (
          <p className="text-sm text-red-600">{state.validationErrors.email}</p>
        )}
      </div>

      <div className="space-y-2">
        <Label htmlFor="phone">Phone Number</Label>
        <Input
          id="phone"
          type="tel"
          value={data.phone || ''}
          onChange={(e) => updateField('phone', e.target.value)}
          placeholder="+1 (555) 123-4567"
          disabled={state.loading}
        />
      </div>
    </div>
  );

  const renderOrganizationStep = () => (
    <div className="space-y-4">
      {orgInfo && (
        <Alert>
          <Building2 className="h-4 w-4" />
          <AlertDescription>
            <div className="space-y-2">
              <p><strong>Organization Detected:</strong> {orgInfo.name}</p>
              <p className="text-sm text-muted-foreground">
                {orgInfo.domain} • {orgInfo.type} • {orgInfo.industry}
              </p>
              {orgInfo.hasExistingTenant && (
                <Badge variant="outline" className="text-xs">
                  <Crown className="h-3 w-3 mr-1" />
                  Existing OIDC Integration Available
                </Badge>
              )}
            </div>
          </AlertDescription>
        </Alert>
      )}

      <div className="space-y-2">
        <Label htmlFor="organization">Organization</Label>
        <Input
          id="organization"
          value={data.organization || ''}
          onChange={(e) => updateField('organization', e.target.value)}
          placeholder="Your company name"
          disabled={state.loading}
        />
      </div>

      <div className="space-y-2">
        <Label htmlFor="jobTitle">Job Title</Label>
        <Input
          id="jobTitle"
          value={data.jobTitle || ''}
          onChange={(e) => updateField('jobTitle', e.target.value)}
          placeholder="Software Engineer"
          disabled={state.loading}
        />
      </div>

      <div className="space-y-2">
        <Label>Organization Size</Label>
        <Select disabled={state.loading}>
          <SelectTrigger>
            <SelectValue placeholder="Select organization size" />
          </SelectTrigger>
          <SelectContent>
            {ORGANIZATION_SIZES.map(size => (
              <SelectItem key={size.value} value={size.value}>
                {size.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>

      <div className="space-y-2">
        <Label>Industry</Label>
        <Select disabled={state.loading}>
          <SelectTrigger>
            <SelectValue placeholder="Select industry" />
          </SelectTrigger>
          <SelectContent>
            {INDUSTRIES.map(industry => (
              <SelectItem key={industry} value={industry.toLowerCase()}>
                {industry}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>
    </div>
  );

  const renderSecurityStep = () => (
    <div className="space-y-4">
      <div className="space-y-2">
        <Label htmlFor="password">Password *</Label>
        <div className="relative">
          <Input
            id="password"
            type={showPassword ? 'text' : 'password'}
            value={data.password}
            onChange={(e) => updateField('password', e.target.value)}
            placeholder="Create a strong password"
            className={`pr-10 ${state.validationErrors.password ? 'border-red-500' : ''}`}
            disabled={state.loading}
          />
          <button
            type="button"
            onClick={() => setShowPassword(!showPassword)}
            className="absolute right-3 top-1/2 transform -translate-y-1/2"
          >
            {showPassword ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
          </button>
        </div>
        {state.validationErrors.password && (
          <p className="text-sm text-red-600">{state.validationErrors.password}</p>
        )}
      </div>

      <div className="space-y-2">
        <Label htmlFor="confirmPassword">Confirm Password *</Label>
        <div className="relative">
          <Input
            id="confirmPassword"
            type={showConfirmPassword ? 'text' : 'password'}
            value={data.confirmPassword}
            onChange={(e) => updateField('confirmPassword', e.target.value)}
            placeholder="Confirm your password"
            className={`pr-10 ${state.validationErrors.confirmPassword ? 'border-red-500' : ''}`}
            disabled={state.loading}
          />
          <button
            type="button"
            onClick={() => setShowConfirmPassword(!showConfirmPassword)}
            className="absolute right-3 top-1/2 transform -translate-y-1/2"
          >
            {showConfirmPassword ? <EyeOff className="h-4 w-4" /> : <Eye className="h-4 w-4" />}
          </button>
        </div>
        {state.validationErrors.confirmPassword && (
          <p className="text-sm text-red-600">{state.validationErrors.confirmPassword}</p>
        )}
      </div>

      {/* Password strength indicator */}
      {passwordStrength && data.password && (
        <div className="space-y-3 p-4 bg-gray-50 rounded-lg">
          <div className="flex items-center gap-2">
            <Shield className="h-4 w-4" />
            <span className="font-medium">Password Strength</span>
          </div>
          
          <div className="flex items-center gap-2">
            <span className="text-sm">{passwordStrength.symbol}</span>
            <span className={`text-sm font-medium ${passwordStrength.color}`}>
              {passwordStrength.text} ({passwordStrength.score}/5)
            </span>
          </div>
          
          <div className="grid grid-cols-2 gap-2 text-xs">
            {Object.entries(passwordStrength.requirements).map(([key, met]) => (
              <div key={key} className={`flex items-center gap-1 ${met ? 'text-green-600' : 'text-red-600'}`}>
                {met ? <CheckCircle className="h-3 w-3" /> : <XCircle className="h-3 w-3" />}
                <span>
                  {key === 'length' && 'At least 8 characters'}
                  {key === 'lowercase' && 'Lowercase letter'}
                  {key === 'uppercase' && 'Uppercase letter'}
                  {key === 'numbers' && 'Number'}
                  {key === 'symbols' && 'Special character'}
                </span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Security recommendations */}
      {security.improvements.length > 0 && (
        <Alert>
          <Info className="h-4 w-4" />
          <AlertDescription>
            <p className="font-medium mb-2">Security Recommendations:</p>
            <ul className="text-sm space-y-1">
              {security.improvements.map((improvement, index) => (
                <li key={index} className="flex items-start gap-2">
                  <span className="text-blue-600">•</span>
                  {improvement}
                </li>
              ))}
            </ul>
          </AlertDescription>
        </Alert>
      )}
    </div>
  );

  const renderPrivacyStep = () => (
    <div className="space-y-6">
      {enablePrivacyControls && (
        <div className="space-y-4">
          <h3 className="font-medium">Privacy Settings</h3>
          
          <div className="space-y-3">
            <div className="flex items-start space-x-3">
              <Checkbox
                id="analyticsConsent"
                checked={privacy.analyticsConsent}
                onCheckedChange={(checked) => 
                  setPrivacy(prev => ({ ...prev, analyticsConsent: !!checked }))
                }
              />
              <div className="grid gap-1.5 leading-none">
                <Label htmlFor="analyticsConsent" className="text-sm font-normal">
                  Allow analytics and performance tracking
                </Label>
                <p className="text-xs text-muted-foreground">
                  Help us improve our service by sharing anonymous usage data.
                </p>
              </div>
            </div>

            <div className="flex items-start space-x-3">
              <Checkbox
                id="marketingConsent"
                checked={privacy.marketingConsent}
                onCheckedChange={(checked) => 
                  setPrivacy(prev => ({ ...prev, marketingConsent: !!checked }))
                }
              />
              <div className="grid gap-1.5 leading-none">
                <Label htmlFor="marketingConsent" className="text-sm font-normal">
                  Receive marketing communications
                </Label>
                <p className="text-xs text-muted-foreground">
                  Get updates about new features and special offers.
                </p>
              </div>
            </div>
          </div>
        </div>
      )}

      <div className="space-y-4">
        <h3 className="font-medium">Required Agreements</h3>
        
        <div className="flex items-start space-x-3">
          <Checkbox
            id="agreedToTerms"
            checked={data.agreedToTerms}
            onCheckedChange={(checked) => updateField('agreedToTerms', !!checked)}
            required
          />
          <div className="grid gap-1.5 leading-none">
            <Label htmlFor="agreedToTerms" className="text-sm font-normal">
              I agree to the <a href="/terms" className="text-blue-600 hover:underline" target="_blank">Terms of Service</a> *
            </Label>
          </div>
        </div>

        <div className="flex items-start space-x-3">
          <Checkbox
            id="agreedToPrivacy"
            checked={data.agreedToPrivacy}
            onCheckedChange={(checked) => updateField('agreedToPrivacy', !!checked)}
            required
          />
          <div className="grid gap-1.5 leading-none">
            <Label htmlFor="agreedToPrivacy" className="text-sm font-normal">
              I accept the <a href="/privacy" className="text-blue-600 hover:underline" target="_blank">Privacy Policy</a> *
            </Label>
          </div>
        </div>

        <div className="flex items-start space-x-3">
          <Checkbox
            id="acceptMarketing"
            checked={data.acceptMarketing}
            onCheckedChange={(checked) => updateField('acceptMarketing', !!checked)}
          />
          <div className="grid gap-1.5 leading-none">
            <Label htmlFor="acceptMarketing" className="text-sm font-normal">
              I want to receive product updates and marketing communications
            </Label>
          </div>
        </div>
      </div>
    </div>
  );

  const renderReviewStep = () => (
    <div className="space-y-6">
      <h3 className="font-medium">Review Your Information</h3>
      
      <div className="space-y-4">
        <div className="p-4 border rounded-lg space-y-2">
          <h4 className="font-medium text-sm">Personal Information</h4>
          <p className="text-sm">{data.firstName} {data.lastName}</p>
          <p className="text-sm">{data.email}</p>
          {data.phone && <p className="text-sm">{data.phone}</p>}
        </div>

        {(data.organization || data.jobTitle) && (
          <div className="p-4 border rounded-lg space-y-2">
            <h4 className="font-medium text-sm">Organization</h4>
            {data.organization && <p className="text-sm">{data.organization}</p>}
            {data.jobTitle && <p className="text-sm">{data.jobTitle}</p>}
          </div>
        )}

        {orgInfo?.hasExistingTenant && (
          <Alert>
            <Crown className="h-4 w-4" />
            <AlertDescription>
              Your organization has an existing OIDC integration. After registration,
              you'll be able to use single sign-on for faster access.
            </AlertDescription>
          </Alert>
        )}

        <div className="p-4 border rounded-lg space-y-2">
          <h4 className="font-medium text-sm">Security</h4>
          <div className="flex items-center gap-2">
            <Shield className="h-4 w-4" />
            <span className="text-sm">Password Strength: {security.passwordStrength}</span>
          </div>
          <p className="text-xs text-muted-foreground">
            Security Score: {security.securityScore}%
          </p>
        </div>
      </div>
    </div>
  );

  const renderCurrentStep = () => {
    switch (state.step) {
      case 'personal': return renderPersonalStep();
      case 'organization': return renderOrganizationStep();
      case 'security': return renderSecurityStep();
      case 'privacy': return renderPrivacyStep();
      case 'review': return renderReviewStep();
      case 'processing':
        return (
          <div className="text-center space-y-4">
            <Loader2 className="h-8 w-8 animate-spin mx-auto" />
            <p>Creating your account...</p>
            <p className="text-sm text-muted-foreground">
              This may take a few moments as we set up your secure workspace.
            </p>
          </div>
        );
      case 'success':
        return (
          <div className="text-center space-y-4">
            <CheckCircle className="h-8 w-8 text-green-500 mx-auto" />
            <p className="font-medium text-green-600">Account created successfully!</p>
            <p className="text-sm text-muted-foreground">
              Redirecting you to your dashboard...
            </p>
          </div>
        );
      case 'error':
        return (
          <div className="text-center space-y-4">
            <XCircle className="h-8 w-8 text-red-500 mx-auto" />
            <p className="font-medium text-red-600">Registration failed</p>
            <p className="text-sm text-muted-foreground">{state.error}</p>
            <Button onClick={() => setState(prev => ({ ...prev, step: 'review', error: null }))}>
              Try Again
            </Button>
          </div>
        );
      default:
        return null;
    }
  };

  return (
    <Card className="w-full max-w-2xl mx-auto">
      <CardHeader className="space-y-4">
        <div className="flex items-center gap-3">
          <UserPlus className="h-6 w-6" />
          <div>
            <CardTitle className="text-2xl">Create Your Account</CardTitle>
            <CardDescription>
              Join EPSX to access advanced analytics and trading insights
            </CardDescription>
          </div>
        </div>

        {/* Progress indicator */}
        <div className="space-y-2">
          <Progress value={completionPercentage} className="h-2" />
          <p className="text-sm text-muted-foreground text-center">
            Step {['personal', 'organization', 'security', 'privacy', 'review'].indexOf(state.step) + 1} of 5
          </p>
        </div>
      </CardHeader>

      <CardContent className="space-y-6">
        {state.error && (
          <Alert variant="destructive">
            <XCircle className="h-4 w-4" />
            <AlertDescription>{state.error}</AlertDescription>
          </Alert>
        )}

        {renderCurrentStep()}

        {/* Navigation buttons */}
        {!['processing', 'success', 'error'].includes(state.step) && (
          <div className="flex justify-between">
            <Button
              variant="outline"
              onClick={prevStep}
              disabled={state.step === 'personal' || state.loading}
            >
              Previous
            </Button>

            {state.step === 'review' ? (
              <Button
                onClick={handleRegister}
                disabled={!canProceedToNext || state.loading}
                className="flex items-center gap-2"
              >
                {state.loading ? (
                  <>
                    <Loader2 className="h-4 w-4 animate-spin" />
                    Creating Account...
                  </>
                ) : (
                  <>
                    <UserPlus className="h-4 w-4" />
                    Create Account
                  </>
                )}
              </Button>
            ) : (
              <Button
                onClick={nextStep}
                disabled={!canProceedToNext || state.loading}
              >
                Next
              </Button>
            )}
          </div>
        )}

        {/* Login link */}
        {!['processing', 'success'].includes(state.step) && (
          <div className="text-center text-sm text-muted-foreground">
            Already have an account?{' '}
            <button
              onClick={() => router.push('/login')}
              className="text-blue-600 hover:underline"
              disabled={state.loading}
            >
              Sign in here
            </button>
          </div>
        )}
      </CardContent>
    </Card>
  );
}