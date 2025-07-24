'use client';

import { apiClient, isApiError   } from '@/lib/api-client';
import type {ProfileUpdateRequest, PasswordChangeRequest} from '@/lib/api-client';
import { useEffect, useState } from 'react';

import type { UserLevelType } from '@/app/constants/packages';
import { canAccessLevel, getPackageByLevel } from '@/app/constants/packages';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Avatar, AvatarFallback, AvatarImage } from '@/components/ui/avatar';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { useAuth } from '@/context/auth-context';
import { status } from '@/services/pay';
import { Crown, Gem, Key, Save, Star, Trophy, User } from 'lucide-react';

export function ProfileSettings() {
  const { user } = useAuth();
  
  // All users are now backend-authenticated
  const isBackendUser = true;
  
  const [displayName, setDisplayName] = useState<string>('');
  const [photoURL, setPhotoURL] = useState<string>('');
  const [showProfileForm, setShowProfileForm] = useState(false);
  const [showPasswordForm, setShowPasswordForm] = useState(false);
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<boolean>(false);
  const [currentPassword, setCurrentPassword] = useState<string>('');
  const [newPassword, setNewPassword] = useState<string>('');
  const [confirmPassword, setConfirmPassword] = useState<string>('');
  const [isPasswordLoading, setIsPasswordLoading] = useState<boolean>(false);
  const [passwordError, setPasswordError] = useState<string | null>(null);
  const [passwordSuccess, setPasswordSuccess] = useState<boolean>(false);
  const [userLevel, setUserLevel] = useState<UserLevelType>('BASIC');
  const [isLoadingLevel, setIsLoadingLevel] = useState(true);

  const levelIcons = {
    BASIC: <Star className="h-5 w-5" />,
    SILVER: <Trophy className="h-5 w-5" />,
    GOLD: <Crown className="h-5 w-5" />,
    PLATINUM: <Gem className="h-5 w-5" />,
    API_PERSONAL: <Star className="h-5 w-5" />,
    API_COMPANY: <Star className="h-5 w-5" />,
    API_PARTNER: <Star className="h-5 w-5" />,
  };

  const levelGradients = {
    BRONZE: 'from-amber-400 to-amber-600',
    BASIC: 'from-gray-400 to-gray-600',
    SILVER: 'from-slate-400 to-slate-600',
    GOLD: 'from-yellow-400 to-orange-500',
    PLATINUM: 'from-purple-500 to-pink-600',
    DIAMOND: 'from-blue-500 to-cyan-600',
    VIP: 'from-red-500 to-pink-600',
    API_PERSONAL: 'from-indigo-500 to-blue-600',
    API_COMPANY: 'from-blue-600 to-cyan-600',
    API_PARTNER: 'from-purple-600 to-indigo-700',
  };

  useEffect(() => {
    if (user) {
      setDisplayName(user.displayName || '');
      setPhotoURL(user.photoURL || '');

      // Fetch user level
      const fetchUserLevel = async () => {
        try {
          const userStatus = await status();
          setUserLevel(userStatus.level as UserLevelType);
        } catch (error) {
          console.error('Failed to fetch user level:', error);
          setUserLevel('BASIC');
        } finally {
          setIsLoadingLevel(false);
        }
      };

      fetchUserLevel();
    } else {
      setIsLoadingLevel(false);
    }
  }, [user]);

  const handleUpdateProfile = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!user) return;

    setIsLoading(true);
    setError(null);
    setSuccess(false);

    try {
      const response = await apiClient.updateProfile({
        displayName,
        photoURL: photoURL || '',
      });

      if (isApiError(response)) {
        setError(response.error || 'Failed to update profile. Please try again.');
      } else {
        setSuccess(true);
        setTimeout(() => setSuccess(false), 3000);
      }
    } catch (err) {
      setError('Failed to update profile. Please try again.');
      console.error('Profile update error:', err);
    } finally {
      setIsLoading(false);
    }
  };

  const handleChangePassword = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!user) return;

    if (newPassword !== confirmPassword) {
      setPasswordError('New passwords do not match.');
      return;
    }

    setIsPasswordLoading(true);
    setPasswordError(null);
    setPasswordSuccess(false);

    try {
      const response = await apiClient.changePassword({
        currentPassword,
        newPassword,
      });

      if (isApiError(response)) {
        setPasswordError(response.error || 'Failed to change password. Please check your current password and try again.');
      } else {
        setPasswordSuccess(true);
        setTimeout(() => setPasswordSuccess(false), 3000);
        setCurrentPassword('');
        setNewPassword('');
        setConfirmPassword('');
      }
    } catch (err) {
      setPasswordError(
        'Failed to change password. Please check your current password and try again.',
      );
      console.error('Password change error:', err);
    } finally {
      setIsPasswordLoading(false);
    }
  };

  if (!user) {
    return (
      <Alert variant="destructive">
        <AlertDescription>
          You must be logged in to view or edit your profile.
        </AlertDescription>
      </Alert>
    );
  }

  // All users can now edit their profiles through the backend API

  const currentPackage = getPackageByLevel(userLevel);

  return (
    <div className="relative">
      {/* Decorative background elements */}
      <div className="absolute -top-8 -left-8 w-32 h-32 bg-gradient-to-br from-purple-400/20 to-pink-400/20 rounded-full blur-2xl pointer-events-none z-0" />
      <div className="absolute -bottom-8 -right-8 w-24 h-24 bg-gradient-to-br from-orange-400/20 to-yellow-400/20 rounded-full blur-xl pointer-events-none z-0" />

      {/* Main content */}
      {!isLoadingLevel && (
        <>
          <div className="absolute top-0 right-0 w-24 h-24 opacity-10 pointer-events-none">
            <div
              className={`w-full h-full bg-gradient-to-br ${levelGradients[userLevel]} rounded-full blur-2xl animate-pulse-slow`}
            ></div>
          </div>
          <div className="absolute bottom-0 left-0 w-16 h-16 opacity-5 pointer-events-none">
            <div
              className={`w-full h-full bg-gradient-to-tr ${levelGradients[userLevel]} rounded-full blur-xl animate-float`}
            ></div>
          </div>
        </>
      )}

      {error && (
        <Alert variant="destructive">
          <AlertDescription>{error}</AlertDescription>
        </Alert>
      )}
      {success && (
        <Alert
          variant="default"
          className="border-green-500 text-green-500 bg-green-50/50 dark:bg-green-900/20"
        >
          <AlertDescription>Profile updated successfully!</AlertDescription>
        </Alert>
      )}
      {passwordError && (
        <Alert variant="destructive">
          <AlertDescription>{passwordError}</AlertDescription>
        </Alert>
      )}
      {passwordSuccess && (
        <Alert
          variant="default"
          className="border-green-500 text-green-500 bg-green-50/50 dark:bg-green-900/20"
        >
          <AlertDescription>Password changed successfully!</AlertDescription>
        </Alert>
      )}

      <div className="flex items-center gap-2 mb-4">
        <div
          className={`p-2 rounded-lg bg-gradient-to-br ${levelGradients[userLevel]} text-white`}
        >
          <User className="h-5 w-5" />
        </div>
        <h4 className="text-lg font-semibold">Profile Information</h4>
      </div>

      {/* Enhanced Profile Header */}
      <div className="relative flex flex-col xs:flex-row items-center gap-3 xs:gap-4 p-3 sm:p-4 rounded-lg bg-background/60 backdrop-blur-sm border border-muted/30">
        <div className="relative flex-shrink-0">
          <Avatar
            className={`w-14 h-14 xs:w-16 xs:h-16 sm:w-20 sm:h-20 ring-2 ring-offset-2 transition-all duration-300 ${
              isLoadingLevel ? 'ring-muted' : `ring-[hsl(var(--primary))]/30`
            }`}
          >
            <AvatarImage src={photoURL} alt={displayName || 'User'} />
            <AvatarFallback
              className={`text-sm xs:text-base sm:text-lg font-semibold ${
                !isLoadingLevel
                  ? `bg-gradient-to-br ${levelGradients[userLevel]} text-white`
                  : ''
              }`}
            >
              {displayName?.charAt(0) || 'U'}
            </AvatarFallback>
          </Avatar>

          {/* Level Badge on Avatar */}
          {!isLoadingLevel && (
            <div
              className={`absolute -bottom-1 -right-1 p-1 xs:p-1.5 rounded-full bg-gradient-to-br ${levelGradients[userLevel]} shadow-lg ring-2 ring-background`}
            >
              <div className="text-white text-xs xs:text-sm">
                {levelIcons[userLevel]}
              </div>
            </div>
          )}
        </div>

        <div className="flex-1 min-w-0 text-center xs:text-left">
          <div className="flex flex-col xs:flex-row xs:items-center gap-1 xs:gap-2 mb-1">
            <h3 className="text-sm xs:text-base sm:text-lg font-semibold truncate">
              {displayName || 'User'}
            </h3>
            {!isLoadingLevel && (
              <Badge
                variant="secondary"
                className={`bg-gradient-to-r ${levelGradients[userLevel]} text-white border-0 font-semibold text-xs w-fit mx-auto xs:mx-0`}
              >
                {userLevel}
              </Badge>
            )}
          </div>
          <p className="text-xs sm:text-sm text-muted-foreground truncate">
            {user.email}
          </p>
          {!isLoadingLevel && currentPackage && (
            <p className="text-xs text-muted-foreground mt-1 hidden xs:block">
              {currentPackage.rankingLimit} stocks access •{' '}
              {currentPackage.name}
            </p>
          )}
        </div>

        {/* Special Effects for Premium Levels */}
        {!isLoadingLevel && canAccessLevel(userLevel, 'GOLD') && (
          <div className="absolute inset-0 pointer-events-none">
            <div className="absolute top-2 right-2 w-1.5 h-1.5 bg-yellow-400 rounded-full animate-ping"></div>
            <div className="absolute bottom-2 left-2 w-1 h-1 bg-yellow-400 rounded-full animate-pulse delay-300"></div>
          </div>
        )}
      </div>
      {/* Enhanced Profile Form */}
      <div className="relative space-y-3 sm:space-y-4 lg:space-y-6">
        <Button
          variant="outline"
          className="mb-2 w-full xs:w-auto h-9 sm:h-10 text-sm sm:text-base"
          onClick={() => setShowProfileForm((v) => !v)}
        >
          {showProfileForm ? 'Hide Name Section' : 'Set Name'}
        </Button>
        {showProfileForm && (
          <form
            onSubmit={handleUpdateProfile}
            className="space-y-3 sm:space-y-4"
          >
            <div className="grid gap-2">
              <Label
                htmlFor="displayName"
                className="flex items-center gap-2 text-sm sm:text-base"
              >
                <User className="h-3 w-3 sm:h-4 sm:w-4 text-muted-foreground" />
                Name
              </Label>
              <Input
                id="displayName"
                value={displayName}
                onChange={(e) => setDisplayName(e.target.value)}
                placeholder="Enter your name"
                className="transition-all duration-200 focus:ring-2 focus:ring-primary/20 h-9 sm:h-10 text-sm sm:text-base"
                error={undefined}
              />
            </div>
            <div className="grid gap-2">
              <Label
                htmlFor="photoURL"
                className="flex items-center gap-2 text-sm sm:text-base"
              >
                <svg
                  className="h-3 w-3 sm:h-4 sm:w-4 text-muted-foreground"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z"
                  />
                </svg>
                Profile Picture URL
              </Label>
              <Input
                id="photoURL"
                value={photoURL}
                onChange={(e) => setPhotoURL(e.target.value)}
                placeholder="Enter URL for profile picture"
                className="transition-all duration-200 focus:ring-2 focus:ring-primary/20 h-9 sm:h-10 text-sm sm:text-base"
                error={undefined}
              />
            </div>
            <Button
              type="submit"
              disabled={isLoading}
              className={`w-full h-9 sm:h-10 bg-gradient-to-r ${levelGradients[userLevel]} hover:opacity-90 text-white border-0 shadow-lg hover:shadow-xl transition-all duration-300 transform hover:scale-105 text-sm sm:text-base flex items-center gap-2`}
            >
              <Save className="h-3 w-3 sm:h-4 sm:w-4" />
              {isLoading ? 'Updating...' : 'Save Changes'}
            </Button>
          </form>
        )}
      </div>

      {/* Enhanced Password Form */}
      <div className="relative space-y-3 sm:space-y-4 lg:space-y-6 pt-3 sm:pt-4 lg:pt-6 border-t border-muted/30">
        <Button
          variant="outline"
          className="mb-2 w-full xs:w-auto h-9 sm:h-10 text-sm sm:text-base flex items-center gap-2"
          onClick={() => setShowPasswordForm((v) => !v)}
        >
          <Key className="h-4 w-4" />
          {showPasswordForm ? 'Hide Change Password' : 'Change Password'}
        </Button>
        {showPasswordForm && (
          <>
            <div className="flex flex-col xs:flex-row items-start xs:items-center gap-2 xs:gap-3 mb-3 sm:mb-4">
              <div
                className={`p-2 rounded-lg bg-gradient-to-br ${levelGradients[userLevel]} text-white`}
              >
                <Key className="h-3 w-3 xs:h-4 xs:w-4 sm:h-5 sm:w-5" />
              </div>
              <h4 className="text-sm xs:text-base sm:text-lg font-semibold">
                Security Settings
              </h4>
            </div>

            <form
              onSubmit={handleChangePassword}
              className="space-y-3 sm:space-y-4"
            >
              <div className="grid gap-2">
                <Label
                  htmlFor="currentPassword"
                  className="flex items-center gap-2 text-sm sm:text-base"
                >
                  <Key className="h-3 w-3 sm:h-4 sm:w-4 text-muted-foreground" />
                  Current Password
                </Label>
                <Input
                  id="currentPassword"
                  type="password"
                  value={currentPassword}
                  onChange={(e) => setCurrentPassword(e.target.value)}
                  placeholder="Enter current password"
                  className="transition-all duration-200 focus:ring-2 focus:ring-primary/20 h-9 sm:h-10 text-sm sm:text-base"
                  error={undefined}
                />
              </div>
              <div className="grid gap-2">
                <Label
                  htmlFor="newPassword"
                  className="flex items-center gap-2 text-sm sm:text-base"
                >
                  <Key className="h-3 w-3 sm:h-4 sm:w-4 text-muted-foreground" />
                  New Password
                </Label>
                <Input
                  id="newPassword"
                  type="password"
                  value={newPassword}
                  onChange={(e) => setNewPassword(e.target.value)}
                  placeholder="Enter new password"
                  className="transition-all duration-200 focus:ring-2 focus:ring-primary/20 h-9 sm:h-10 text-sm sm:text-base"
                  error={undefined}
                />
              </div>
              <div className="grid gap-2">
                <Label
                  htmlFor="confirmPassword"
                  className="flex items-center gap-2 text-sm sm:text-base"
                >
                  <Key className="h-3 w-3 sm:h-4 sm:w-4 text-muted-foreground" />
                  Confirm New Password
                </Label>
                <Input
                  id="confirmPassword"
                  type="password"
                  value={confirmPassword}
                  onChange={(e) => setConfirmPassword(e.target.value)}
                  placeholder="Confirm new password"
                  className="transition-all duration-200 focus:ring-2 focus:ring-primary/20 h-9 sm:h-10 text-sm sm:text-base"
                  error={undefined}
                />
              </div>
              <Button
                type="submit"
                disabled={isPasswordLoading}
                className={`w-full h-9 sm:h-10 bg-gradient-to-r ${levelGradients[userLevel]} hover:opacity-90 text-white border-0 shadow-lg hover:shadow-xl transition-all duration-300 transform hover:scale-105 text-sm sm:text-base flex items-center gap-2`}
              >
                <Key className="h-3 w-3 sm:h-4 sm:w-4" />
                {isPasswordLoading ? 'Changing...' : 'Change Password'}
              </Button>
            </form>
          </>
        )}
      </div>
    </div>
  );
}
