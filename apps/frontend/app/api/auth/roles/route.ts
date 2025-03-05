import { auth } from "@/lib/firebase-admin";
import { UserRole } from "@/constants/roles";
import { NextResponse } from "next/server";
import { UserRecord } from "firebase-admin/auth";

export async function GET() {
  try {
    const usersResult = await auth().listUsers();
    const users = usersResult.users.map((user: UserRecord) => ({
      userId: user.uid,
      email: user.email,
      role: (user.customClaims?.role as UserRole) || UserRole.PUBLIC,
    }));

    return NextResponse.json(users);
  } catch (error) {
    console.error("Error fetching users:", error);
    return NextResponse.json(
      { error: "Failed to fetch users" },
      { status: 500 }
    );
  }
}
