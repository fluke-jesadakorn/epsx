"use client";

import { updateProfile, reauthenticateWithCredential, updatePassword, EmailAuthProvider } from "firebase/auth";
import { useState, useEffect } from "react";

import { Alert, AlertDescription } from "@/components/ui/alert";
import { Avatar, AvatarFallback, AvatarImage } from "@/components/ui/avatar";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useAuth } from "@/context/auth-context";

export function ProfileSettings() {
  const { user } = useAuth();
  const [displayName, setDisplayName] = useState<string>("");
  const [photoURL, setPhotoURL] = useState<string>("");
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState<boolean>(false);
  const [currentPassword, setCurrentPassword] = useState<string>("");
  const [newPassword, setNewPassword] = useState<string>("");
  const [confirmPassword, setConfirmPassword] = useState<string>("");
  const [isPasswordLoading, setIsPasswordLoading] = useState<boolean>(false);
  const [passwordError, setPasswordError] = useState<string | null>(null);
  const [passwordSuccess, setPasswordSuccess] = useState<boolean>(false);

  useEffect(() => {
    if (user) {
      setDisplayName(user.displayName || "");
      setPhotoURL(user.photoURL || "");
    }
  }, [user]);

  const handleUpdateProfile = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!user) return;

    setIsLoading(true);
    setError(null);
    setSuccess(false);

    try {
      await updateProfile(user, {
        displayName,
        photoURL: photoURL || "",
      });
      setSuccess(true);
      setTimeout(() => setSuccess(false), 3000);
    } catch (err) {
      setError("Failed to update profile. Please try again.");
      console.error("Profile update error:", err);
    } finally {
      setIsLoading(false);
    }
  };

  const handleChangePassword = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!user) return;

    if (newPassword !== confirmPassword) {
      setPasswordError("New passwords do not match.");
      return;
    }

    setIsPasswordLoading(true);
    setPasswordError(null);
    setPasswordSuccess(false);

    try {
      // Reauthenticate user before changing password
      const credential = EmailAuthProvider.credential(user.email || "", currentPassword);
      await reauthenticateWithCredential(user, credential);
      
      // Update password
      await updatePassword(user, newPassword);
      setPasswordSuccess(true);
      setTimeout(() => setPasswordSuccess(false), 3000);
      setCurrentPassword("");
      setNewPassword("");
      setConfirmPassword("");
    } catch (err) {
      setPasswordError("Failed to change password. Please check your current password and try again.");
      console.error("Password change error:", err);
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

  return (
    <div className="space-y-6">
      {error && (
        <Alert variant="destructive">
          <AlertDescription>{error}</AlertDescription>
        </Alert>
      )}
      {success && (
        <Alert variant="default" className="border-green-500 text-green-500">
          <AlertDescription>Profile updated successfully!</AlertDescription>
        </Alert>
      )}
      {passwordError && (
        <Alert variant="destructive">
          <AlertDescription>{passwordError}</AlertDescription>
        </Alert>
      )}
      {passwordSuccess && (
        <Alert variant="default" className="border-green-500 text-green-500">
          <AlertDescription>Password changed successfully!</AlertDescription>
        </Alert>
      )}
      <div className="flex items-center gap-4">
        <Avatar className="w-20 h-20">
          <AvatarImage src={photoURL} alt={displayName || "User"} />
          <AvatarFallback>{displayName?.charAt(0) || "U"}</AvatarFallback>
        </Avatar>
        <div>
          <h3 className="text-lg font-medium">{displayName || "User"}</h3>
          <p className="text-sm text-muted-foreground">{user.email}</p>
        </div>
      </div>
      <form onSubmit={handleUpdateProfile} className="space-y-4">
        <div className="grid gap-2">
          <Label htmlFor="displayName">Name</Label>
          <Input
            id="displayName"
            value={displayName}
            onChange={(e) => setDisplayName(e.target.value)}
            placeholder="Enter your name"
          />
        </div>
        <div className="grid gap-2">
          <Label htmlFor="photoURL">Profile Picture URL</Label>
          <Input
            id="photoURL"
            value={photoURL}
            onChange={(e) => setPhotoURL(e.target.value)}
            placeholder="Enter URL for profile picture"
          />
        </div>
        <Button type="submit" disabled={isLoading} className="border border-primary">
          {isLoading ? "Updating..." : "Save Changes"}
        </Button>
      </form>
      
      <div className="mt-8">
        <h3 className="text-lg font-medium mb-4">Change Password</h3>
        <form onSubmit={handleChangePassword} className="space-y-4">
          <div className="grid gap-2">
            <Label htmlFor="currentPassword">Current Password</Label>
            <Input
              id="currentPassword"
              type="password"
              value={currentPassword}
              onChange={(e) => setCurrentPassword(e.target.value)}
              placeholder="Enter current password"
            />
          </div>
          <div className="grid gap-2">
            <Label htmlFor="newPassword">New Password</Label>
            <Input
              id="newPassword"
              type="password"
              value={newPassword}
              onChange={(e) => setNewPassword(e.target.value)}
              placeholder="Enter new password"
            />
          </div>
          <div className="grid gap-2">
            <Label htmlFor="confirmPassword">Confirm New Password</Label>
            <Input
              id="confirmPassword"
              type="password"
              value={confirmPassword}
              onChange={(e) => setConfirmPassword(e.target.value)}
              placeholder="Confirm new password"
            />
          </div>
          <Button type="submit" disabled={isPasswordLoading} className="border border-primary">
            {isPasswordLoading ? "Changing..." : "Change Password"}
          </Button>
        </form>
      </div>
    </div>
  );
}
