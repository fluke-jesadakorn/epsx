'use client';

import { useState } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import * as z from 'zod';

import { Button } from '@/components/ui/button';
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from '@/components/ui/card';
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from '@/components/ui/form';
import { Input } from '@/components/ui/input';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { LoadingSpinner } from '@/components/ui/loading-spinner';
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar';
import { useSession } from 'next-auth/react';
import { Save, Mail, Lock } from 'lucide-react';

// Validation schemas
const profileSchema = z.object({
  displayName: z
    .string()
    .min(2, 'Display name must be at least 2 characters')
    .optional(),
  email: z.string().email('Please enter a valid email address'),
});

const passwordSchema = z
  .object({
    currentPassword: z.string().min(1, 'Current password is required'),
    newPassword: z.string().min(6, 'Password must be at least 6 characters'),
    confirmPassword: z.string().min(6, 'Please confirm your password'),
  })
  .refine((data) => data.newPassword === data.confirmPassword, {
    message: "Passwords don't match",
    path: ['confirmPassword'],
  });

type ProfileFormData = z.infer<typeof profileSchema>;
type PasswordFormData = z.infer<typeof passwordSchema>;

export function UserProfileSettings() {
  const { data: session, status } = useSession();
  const user = session?.user;
  const loading = status === 'loading';
  
  // Simplified auth utilities
  const getUserInitials = (user: any) => {
    if (!user) return 'U';
    const name = user.name || user.displayName || user.email;
    return name?.charAt(0).toUpperCase() || 'U';
  };
  
  const getUserDisplayName = (user: any) => {
    return user?.name || user?.displayName || user?.email || 'Unknown User';
  };

  const [activeTab, setActiveTab] = useState<
    'profile' | 'security' | 'providers'
  >('profile');
  const [isUpdating, setIsUpdating] = useState(false);
  const [error, setError] = useState<string | null>(null);
  
  // Placeholder functions - these would need to be implemented with server actions
  const updateProfile = async (data: any) => {
    setError('Profile update not implemented');
  };
  
  const changePassword = async (data: any) => {
    setError('Password change not implemented');
  };
  
  const linkGoogleAccount = async () => {
    setError('Account linking not implemented');
  };
  
  const clearError = () => setError(null);

  if (!user) {
    return (
      <Card>
        <CardContent className="p-6">
          <p className="text-center text-muted-foreground">
            Please sign in to view your profile settings.
          </p>
        </CardContent>
      </Card>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex flex-col space-y-4">
        <h1 className="text-3xl font-bold">Profile Settings</h1>
        <p className="text-muted-foreground">
          Manage your account settings and preferences.
        </p>
      </div>

      {error && (
        <Alert variant="destructive">
          <AlertDescription>{error}</AlertDescription>
        </Alert>
      )}

      {/* Profile Header */}
      <Card>
        <CardContent className="p-6">
          <div className="flex items-center space-x-4">
            <Avatar className="h-20 w-20">
              <AvatarImage
                src={user.photoURL || undefined}
                alt={getUserDisplayName()}
              />
              <AvatarFallback className="text-lg">
                {getUserInitials()}
              </AvatarFallback>
            </Avatar>
            <div className="space-y-1">
              <h2 className="text-2xl font-bold">{getUserDisplayName()}</h2>
              <p className="text-muted-foreground">{user.email}</p>
              <div className="flex items-center space-x-2">
                {user.emailVerified ? (
                  <Badge variant="secondary" className="text-green-600">
                    Email Verified
                  </Badge>
                ) : (
                  <Badge variant="destructive">Email Not Verified</Badge>
                )}
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Tabs */}
      <div className="flex space-x-1 bg-muted p-1 rounded-lg w-fit">
        {(['profile', 'security', 'providers'] as const).map((tab) => (
          <Button
            key={tab}
            variant={activeTab === tab ? 'default' : 'ghost'}
            size="sm"
            onClick={() => setActiveTab(tab)}
            className="capitalize"
          >
            {tab}
          </Button>
        ))}
      </div>

      {/* Tab Content */}
      {activeTab === 'profile' && (
        <ProfileTab
          user={user}
          isUpdating={isUpdating}
          setIsUpdating={setIsUpdating}
          updateProfile={updateProfile}
          sendEmailVerification={sendEmailVerification}
          clearError={clearError}
        />
      )}

      {activeTab === 'security' && (
        <SecurityTab
          hasPassword={hasPassword}
          isUpdating={isUpdating}
          setIsUpdating={setIsUpdating}
          changePassword={changePassword}
          clearError={clearError}
        />
      )}

      {activeTab === 'providers' && (
        <ProvidersTab
          providers={providers}
          hasGoogle={hasGoogle}
          isUpdating={isUpdating}
          setIsUpdating={setIsUpdating}
          linkGoogleAccount={linkGoogleAccount}
          unlinkProvider={unlinkProvider}
          clearError={clearError}
        />
      )}
    </div>
  );
}

interface ProfileTabProps {
  user: any;
  isUpdating: boolean;
  setIsUpdating: (updating: boolean) => void;
  updateProfile: (data: {
    displayName?: string;
    photoURL?: string;
  }) => Promise<void>;
  sendEmailVerification: () => Promise<void>;
  clearError: () => void;
}

function ProfileTab({
  user,
  isUpdating,
  setIsUpdating,
  updateProfile,
  sendEmailVerification,
  clearError,
}: ProfileTabProps) {
  const [emailSent, setEmailSent] = useState(false);

  const form = useForm<ProfileFormData>({
    resolver: zodResolver(profileSchema),
    defaultValues: {
      displayName: user.displayName || '',
      email: user.email || '',
    },
  });

  const onSubmit = async (data: ProfileFormData) => {
    try {
      clearError();
      setIsUpdating(true);
      await updateProfile({ displayName: data.displayName });
    } catch (_error) {
      // Error handled by auth context
    } finally {
      setIsUpdating(false);
    }
  };

  const handleSendVerification = async () => {
    try {
      clearError();
      setIsUpdating(true);
      await sendEmailVerification();
      setEmailSent(true);
    } catch (_error) {
      // Error handled by auth context
    } finally {
      setIsUpdating(false);
    }
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle>Profile Information</CardTitle>
        <CardDescription>
          Update your profile information and email verification status.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        <Form {...form}>
          <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
            <FormField
              control={form.control}
              name="displayName"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Display Name</FormLabel>
                  <FormControl>
                    <Input
                      {...field}
                      placeholder="Enter your display name"
                      disabled={isUpdating}
                    />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="email"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Email</FormLabel>
                  <FormControl>
                    <Input
                      {...field}
                      type="email"
                      disabled
                      className="bg-muted"
                    />
                  </FormControl>
                  <p className="text-sm text-muted-foreground">
                    Email cannot be changed through this form.
                  </p>
                  <FormMessage />
                </FormItem>
              )}
            />

            <Button
              type="submit"
              disabled={isUpdating}
              className="flex items-center gap-2"
            >
              {isUpdating ? (
                <>
                  <LoadingSpinner size="sm" />
                  Updating...
                </>
              ) : (
                <>
                  <Save className="h-4 w-4" />
                  Update Profile
                </>
              )}
            </Button>
          </form>
        </Form>

        {!user.emailVerified && (
          <div className="p-4 border rounded-lg bg-yellow-50 dark:bg-yellow-900/20">
            <h4 className="font-medium text-yellow-800 dark:text-yellow-200 mb-2">
              Email Verification Required
            </h4>
            <p className="text-sm text-yellow-700 dark:text-yellow-300 mb-3">
              Please verify your email address to secure your account and enable
              all features.
            </p>
            {emailSent ? (
              <p className="text-sm text-green-600 dark:text-green-400">
                Verification email sent! Check your inbox.
              </p>
            ) : (
              <Button
                variant="outline"
                size="sm"
                onClick={handleSendVerification}
                disabled={isUpdating}
                className="flex items-center gap-2"
              >
                {isUpdating ? (
                  <>
                    <LoadingSpinner size="sm" />
                    Sending...
                  </>
                ) : (
                  <>
                    <Mail className="h-4 w-4" />
                    Send Verification Email
                  </>
                )}
              </Button>
            )}
          </div>
        )}
      </CardContent>
    </Card>
  );
}

interface SecurityTabProps {
  hasPassword: boolean;
  isUpdating: boolean;
  setIsUpdating: (updating: boolean) => void;
  changePassword: (
    currentPassword: string,
    newPassword: string,
  ) => Promise<void>;
  clearError: () => void;
}

function SecurityTab({
  hasPassword,
  isUpdating,
  setIsUpdating,
  changePassword,
  clearError,
}: SecurityTabProps) {
  const [passwordChanged, setPasswordChanged] = useState(false);

  const form = useForm<PasswordFormData>({
    resolver: zodResolver(passwordSchema),
    defaultValues: {
      currentPassword: '',
      newPassword: '',
      confirmPassword: '',
    },
  });

  const onSubmit = async (data: PasswordFormData) => {
    try {
      clearError();
      setIsUpdating(true);
      await changePassword(data.currentPassword, data.newPassword);
      setPasswordChanged(true);
      form.reset();
    } catch (_error) {
      // Error handled by auth context
    } finally {
      setIsUpdating(false);
    }
  };

  if (!hasPassword) {
    return (
      <Card>
        <CardHeader>
          <CardTitle>Security Settings</CardTitle>
          <CardDescription>
            Manage your account security and password.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <div className="p-4 border rounded-lg bg-blue-50 dark:bg-blue-900/20">
            <h4 className="font-medium text-blue-800 dark:text-blue-200 mb-2">
              No Password Set
            </h4>
            <p className="text-sm text-blue-700 dark:text-blue-300">
              You&apos;re currently signed in with Google. You can set a password to
              enable email/password login.
            </p>
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle>Security Settings</CardTitle>
        <CardDescription>
          Change your password to keep your account secure.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-6">
        {passwordChanged && (
          <Alert>
            <AlertDescription>
              Your password has been changed successfully.
            </AlertDescription>
          </Alert>
        )}

        <Form {...form}>
          <form onSubmit={form.handleSubmit(onSubmit)} className="space-y-4">
            <FormField
              control={form.control}
              name="currentPassword"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Current Password</FormLabel>
                  <FormControl>
                    <Input
                      {...field}
                      type="password"
                      placeholder="Enter your current password"
                      disabled={isUpdating}
                    />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="newPassword"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>New Password</FormLabel>
                  <FormControl>
                    <Input
                      {...field}
                      type="password"
                      placeholder="Enter your new password"
                      disabled={isUpdating}
                    />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            <FormField
              control={form.control}
              name="confirmPassword"
              render={({ field }) => (
                <FormItem>
                  <FormLabel>Confirm New Password</FormLabel>
                  <FormControl>
                    <Input
                      {...field}
                      type="password"
                      placeholder="Confirm your new password"
                      disabled={isUpdating}
                    />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />

            <Button
              type="submit"
              disabled={isUpdating}
              className="flex items-center gap-2"
            >
              {isUpdating ? (
                <>
                  <LoadingSpinner size="sm" />
                  Changing Password...
                </>
              ) : (
                <>
                  <Lock className="h-4 w-4" />
                  Change Password
                </>
              )}
            </Button>
          </form>
        </Form>
      </CardContent>
    </Card>
  );
}

interface ProvidersTabProps {
  providers: string[];
  hasGoogle: boolean;
  isUpdating: boolean;
  setIsUpdating: (updating: boolean) => void;
  linkGoogleAccount: () => Promise<void>;
  unlinkProvider: (providerId: string) => Promise<void>;
  clearError: () => void;
}

function ProvidersTab({
  providers,
  hasGoogle,
  isUpdating,
  setIsUpdating,
  linkGoogleAccount,
  unlinkProvider,
  clearError,
}: ProvidersTabProps) {
  const handleLinkGoogle = async () => {
    try {
      clearError();
      setIsUpdating(true);
      await linkGoogleAccount();
    } catch (_error) {
      // Error handled by auth context
    } finally {
      setIsUpdating(false);
    }
  };

  const handleUnlinkProvider = async (providerId: string) => {
    try {
      clearError();
      setIsUpdating(true);
      await unlinkProvider(providerId);
    } catch (_error) {
      // Error handled by auth context
    } finally {
      setIsUpdating(false);
    }
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle>Connected Accounts</CardTitle>
        <CardDescription>
          Manage your connected social accounts and sign-in methods.
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        <div className="space-y-3">
          <div className="flex items-center justify-between p-3 border rounded-lg">
            <div className="flex items-center space-x-3">
              <svg className="h-5 w-5" viewBox="0 0 24 24">
                <path
                  d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z"
                  fill="#4285F4"
                />
                <path
                  d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"
                  fill="#34A853"
                />
                <path
                  d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z"
                  fill="#FBBC05"
                />
                <path
                  d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z"
                  fill="#EA4335"
                />
              </svg>
              <div>
                <p className="font-medium">Google</p>
                <p className="text-sm text-muted-foreground">
                  Sign in with your Google account
                </p>
              </div>
            </div>
            <div>
              {hasGoogle ? (
                <div className="flex items-center space-x-2">
                  <Badge variant="secondary">Connected</Badge>
                  {providers.length > 1 && (
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => handleUnlinkProvider('google.com')}
                      disabled={isUpdating}
                    >
                      {isUpdating ? <LoadingSpinner size="sm" /> : 'Disconnect'}
                    </Button>
                  )}
                </div>
              ) : (
                <Button
                  variant="outline"
                  size="sm"
                  onClick={handleLinkGoogle}
                  disabled={isUpdating}
                >
                  {isUpdating ? <LoadingSpinner size="sm" /> : 'Connect'}
                </Button>
              )}
            </div>
          </div>
        </div>

        {providers.length > 1 && (
          <div className="p-3 bg-yellow-50 dark:bg-yellow-900/20 rounded-lg">
            <p className="text-sm text-yellow-700 dark:text-yellow-300">
              <strong>Note:</strong> You need at least one sign-in method
              connected to your account. You cannot disconnect your last
              remaining sign-in method.
            </p>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
