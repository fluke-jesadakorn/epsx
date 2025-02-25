"use server";

/**
 * This file contains server actions for stock data fetching.
 * The "use server" directive is placed at the top of the file to ensure all exports
 * are treated as server actions when imported by client components.
 *
 * Future Considerations:
 * - Add request caching for improved performance
 * - Implement rate limiting for API requests
 * - Add error handling middleware
 * - Add request validation
 * - Implement data transformation utilities
 * - Add response compression
 * - Consider implementing GraphQL for more flexible data fetching
 * - Add metrics and monitoring for API calls
 * - Implement circuit breaker pattern for API failures
 * - Add request retries with exponential backoff
 */

import {
  EpsGrowthRankingResponse,
  EpsGrowthData,
} from "@/types/epsGrowthRanking";

/**
 * Fetches EPS growth ranking data from the API
 *
 * @param params - Query parameters for pagination
 * @param params.limit - Number of records to return (default: 20)
 * @param params.skip - Number of records to skip (default: 0)
 *
 * Future Considerations:
 * - Add market filter support (e.g., filter by TYO, BOM, OTC)
 * - Add date range filter for last_report_date
 * - Add minimum/maximum EPS filter
 * - Add sorting options (e.g., by eps, eps_growth, company_name)
 * - Add company search functionality
 * - Implement data caching with configurable TTL
 * - Add market cap and sector filters
 * - Consider adding data export functionality (CSV, Excel)
 *
 * @returns Promise<EpsGrowthRankingResponse>
 */
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

  try {
    const response = await fetch(
      `${baseUrl}/market/financials/eps-growth?limit=${limit}&skip=${skip}`,
      {
        method: "GET",
        headers: {
          "Content-Type": "application/json",
        },
        // next: {
        //   revalidate: 300, // Cache for 5 minutes
        // },
      }
    );

    if (!response.ok) {
      console.error(`HTTP error! status: ${response}`);
    }

    const data = await response.json();
    console.log("Data:", data);

    // Transform the response to match expected structure
    const validatedData = data.data.map(
      (item: any): EpsGrowthData => ({
        symbol: item.symbol,
        company_name: item.company_name,
        market_code: item.market_code,
        eps_diluted: item.eps_diluted,
        eps_growth: item.eps_growth,
        previous_eps_diluted: item.previous_eps_diluted,
        report_date: item.report_date,
        quarter: item.quarter,
        year: item.year,
      })
    );

    // Construct response with metadata
    return {
      data: validatedData,
      metadata: {
        total: data.metadata.total,
        page: data.metadata.page,
        limit: data.metadata.limit,
        totalPages: data.metadata.totalPages,
        skip: data.metadata.skip,
      },
    };
  } catch (error) {
    console.error("Failed to fetch EPS growth ranking:", error);
    return {
      data: [],
      metadata: {
        total: 0,
        page: 1,
        limit: limit,
        totalPages: 0,
        skip: skip,
      },
    };
  }
}
