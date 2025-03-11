"use server";

import { cookies } from "next/headers";
import { UserRole} from "@/types/auth/roles";
import { TokenFeature, Permission } from "@/types/auth/features";

interface User {
  userId: string;
  email?: string;
  role: UserRole;
  tokenBalance: number;
  features: TokenFeature[];
  permissions: Permission[];
}

export async function fetchUserDetails() {
  try {
    const cookieStore = await cookies();
    const sessionToken = cookieStore.get("__session");
    const email = cookieStore.get("email");
    const role = cookieStore.get("role");

    if (!sessionToken || !email || !role) {
      throw new Error("Unauthorized access. Please login.");
    }

    const response = await fetch(
      `${process.env.NEXT_PUBLIC_API_URL}/auth/roles`,
      {
        method: "GET",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${sessionToken.value}`,
        },
        credentials: "include",
      }
    );

    if (!response.ok) {
      const errorData = await response.text();
      throw new Error(`Failed to fetch users: ${errorData}`);
    }

    const data = await response.json();
    return data.users as User[];
  } catch (error) {
    console.error("Error fetching user details:", error);
    throw error;
  }
}

export async function updateUserRole(userId: string, role: string) {
  try {
    const cookieStore = await cookies();
    const sessionToken = cookieStore.get("__session");

    if (!sessionToken) {
      throw new Error("Unauthorized access. Please login.");
    }

    const response = await fetch(
      `${process.env.NEXT_PUBLIC_API_URL}/auth/roles/${userId}`,
      {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${sessionToken.value}`,
        },
        credentials: "include",
        body: JSON.stringify({ role }),
      }
    );

    if (!response.ok) {
      const errorData = await response.text();
      throw new Error(`Failed to update user role: ${errorData}`);
    }

    return await response.json();
  } catch (error) {
    console.error("Error updating user role:", error);
    throw error;
  }
}
