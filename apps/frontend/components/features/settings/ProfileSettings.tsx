"use client";

import { updateProfile } from "firebase/auth";
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
        <Button type="submit" disabled={isLoading}>
          {isLoading ? "Updating..." : "Save Changes"}
        </Button>
      </form>
    </div>
  );
}
