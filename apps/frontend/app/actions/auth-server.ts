"use server";

import { cookies } from "next/headers";
import { redirect } from "next/navigation";

interface AuthResponseData {
  token: string;
  email: string;
  role: string;
  redirectUrl?: string;
}

interface OAuthResponseData {
  authUrl: string;
}

export async function signInWithEmail(formData: FormData) {
  const email = formData.get("email") as string;
  const password = formData.get("password") as string;
  const redirectTo = (formData.get("redirectTo") as string) || "/home";

  if (!email || !password) {
    throw new Error("Email and password are required");
  }

  try {
    const response = await fetch(
      `${process.env.NEXT_PUBLIC_API_URL}/api/auth/session`,
      {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          email,
          password,
        }),
      }
    );

    if (!response.ok) {
      throw new Error("Invalid credentials");
    }

    const data: AuthResponseData = await response.json();

    const cookieStore = await cookies();
    cookieStore.set("__session", data.token, {
      httpOnly: true,
      secure: process.env.NODE_ENV === "production",
      sameSite: "lax",
      path: "/",
      maxAge: 60 * 60 * 24 * 7, // 1 week
    });
    cookieStore.set("email", data.email, {
      httpOnly: true,
      secure: process.env.NODE_ENV === "production",
      sameSite: "lax",
      path: "/",
      maxAge: 60 * 60 * 24 * 7, // 1 week
    });
    cookieStore.set("role", data.role, {
      httpOnly: true,
      secure: process.env.NODE_ENV === "production",
      sameSite: "lax",
      path: "/",
      maxAge: 60 * 60 * 24 * 7, // 1 week
    });

    redirect(redirectTo);
  } catch (error) {
    throw error;
  }
}

export async function signInWithOAuth(formData: FormData) {
  const provider = formData.get("provider") as string;
  const redirectTo = (formData.get("redirectTo") as string) || "/home";

  if (!provider) {
    throw new Error("Provider is required");
  }

  try {
    const response = await fetch(
      `${process.env.NEXT_PUBLIC_API_URL}/auth/oauth`,
      {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          provider,
          redirectUrl: redirectTo,
        }),
      }
    );

    if (!response.ok) {
      throw new Error("Failed to initialize OAuth");
    }

    const data: OAuthResponseData = await response.json();
    if (!data.authUrl) {
      throw new Error("No auth URL provided");
    }

    // Store the redirectTo URL in session storage to use after callback
    if (typeof window !== "undefined") {
      sessionStorage.setItem("oauth_redirect", redirectTo);
    }

    // Redirect to provider's consent page
    redirect(data.authUrl);
  } catch (error) {
    console.error("OAuth error:", error);
    throw error;
  }
}

export async function handleOAuthCallback(searchParams: URLSearchParams) {
  const code = searchParams.get("code");
  const state = searchParams.get("state");
  const error = searchParams.get("error");

  if (error) {
    throw new Error(`OAuth error: ${error}`);
  }

  if (!code || !state) {
    throw new Error("Invalid OAuth callback");
  }

  try {
    const callbackUrl = `${process.env.NEXT_PUBLIC_API_URL}/auth/oauth/callback?code=${encodeURIComponent(code)}&state=${encodeURIComponent(state)}`;
    const response = await fetch(callbackUrl);

    if (!response.ok) {
      throw new Error("Failed to complete OAuth");
    }

    const data: AuthResponseData = await response.json();

    const cookieStore = await cookies();
    cookieStore.set("__session", data.token, {
      httpOnly: true,
      secure: process.env.NODE_ENV === "production",
      sameSite: "lax",
      path: "/",
      maxAge: 60 * 60 * 24 * 7, // 1 week
    });
    cookieStore.set("email", data.email, {
      httpOnly: true,
      secure: process.env.NODE_ENV === "production",
      sameSite: "lax",
      path: "/",
      maxAge: 60 * 60 * 24 * 7, // 1 week
    });
    cookieStore.set("role", data.role, {
      httpOnly: true,
      secure: process.env.NODE_ENV === "production",
      sameSite: "lax",
      path: "/",
      maxAge: 60 * 60 * 24 * 7, // 1 week
    });

    // Get stored redirect URL or default to /home
    let redirectUrl = "/home";
    if (typeof window !== "undefined") {
      redirectUrl = sessionStorage.getItem("oauth_redirect") || "/home";
      sessionStorage.removeItem("oauth_redirect");
    }

    redirect(redirectUrl);
  } catch (error) {
    console.error("OAuth callback error:", error);
    throw error;
  }
}

export async function signOut() {
  try {
    const cookieStore = await cookies();
    cookieStore.delete("__session");
    cookieStore.delete("email");
    cookieStore.delete("role");

    redirect("/login");
  } catch (error) {
    console.error("Sign out error:", error);
    throw error;
  }
}
