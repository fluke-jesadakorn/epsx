import { AuthForm } from "@/components/auth/AuthForm";
import { handleOAuthCallback } from "../actions/auth-server";
import { Suspense } from "react";
import { LoadingForm } from "@/components/common/LoadingForm";
import { redirect } from "next/navigation";

type SearchParams = { [key: string]: string | string[] | undefined };

interface PageProps {
  params: {};
  searchParams: Promise<SearchParams>;
}

function getParamValue(
  value: string | string[] | undefined
): string | undefined {
  if (typeof value === "string") return value;
  if (Array.isArray(value)) return value[0];
  return undefined;
}

export default async function Page({ searchParams }: PageProps) {
  // Await the searchParams before accessing properties
  const params = await searchParams;

  // Extract search parameters after awaiting
  const code = getParamValue(params.code);
  const state = getParamValue(params.state);
  const token = getParamValue(params.token);
  const error = getParamValue(params.error);

  try {
    if (code && state) {
      const urlParams = new URLSearchParams();
      urlParams.set("code", code);
      urlParams.set("state", state);
      await handleOAuthCallback(urlParams);
      redirect("/home");
    }
  } catch (error) {
    console.error("Auth error:", error);
  }

  // Handle errors from OAuth callback
  const authError = error 
    ? decodeURIComponent(error) 
    : undefined;

  // Show loading state during callback processing
  if (code && state) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <LoadingForm>Processing authentication...</LoadingForm>
      </div>
    );
  }

  // Show auth form
  return (
    <div className="min-h-screen flex items-center justify-center">
      <AuthForm error={authError} />
    </div>
  );
}
