 
'use client';

import { Alert, AlertDescription } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { useApiClient } from '@/shared/hooks/use-api-client';
import { type User } from '@/shared/types/auth';
import { AlertCircle, Bell, Check, Edit, Mail, X } from 'lucide-react';
import { useRouter } from 'next/navigation';
import { useState } from 'react';
import { toast } from 'sonner';

interface EmailManagementProps {
  user: User;
}

interface EmailState {
  isEditing: boolean;
  newEmail: string;
  verificationCode: string;
  isVerifying: boolean;
  isSending: boolean;
  showVerification: boolean;
  error?: string;
}

export function EmailManagement({ user }: EmailManagementProps) {
  const { base } = useApiClient({ platform: 'frontend' });
  const [emailState, setEmailState] = useState<EmailState>({
    isEditing: false,
    newEmail: user.email,
    verificationCode: '',
    isVerifying: false,
    isSending: false,
    showVerification: false,
  });
  const router = useRouter();

  const handleStartEdit = () => {
    setEmailState(prev => ({
      ...prev,
      isEditing: true,
      newEmail: user.email,
      error: undefined,
    }));
  };

  const handleCancelEdit = () => {
    setEmailState(prev => ({
      ...prev,
      isEditing: false,
      newEmail: user.email,
      showVerification: false,
      verificationCode: '',
      error: undefined,
    }));
  };

  const handleSendVerification = async () => {
    if (!emailState.newEmail || emailState.newEmail === user.email) {
      setEmailState(prev => ({ ...prev, error: 'Please enter a new email address' }));
      return;
    }

    setEmailState(prev => ({ ...prev, isSending: true, error: undefined }));

    try {
      await base.post('/api/auth/change-email', { new_email: emailState.newEmail });

      setEmailState(prev => ({
        ...prev,
        isSending: false,
        showVerification: true,
      }));

      toast.success('Verification code sent to your new email address');

    } catch (err: unknown) {
      // Error logged silently
      const error = err as Record<string, unknown>;
      const errorMessage = typeof error?.message === 'string' ? error.message : 'Failed to send verification code';
      setEmailState(prev => ({
        ...prev,
        isSending: false,
        error: errorMessage
      }));
      toast.error(errorMessage);
    }
  };

  const handleVerifyEmail = async () => {
    if (emailState.verificationCode?.length !== 6) {
      setEmailState(prev => ({ ...prev, error: 'Please enter the 6-digit verification code' }));
      return;
    }

    setEmailState(prev => ({ ...prev, isVerifying: true, error: undefined }));

    try {
      await base.post('/api/auth/verify-email-change', {
          new_email: emailState.newEmail,
          verification_code: emailState.verificationCode
        });

      // Success - email changed
      setEmailState(prev => ({
        ...prev,
        isVerifying: false,
        isEditing: false,
        showVerification: false,
        verificationCode: '',
      }));

      toast.success('Email address updated successfully');

      // Refresh page to update user data
      router.refresh();

    } catch (err: unknown) {
      // Error logged silently
      const error = err as Record<string, unknown>;
      const errorMessage = typeof error?.message === 'string' ? error.message : 'Failed to verify email';
      setEmailState(prev => ({
        ...prev,
        isVerifying: false,
        error: errorMessage
      }));
      toast.error(errorMessage);
    }
  };

  return (
    <div className="space-y-6">
      {/* Email Settings */}
      <Card className="border-orange-100 dark:border-slate-700">
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Mail className="h-5 w-5 text-orange-500" />
            Email Settings
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between p-4 bg-slate-50 dark:bg-slate-800 rounded-lg">
            <div>
              <div className="font-medium text-slate-900 dark:text-slate-100">
                Current Email
              </div>
              <div className="text-sm text-slate-600 dark:text-slate-400 flex items-center gap-2">
                {user.email}
                {user.verified && (
                  <Badge variant="default" className="bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-300 text-xs">
                    <Check className="h-3 w-3 mr-1" />
                    Verified
                  </Badge>
                )}
              </div>
            </div>
            {!emailState.isEditing && (
              <Button
                variant="outline"
                size="sm"
                onClick={handleStartEdit}
                className="flex items-center gap-2"
              >
                <Edit className="h-4 w-4" />
                Change Email
              </Button>
            )}
          </div>

          {/* Email Change Form */}
          {emailState.isEditing && !emailState.showVerification && (
            <div className="space-y-4 p-4 border border-orange-200 dark:border-orange-700 rounded-lg">
              <div>
                <Label htmlFor="new_email">New Email Address</Label>
                <Input
                  id="new_email"
                  type="email"
                  placeholder="your.new.email@example.com"
                  value={emailState.newEmail}
                  onChange={(e) => setEmailState(prev => ({
                    ...prev,
                    newEmail: e.target.value,
                    error: undefined
                  }))}
                  disabled={emailState.isSending}
                />
                <p className="text-xs text-slate-500 dark:text-slate-400 mt-1">
                  A verification code will be sent to this email address
                </p>
              </div>

              {emailState.error && (
                <Alert variant="destructive">
                  <AlertCircle className="h-4 w-4" />
                  <AlertDescription>{emailState.error}</AlertDescription>
                </Alert>
              )}

              <div className="flex gap-2">
                <Button
                  onClick={handleSendVerification}
                  disabled={emailState.isSending || !emailState.newEmail}
                  className="flex-1"
                >
                  {emailState.isSending ? 'Sending...' : 'Send Verification Code'}
                </Button>
                <Button
                  variant="outline"
                  onClick={handleCancelEdit}
                  disabled={emailState.isSending}
                >
                  <X className="h-4 w-4" />
                </Button>
              </div>
            </div>
          )}

          {/* Verification Code Form */}
          {emailState.showVerification && (
            <div className="space-y-4 p-4 border border-orange-200 dark:border-orange-700 rounded-lg">
              <Alert>
                <Mail className="h-4 w-4" />
                <AlertDescription>
                  We've sent a verification code to <strong>{emailState.newEmail}</strong>
                </AlertDescription>
              </Alert>

              <div>
                <Label htmlFor="verification_code">Verification Code</Label>
                <Input
                  id="verification_code"
                  placeholder="Enter 6-digit code"
                  value={emailState.verificationCode}
                  onChange={(e) => setEmailState(prev => ({
                    ...prev,
                    verificationCode: e.target.value.replace(/\D/g, '').slice(0, 6),
                    error: undefined
                  }))}
                  disabled={emailState.isVerifying}
                  maxLength={6}
                />
              </div>

              {emailState.error && (
                <Alert variant="destructive">
                  <AlertCircle className="h-4 w-4" />
                  <AlertDescription>{emailState.error}</AlertDescription>
                </Alert>
              )}

              <div className="flex gap-2">
                <Button
                  onClick={handleVerifyEmail}
                  disabled={emailState.isVerifying || emailState.verificationCode.length !== 6}
                  className="flex-1"
                >
                  <Check className="h-4 w-4 mr-2" />
                  {emailState.isVerifying ? 'Verifying...' : 'Verify Email'}
                </Button>
                <Button
                  variant="outline"
                  onClick={handleCancelEdit}
                  disabled={emailState.isVerifying}
                >
                  <X className="h-4 w-4" />
                </Button>
              </div>

              <Button
                variant="ghost"
                size="sm"
                onClick={handleSendVerification}
                disabled={emailState.isSending}
                className="w-full text-sm"
              >
                Resend Code
              </Button>
            </div>
          )}
        </CardContent>
      </Card>

      {/* Email Preferences */}
      <Card className="border-orange-100 dark:border-slate-700">
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <Bell className="h-5 w-5 text-orange-500" />
            Email Preferences
          </CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-3">
            <div className="flex items-center justify-between p-3 bg-slate-50 dark:bg-slate-800 rounded-lg">
              <div>
                <div className="font-medium text-slate-900 dark:text-slate-100">
                  Account Notifications
                </div>
                <div className="text-sm text-slate-600 dark:text-slate-400">
                  Security alerts, login notifications
                </div>
              </div>
              <Badge variant="outline" className="text-green-600 border-green-600">
                Enabled
              </Badge>
            </div>

            <div className="flex items-center justify-between p-3 bg-slate-50 dark:bg-slate-800 rounded-lg">
              <div>
                <div className="font-medium text-slate-900 dark:text-slate-100">
                  Platform Updates
                </div>
                <div className="text-sm text-slate-600 dark:text-slate-400">
                  Feature announcements, system updates
                </div>
              </div>
              <Badge variant="outline" className="text-green-600 border-green-600">
                Enabled
              </Badge>
            </div>

            <div className="flex items-center justify-between p-3 bg-slate-50 dark:bg-slate-800 rounded-lg">
              <div>
                <div className="font-medium text-slate-900 dark:text-slate-100">
                  Marketing Communications
                </div>
                <div className="text-sm text-slate-600 dark:text-slate-400">
                  Promotional offers, newsletter
                </div>
              </div>
              <Badge variant="outline" className="text-slate-600 border-slate-600">
                Disabled
              </Badge>
            </div>
          </div>

          <div className="pt-4 border-t border-slate-200 dark:border-slate-700">
            <p className="text-sm text-slate-600 dark:text-slate-400">
              Email preferences are managed through your account settings.
              Critical security notifications cannot be disabled.
            </p>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}