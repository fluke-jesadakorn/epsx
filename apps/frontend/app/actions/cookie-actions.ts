"use server";

import { cookies } from "next/headers";

const COOKIE_NAME = "cookie-consent";
const ONE_YEAR = 60 * 60 * 24 * 365;

export type ConsentStatus = "accepted" | "rejected" | null;

export async function getCookieConsent(): Promise<ConsentStatus> {
  const cookieStore = await cookies();
  const consent = cookieStore.get(COOKIE_NAME);
  return (consent?.value as ConsentStatus) ?? null;
}

export async function setCookieConsent(status: "accepted" | "rejected") {
  const cookieStore = await cookies();
  cookieStore.set(COOKIE_NAME, status, {
    httpOnly: true,
    secure: process.env.NODE_ENV === "production",
    sameSite: "lax",
    path: "/",
    maxAge: ONE_YEAR,
  });
}
