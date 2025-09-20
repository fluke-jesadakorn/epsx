'use client';

import { useState, useEffect } from 'react';
import { useAccount } from 'wagmi';
import { Mail, Link, CheckCircle, AlertCircle, X, Bell } from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Badge } from '@/components/ui/badge';
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger } from '@/components/ui/dialog';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { toast } from 'sonner';

interface EmailLinkingProps {
  onLinkSuccess?: (email: string) => void;
  onLinkError?: (error: string) => void;
  className?: string;
  autoShow?: boolean;
  showAsDialog?: boolean;
}

interface LinkingState {
  isLinked: boolean;
  linkedEmail?: string;
  isLinking: boolean;
  emailToLink: string;
  verificationCode: string;
  isVerifying: boolean;
  showVerification: boolean;
  error?: string;
}

export function EmailLinking({ 
  onLinkSuccess, 
  onLinkError, 
  className = '',
  autoShow = false,
  showAsDialog = false
}: EmailLinkingProps) {
  const { address, isConnected } = useAccount();
  const [linkingState, setLinkingState] = useState<LinkingState>({
    isLinked: false,
    isLinking: false,
    emailToLink: '',
    verificationCode: '',
    isVerifying: false,
    showVerification: false,
  });

  // Check if email is already linked
  useEffect(() => {
    if (address && isConnected) {
      checkEmailLinkStatus();
    }
  }, [address, isConnected]);

  const checkEmailLinkStatus = async () => {
    try {
      const response = await fetch('/api/auth/web3/email-status', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ wallet_address: address }),
        credentials: 'include',
      });

      if (response.ok) {
        const data = await response.json();
        setLinkingState(prev => ({
          ...prev,
          isLinked: data.is_linked,
          linkedEmail: data.email,
        }));
      }
    } catch (error) {
      console.error('Failed to check email link status:', error);
    }
  };

  const handleInitiateLinking = async () => {
    if (!linkingState.emailToLink) {
      setLinkingState(prev => ({ ...prev, error: 'Please enter an email address' }));
      return;
    }

    setLinkingState(prev => ({ ...prev, isLinking: true, error: undefined }));

    try {
      const response = await fetch('/api/auth/web3/link-email', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ 
          wallet_address: address,
          email: linkingState.emailToLink 
        }),
        credentials: 'include',
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.message || 'Failed to initiate email linking');
      }

      setLinkingState(prev => ({
        ...prev,
        isLinking: false,
        showVerification: true,
      }));

      toast.success('Verification code sent to your email');

    } catch (error: any) {
      console.error('Email linking error:', error);
      const errorMessage = error.message || 'Failed to send verification code';
      setLinkingState(prev => ({ 
        ...prev, 
        isLinking: false, 
        error: errorMessage 
      }));
      toast.error(errorMessage);
      onLinkError?.(errorMessage);
    }
  };

  const handleVerifyCode = async () => {
    if (!linkingState.verificationCode) {
      setLinkingState(prev => ({ ...prev, error: 'Please enter the verification code' }));
      return;
    }

    setLinkingState(prev => ({ ...prev, isVerifying: true, error: undefined }));

    try {
      const response = await fetch('/api/auth/web3/verify-email', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ 
          wallet_address: address,
          email: linkingState.emailToLink,
          verification_code: linkingState.verificationCode
        }),
        credentials: 'include',
      });

      if (!response.ok) {
        const error = await response.json();
        throw new Error(error.message || 'Invalid verification code');
      }

      // Success
      setLinkingState(prev => ({
        ...prev,
        isVerifying: false,
        isLinked: true,
        linkedEmail: prev.emailToLink,
        showVerification: false,
        emailToLink: '',
        verificationCode: '',
      }));

      toast.success('Email successfully linked to your wallet');
      onLinkSuccess?.(linkingState.emailToLink);

    } catch (error: any) {
      console.error('Email verification error:', error);
      const errorMessage = error.message || 'Failed to verify email';
      setLinkingState(prev => ({ 
        ...prev, 
        isVerifying: false, 
        error: errorMessage 
      }));
      toast.error(errorMessage);
      onLinkError?.(errorMessage);
    }
  };

  const handleUnlinkEmail = async () => {
    try {
      const response = await fetch('/api/auth/web3/unlink-email', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ wallet_address: address }),
        credentials: 'include',
      });

      if (!response.ok) {
        throw new Error('Failed to unlink email');
      }

      setLinkingState(prev => ({
        ...prev,
        isLinked: false,
        linkedEmail: undefined,
      }));

      toast.success('Email unlinked from wallet');

    } catch (error: any) {
      console.error('Email unlinking error:', error);
      toast.error(error.message || 'Failed to unlink email');
    }
  };

  const resetLinking = () => {
    setLinkingState(prev => ({
      ...prev,
      showVerification: false,
      emailToLink: '',
      verificationCode: '',
      error: undefined,
    }));
  };

  if (!isConnected) {
    return null;
  }

  const EmailLinkingContent = () => (
    <div className={`space-y-4 ${className}`}>
      {/* Already Linked State */}
      {linkingState.isLinked && linkingState.linkedEmail && (
        <Alert>
          <CheckCircle className="h-4 w-4" />
          <AlertDescription className="flex items-center justify-between">
            <div>
              <strong>Email linked:</strong> {linkingState.linkedEmail}
              <div className="text-sm text-gray-600 dark:text-gray-400 mt-1">
                You'll receive notifications for wallet activity
              </div>
            </div>
            <Button
              variant="ghost"
              size="sm"
              onClick={handleUnlinkEmail}
              className="text-red-600 hover:text-red-700"
            >
              Unlink
            </Button>
          </AlertDescription>
        </Alert>
      )}

      {/* Not Linked - Show Linking Form */}
      {!linkingState.isLinked && !linkingState.showVerification && (
        <div className="space-y-4">
          <div className="text-center p-6 border-2 border-dashed border-gray-200 dark:border-gray-700 rounded-lg">
            <Bell className="h-12 w-12 text-gray-400 mx-auto mb-4" />
            <h3 className="text-lg font-medium text-gray-900 dark:text-gray-100 mb-2">
              Enable Email Notifications
            </h3>
            <p className="text-sm text-gray-600 dark:text-gray-400 mb-4">
              Link your email to receive notifications about wallet activity, permission changes, and platform updates.
            </p>
            <Badge variant="secondary" className="mb-4">
              <Mail className="h-3 w-3 mr-1" />
              Optional Feature
            </Badge>
          </div>

          <div className="space-y-3">
            <div>
              <Label htmlFor="email">Email Address</Label>
              <Input
                id="email"
                type="email"
                placeholder="your.email@example.com"
                value={linkingState.emailToLink}
                onChange={(e) => setLinkingState(prev => ({ 
                  ...prev, 
                  emailToLink: e.target.value,
                  error: undefined 
                }))}
                disabled={linkingState.isLinking}
              />
            </div>

            {linkingState.error && (
              <Alert variant="destructive">
                <AlertCircle className="h-4 w-4" />
                <AlertDescription>{linkingState.error}</AlertDescription>
              </Alert>
            )}

            <div className="flex gap-2">
              <Button
                onClick={handleInitiateLinking}
                disabled={linkingState.isLinking || !linkingState.emailToLink}
                className="flex-1"
              >
                <Link className="h-4 w-4 mr-2" />
                {linkingState.isLinking ? 'Sending Code...' : 'Link Email'}
              </Button>
              
              <Button
                variant="outline"
                onClick={() => setLinkingState(prev => ({ ...prev, emailToLink: '' }))}
                disabled={linkingState.isLinking}
              >
                Skip
              </Button>
            </div>
          </div>
        </div>
      )}

      {/* Verification Code Entry */}
      {linkingState.showVerification && (
        <div className="space-y-4">
          <Alert>
            <Mail className="h-4 w-4" />
            <AlertDescription>
              We've sent a verification code to <strong>{linkingState.emailToLink}</strong>
            </AlertDescription>
          </Alert>

          <div className="space-y-3">
            <div>
              <Label htmlFor="verification_code">Verification Code</Label>
              <Input
                id="verification_code"
                placeholder="Enter 6-digit code"
                value={linkingState.verificationCode}
                onChange={(e) => setLinkingState(prev => ({ 
                  ...prev, 
                  verificationCode: e.target.value.replace(/\D/g, '').slice(0, 6),
                  error: undefined 
                }))}
                disabled={linkingState.isVerifying}
                maxLength={6}
              />
            </div>

            {linkingState.error && (
              <Alert variant="destructive">
                <AlertCircle className="h-4 w-4" />
                <AlertDescription>{linkingState.error}</AlertDescription>
              </Alert>
            )}

            <div className="flex gap-2">
              <Button
                onClick={handleVerifyCode}
                disabled={linkingState.isVerifying || linkingState.verificationCode.length !== 6}
                className="flex-1"
              >
                <CheckCircle className="h-4 w-4 mr-2" />
                {linkingState.isVerifying ? 'Verifying...' : 'Verify Email'}
              </Button>
              
              <Button
                variant="outline"
                onClick={resetLinking}
                disabled={linkingState.isVerifying}
              >
                <X className="h-4 w-4" />
              </Button>
            </div>

            <Button
              variant="ghost"
              size="sm"
              onClick={handleInitiateLinking}
              disabled={linkingState.isLinking}
              className="w-full text-sm"
            >
              Resend Code
            </Button>
          </div>
        </div>
      )}
    </div>
  );

  // Render as dialog if specified
  if (showAsDialog) {
    return (
      <Dialog>
        <DialogTrigger asChild>
          <Button variant="outline" size="sm">
            <Mail className="h-4 w-4 mr-2" />
            {linkingState.isLinked ? 'Email Settings' : 'Link Email'}
          </Button>
        </DialogTrigger>
        <DialogContent className="sm:max-w-md">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <Mail className="h-5 w-5" />
              Email Notifications
            </DialogTitle>
          </DialogHeader>
          <EmailLinkingContent />
        </DialogContent>
      </Dialog>
    );
  }

  // Render as card
  return (
    <Card className={className}>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Mail className="h-5 w-5 text-blue-500" />
          Email Notifications
        </CardTitle>
      </CardHeader>
      <CardContent>
        <EmailLinkingContent />
      </CardContent>
    </Card>
  );
}