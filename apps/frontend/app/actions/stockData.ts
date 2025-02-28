"use server";

import { EpsGrowthRankingResponse } from "@/types/epsGrowthRanking";

export async function fetchEpsGrowthRanking({
  limit = 3,
  skip = 0,
}: {
  limit?: number;
  skip?: number;
}): Promise<EpsGrowthRankingResponse> {
  const baseUrl = process.env.NEXT_PUBLIC_API_URL;

  if (!baseUrl) {
    throw new Error(
      "NEXT_PUBLIC_API_URL is not defined in environment variables"
    );
  }

  const response = await fetch(
    `${baseUrl}/market/financials/eps-growth?limit=${limit}&skip=${skip}`,
    {
      method: "GET",
      headers: {
        "Content-Type": "application/json",
      },
      next: {
        revalidate: 300, // Cache for 5 minutes
      },
    }
  );

  if (!response.ok) {
    throw new Error(`HTTP error! status: ${response.status}`);
  }

  return response.json();
}
