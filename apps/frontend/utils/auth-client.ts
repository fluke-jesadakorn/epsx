import { 
  GoogleAuthProvider, 
  signInWithPopup, 
  signInWithEmailAndPassword as fbSignInWithEmailAndPassword,
  createUserWithEmailAndPassword as fbCreateUserWithEmailAndPassword,
  signOut as fbSignOut
} from "firebase/auth";
import { auth as clientAuth } from "@/lib/firebase-client";

export type AuthResponse = {
  success: boolean;
  error?: string;
  redirectUrl?: string;
};

export async function signInWithOAuth(provider: "google" | "azure") {
  try {
    if (provider === "google") {
      const googleProvider = new GoogleAuthProvider();
      googleProvider.setCustomParameters({
        prompt: 'select_account',
        // Add additional OAuth scopes if needed
        scope: 'email profile'
      });
      
      // Use try-catch specifically for popup issues
      try {
        const result = await signInWithPopup(clientAuth, googleProvider);
        const idToken = await result.user.getIdToken();
        
        // Send token to server endpoint for session creation
        const response = await fetch('/api/auth/session', {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
            'Accept': 'application/json'
          },
          credentials: 'include',
          body: JSON.stringify({ idToken }),
        });
        
        if (!response.ok) {
          const errorData = await response.json().catch(() => ({}));
          throw new Error(
            `Failed to create session: ${response.status} - ${errorData.error || response.statusText}`
          );
        }

        const data = await response.json();
        return {
          success: true,
          redirectUrl: data.redirectUrl || "/home",
        };
      } catch (popupError) {
        console.error("Popup error:", popupError);
        throw popupError;
      }
    }

    throw new Error(`Provider ${provider} not supported`);
  } catch (error: any) {
    console.error("OAuth error:", error);
    return {
      success: false,
      error: error.message || "Failed to sign in with provider",
    };
  }
}

export async function signInWithEmailPassword(email: string, password: string) {
  try {
    const userCredential = await fbSignInWithEmailAndPassword(
      clientAuth,
      email,
      password
    );
    const idToken = await userCredential.user.getIdToken();
    
    // Send token to server endpoint for session creation
    const response = await fetch('/api/auth/session', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Accept': 'application/json'
      },
      credentials: 'include',
      body: JSON.stringify({ idToken }),
    });
    
    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      throw new Error(
        `Failed to create session: ${response.status} - ${errorData.error || response.statusText}`
      );
    }

    const data = await response.json();
    return {
      success: true,
      redirectUrl: data.redirectUrl || "/home",
    };
  } catch (error: any) {
    console.error("Sign in error:", error);
    return {
      success: false,
      error: error.message || "Failed to sign in",
    };
  }
}

export async function signUpWithEmailPassword(email: string, password: string) {
  try {
    const userCredential = await fbCreateUserWithEmailAndPassword(
      clientAuth,
      email,
      password
    );
    const idToken = await userCredential.user.getIdToken();
    
    // Send token to server endpoint for session creation
    const response = await fetch('/api/auth/session', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Accept': 'application/json'
      },
      credentials: 'include',
      body: JSON.stringify({ idToken }),
    });
    
    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      throw new Error(
        `Failed to create session: ${response.status} - ${errorData.error || response.statusText}`
      );
    }

    const data = await response.json();
    return {
      success: true,
      redirectUrl: data.redirectUrl || "/home",
    };
  } catch (error: any) {
    console.error("Sign up error:", error);
    return {
      success: false,
      error: error.message || "Failed to create account",
    };
  }
}

export async function signOut() {
  try {
    await fbSignOut(clientAuth);
    
    // Call server endpoint to clear session
    const response = await fetch('/api/auth/logout', {
      method: 'POST',
      headers: {
        'Accept': 'application/json'
      },
      credentials: 'include'
    });
    
    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      throw new Error(
        `Failed to clear session: ${response.status} - ${errorData.error || response.statusText}`
      );
    }

    return {
      success: true,
      redirectUrl: "/login",
    };
  } catch (error: any) {
    console.error("Sign out error:", error);
    return {
      success: false,
      error: error.message || "Failed to sign out",
    };
  }
}
