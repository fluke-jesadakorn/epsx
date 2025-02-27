import StockRankTable from "@/components/home/StockRankTable";
import HeroSection from "@/components/home/HeroSection";
import ClientEpsCardSection from "@/components/home/ClientEpsCardSection";
import { fetchEpsGrowthRanking } from "@/app/actions/stockData";
import { Suspense } from "react";

/**
 * Home page component that displays various sections including EPS growth ranking
 *
 * Future Features:
 * - Add error boundary component for better error handling
 * - Add error reporting to external service
 * - Add data prefetching for improved performance
 * - Add SEO optimization
 * - Add skeleton loading UI
 * - Add market filter functionality
 * - Add date range filtering
 * - Add sorting options
 * - Add responsive design improvements
 * - Add data export functionality
 */
async function getInitialEpsData() {
  try {
    const response = await fetchEpsGrowthRanking({ limit: 3, skip: 0 });

    // Validate response structure and data
    if (!response?.data || !Array.isArray(response.data)) {
      console.error("Invalid EPS data structure:", response);
      return {
        data: [],
        total: 0,
      };
    }

    // Filter out any invalid items
    const validData = response.data.filter(
      (item) =>
        item &&
        typeof item === "object" &&
        "symbol" in item &&
        "company_name" in item
    );

    return {
      data: validData,
      total: response.metadata?.total || validData.length,
    };
  } catch (error) {
    console.error("Failed to fetch initial EPS growth ranking:", error);
    return {
      data: [],
      total: 0,
    };
  }
}

export default async function HomeView() {
  const { data: initialData, total: initialTotal } = await getInitialEpsData();

  return (
    <div className="flex justify-center items-center min-h-screen">
      <div className="w-[83.333%]"> {/* equivalent to col-20/24 */}
        <div className="flex flex-col gap-10 py-6">
          <div className="w-full">
            <Suspense fallback={
              <div className="flex justify-center">
                <div className="animate-spin h-8 w-8 border-4 border-blue-500 border-t-transparent rounded-full"/>
              </div>
            }>
              <HeroSection className="min-h-[400px]" />
            </Suspense>
          </div>
          
          <div className="w-full">
            <Suspense fallback={
              <div className="flex justify-center">
                <div className="animate-spin h-8 w-8 border-4 border-blue-500 border-t-transparent rounded-full"/>
              </div>
            }>
              <ClientEpsCardSection
                className="min-h-[300px]"
                initialData={initialData}
                initialTotal={initialTotal}
              />
            </Suspense>
          </div>
          
          <div className="w-full">
            <Suspense fallback={
              <div className="flex justify-center">
                <div className="animate-spin h-8 w-8 border-4 border-blue-500 border-t-transparent rounded-full"/>
              </div>
            }>
              <StockRankTable className="min-h-[500px]" />
            </Suspense>
          </div>
          
          <div className="w-full">
            <Suspense fallback={
              <div className="flex justify-center">
                <div className="animate-spin h-8 w-8 border-4 border-blue-500 border-t-transparent rounded-full"/>
              </div>
            }>
              {/* <ChatSection /> */}
            </Suspense>
          </div>
        </div>
      </div>
    </div>
  );
}
