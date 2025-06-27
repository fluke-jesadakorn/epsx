"use client";

import { useState, useEffect } from "react";
import { useAuth } from "@/context/auth-context";
import { GoogleAuthProvider, linkWithPopup, unlink } from "firebase/auth";
import { Button } from "@/components/ui/button";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Badge } from "@/components/ui/badge";

export function AuthProviders() {
  const { user } = useAuth();
  const [providers, setProviders] = useState<string[]>([]);
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (user) {
      const providerIds = user.providerData.map((data) => data.providerId);
      setProviders(providerIds);
    }
  }, [user]);

  const handleLinkGoogle = async () => {
    if (!user) return;
    setIsLoading(true);
    setError(null);

    try {
      const provider = new GoogleAuthProvider();
      await linkWithPopup(user, provider);
      // Refresh providers list
      const updatedProviders = user.providerData.map((data) => data.providerId);
      setProviders(updatedProviders);
    } catch (err) {
      setError("Failed to link Google account. Please try again.");
      console.error("Link Google error:", err);
    } finally {
      setIsLoading(false);
    }
  };

  const handleUnlinkProvider = async (providerId: string) => {
    if (!user) return;
    setIsLoading(true);
    setError(null);

    try {
      // Ensure user has at least one provider after unlinking
      if (providers.length > 1) {
        await unlink(user, providerId);
        // Refresh providers list
        const updatedProviders = user.providerData.map((data) => data.providerId);
        setProviders(updatedProviders);
      } else {
        setError("You must have at least one authentication method linked.");
      }
    } catch (err) {
      setError(`Failed to unlink ${providerId}. Please try again.`);
      console.error(`Unlink ${providerId} error:`, err);
    } finally {
      setIsLoading(false);
    }
  };

  if (!user) {
    return (
      <Alert variant="destructive">
        <AlertDescription>
          You must be logged in to manage authentication providers.
        </AlertDescription>
      </Alert>
    );
  }

  const providerNames: Record<string, string> = {
    "google.com": "Google",
    "password": "Email/Password",
  };

  const canLinkGoogle = !providers.includes("google.com");

  return (
    <div className="space-y-4">
      {error && (
        <Alert variant="destructive">
          <AlertDescription>{error}</AlertDescription>
        </Alert>
      )}
      <div>
        <h3 className="text-lg font-medium mb-2">Linked Providers</h3>
        <div className="flex flex-wrap gap-2">
          {providers.length > 0 ? (
            providers.map((providerId) => (
              <div key={providerId} className="flex items-center gap-2">
                <Badge variant="outline">{providerNames[providerId] || providerId}</Badge>
                {providers.length > 1 && (
                  <Button
                    variant="destructive"
                    size="sm"
                    onClick={() => handleUnlinkProvider(providerId)}
                    disabled={isLoading}
                  >
                    Unlink
                  </Button>
                )}
              </div>
            ))
          ) : (
            <p className="text-sm text-muted-foreground">No providers linked.</p>
          )}
        </div>
      </div>
      <div>
        <h3 className="text-lg font-medium mb-2">Link New Provider</h3>
        {canLinkGoogle ? (
          <Button
            onClick={handleLinkGoogle}
            disabled={isLoading}
          >
            {isLoading ? "Linking..." : "Link Google Account"}
          </Button>
        ) : (
          <p className="text-sm text-muted-foreground">Google account already linked.</p>
        )}
      </div>
    </div>
  );
}
