"use server";

import { cookies } from "next/headers";
import { apiClient } from "@/lib/api-client";

interface ApiResponse<T> {
  data: T;
  status: number;
}

export async function checkSession() {
  const cookieStore = await cookies();
  const session = await cookieStore.get("__session")?.value;
  return {
    isAuthenticated: !!session,
  };
}

interface RoleAssignmentResponse {
  success: boolean;
  message?: string;
}

interface UserRoleResponse {
  role: string;
  permissions: string[];
}

interface UserListResponse {
  users: Array<{
    id: string;
    email: string;
    role: string;
  }>;
}

export async function logout() {
  try {
    // Call backend to clear all cookies
    await apiClient.post(
      "/auth/logout",
      {},
      {
        headers: {
          "Content-Type": "application/json",
        },
      }
    );

    // Clear client-side cookies as well
    const cookieStore = await cookies();
    await Promise.all(
      ["__session", "uid", "email", "role"].map(async (name) => {
        cookieStore.set(name, "", {
          maxAge: 0,
          path: "/",
          expires: new Date(0),
        });
      })
    );

    return {
      success: true,
      redirectUrl: "/login",
    };
  } catch (error) {
    console.error("Logout error:", error);
    throw new Error("Failed to logout");
  }
}

export async function assignRole(userId: string, role: string) {
  const cookieStore = await cookies();
  const session = await cookieStore.get("__session")?.value;

  if (!session) {
    throw new Error("Unauthorized");
  }

  try {
    const response = await apiClient.post<ApiResponse<RoleAssignmentResponse>>(
      "/auth/roles/assign",
      {
        userId,
        role,
      },
      {
        headers: {
          Authorization: `Bearer ${session}`,
        },
      }
    );

    return response.data;
  } catch (error) {
    throw new Error("Failed to assign role" + error);
  }
}

export async function getUserRole(userId: string) {
  const cookieStore = await cookies();
  const session = await cookieStore.get("__session")?.value;

  if (!session) {
    throw new Error("Unauthorized");
  }

  try {
    const response = await apiClient.get<ApiResponse<UserRoleResponse>>(
      `/auth/roles/${userId}`,
      {
        headers: {
          Authorization: `Bearer ${session}`,
        },
      }
    );

    return response.data;
  } catch (error) {
    console.error("Get user role error:", error);
    throw new Error("Failed to get user role");
  }
}

export async function listUsers(maxResults?: number) {
  const cookieStore = await cookies();
  const session = await cookieStore.get("__session")?.value;

  if (!session) {
    throw new Error("Unauthorized");
  }

  try {
    const response = await apiClient.get<ApiResponse<UserListResponse>>(
      "/auth/roles",
      {
        params: { maxResults },
        headers: {
          Authorization: `Bearer ${session}`,
        },
      }
    );

    return response.data;
  } catch (error) {
    throw new Error("Failed to list users" + error);
  }
}
