import { cookies } from "next/headers";
import { NextResponse } from "next/server";
import { auth } from "@/lib/firebase-admin";

export const dynamic = "force-dynamic";

export interface UserSession {
  user: {
    uid: string;
    email: string | null | undefined;
    emailVerified: boolean;
    displayName: string | null | undefined;
    photoURL: string | null | undefined;
    customClaims?: {
      roles?: string[];
      [key: string]: unknown;
    };
  } | null;
}

export async function GET(): Promise<NextResponse<UserSession>> {
  try {
    const cookieStore = await cookies();
    const sessionCookie = cookieStore.get("session")?.value;

    if (!sessionCookie) {
      return NextResponse.json({ user: null }, { status: 401 });
    }

    const decodedClaim = await auth().verifySessionCookie(sessionCookie, true);

    if (!decodedClaim) {
      return NextResponse.json({ user: null }, { status: 401 });
    }

    const user = await auth().getUser(decodedClaim.uid);

    return NextResponse.json({
      user: {
        uid: user.uid,
        email: user.email ?? null,
        emailVerified: user.emailVerified,
        displayName: user.displayName ?? null,
        photoURL: user.photoURL ?? null,
        customClaims: user.customClaims,
      },
    });
  } catch (error) {
    console.error("Session verification error:", error);
    return NextResponse.json({ user: null }, { status: 401 });
  }
}
