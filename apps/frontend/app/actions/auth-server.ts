"use server";

import { cookies } from "next/headers";
import { redirect } from "next/navigation";
import { UserRole } from "@/types/auth/roles";
import { TokenFeature, Permission } from "@/types/auth/features";

interface AuthResponse {
  token: string;
  email: string;
  role: UserRole;
  tokenBalance: number;
  features: TokenFeature[];
  permissions: Permission[];
  redirectUrl?: string;
}

export async function verifyAuth() {
  const cookieStore = await cookies();
  const sessionToken = cookieStore.get("__session");
  const email = cookieStore.get("email");
  const role = cookieStore.get("role");
  const tokenBalance = cookieStore.get("token_balance");
  const features = cookieStore.get("features");
  const permissions = cookieStore.get("permissions");

  if (!sessionToken || !email || !role) {
    return {
      isLoggedIn: false,
      userEmail: null,
      role: UserRole.GUEST,
      tokenBalance: 0,
      features: [],
      permissions: [],
      isAdmin: false,
    };
  }

  return {
    isLoggedIn: true,
    userEmail: email.value,
    role: role.value as UserRole,
    tokenBalance: tokenBalance ? parseInt(tokenBalance.value, 10) : 0,
    features: features ? (JSON.parse(features.value) as TokenFeature[]) : [],
    permissions: permissions
      ? (JSON.parse(permissions.value) as Permission[])
      : [],
    isAdmin: role.value === UserRole.ADMINISTRATOR,
  };
}

interface OAuthProvider {
  providerId: string;
}

export async function signInWithOAuth(provider: OAuthProvider) {
  try {
    const response = await fetch(
      `${process.env.NEXT_PUBLIC_API_URL}/api/auth/oauth/init`,
      {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          provider: provider.providerId,
        }),
      }
    );

    if (!response.ok) {
      throw new Error("Failed to initiate OAuth flow");
    }

    const { url } = await response.json();
    window.location.href = url;
  } catch (error) {
    console.error("OAuth initialization error:", error);
    throw error;
  }
}

export async function signUpWithEmailPassword({
  email,
  password,
}: {
  email: string;
  password: string;
}) {
  if (!email || !password) {
    throw new Error("Email and password are required");
  }

  try {
    const response = await fetch(
      `${process.env.NEXT_PUBLIC_API_URL}/api/auth/register`,
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
      throw new Error("Registration failed");
    }

    return response.json();
  } catch (error) {
    console.error("Registration error:", error);
    throw error;
  }
}

export async function signOut() {
  try {
    const cookieStore = await cookies();
    cookieStore.delete("__session");
    cookieStore.delete("email");
    cookieStore.delete("role");
    cookieStore.delete("token_balance");
    cookieStore.delete("features");
    cookieStore.delete("permissions");

    redirect("/login");
  } catch (error) {
    console.error("Sign out error:", error);
    throw error;
  }
}

export async function handleOAuthCallback(params: URLSearchParams) {
  const code = params.get("code");
  const state = params.get("state");

  if (!code || !state) {
    throw new Error("Invalid OAuth callback parameters");
  }

  try {
    const response = await fetch(
      `${process.env.NEXT_PUBLIC_API_URL}/api/auth/oauth/callback`,
      {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          code,
          state,
        }),
      }
    );

    if (!response.ok) {
      throw new Error("OAuth authentication failed");
    }

    const data: AuthResponse = await response.json();

    const cookieOptions = {
      httpOnly: true,
      secure: process.env.NODE_ENV === "production",
      sameSite: "lax" as const,
      path: "/",
      maxAge: 60 * 60 * 24 * 7, // 1 week
    };

    const cookieStore = await cookies();
    cookieStore.set("__session", data.token, cookieOptions);
    cookieStore.set("email", data.email, cookieOptions);
    cookieStore.set("role", data.role, cookieOptions);
    cookieStore.set(
      "token_balance",
      data.tokenBalance.toString(),
      cookieOptions
    );
    cookieStore.set("features", JSON.stringify(data.features), cookieOptions);
    cookieStore.set(
      "permissions",
      JSON.stringify(data.permissions),
      cookieOptions
    );

    return data;
  } catch (error) {
    console.error("OAuth callback error:", error);
    throw error;
  }
}

export async function listUsers() {
  try {
    const response = await fetch(
      `${process.env.NEXT_PUBLIC_API_URL}/api/auth/roles`,
      {
        method: "GET",
        headers: {
          "Content-Type": "application/json",
        },
      }
    );

    if (!response.ok) {
      throw new Error("Failed to fetch users");
    }

    interface UserResponse {
      uid: string;
      email: string;
      role: UserRole;
    }
    
    const users = await response.json();
    return users.map((user: UserResponse) => ({
      userId: user.uid,
      email: user.email,
      role: user.role,
    }));
  } catch (error) {
    console.error("Error fetching users:", error);
    throw error;
  }
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

    const data: AuthResponse = await response.json();

    const cookieOptions = {
      httpOnly: true,
      secure: process.env.NODE_ENV === "production",
      sameSite: "lax" as const,
      path: "/",
      maxAge: 60 * 60 * 24 * 7, // 1 week
    };

    const cookieStore = await cookies();
    cookieStore.set("__session", data.token, cookieOptions);
    cookieStore.set("email", data.email, cookieOptions);
    cookieStore.set("role", data.role, cookieOptions);
    cookieStore.set(
      "token_balance",
      data.tokenBalance.toString(),
      cookieOptions
    );
    cookieStore.set("features", JSON.stringify(data.features), cookieOptions);
    cookieStore.set(
      "permissions",
      JSON.stringify(data.permissions),
      cookieOptions
    );

    redirect(redirectTo);
  } catch (error) {
    throw error;
  }
}
