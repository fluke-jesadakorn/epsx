import StockRankTable from "@/components/home/StockRankTable";
import HeroSection from "@/components/home/HeroSection";
import ClientEpsCardSection from "@/components/home/ClientEpsCardSection";
import { fetchEpsGrowthRanking } from "@/app/actions/stockData";
import { ErrorBoundary } from "@/components/common/ErrorBoundary";
import { Suspense } from "react";
import {
  TableRowsSkeleton,
  EpsCardsSkeleton,
  HeroSkeleton,
} from "@/components/home/LoadingStates";
import type { EpsGrowthData } from "@/types/epsGrowthRanking";
import ChatSection from "@/components/home/ChatSection";

async function getInitialEpsData() {
  try {
    const response = await fetchEpsGrowthRanking({ limit: 10, skip: 0 });

    // Validate and transform each item
    const validData = response.data.map((item) => {
      if (!item?.symbol || !item?.company_name) {
        throw new Error(
          `Invalid item data: Missing required fields for item ${JSON.stringify(item)}`
        );
      }
      return item;
    });

    return {
      data: validData,
      total: response.metadata?.total || 0,
    };
  } catch (error) {
    console.error("Failed to fetch initial EPS growth ranking:", error);
    throw error; // Let the error boundary handle it
  }
}

export default async function HomeView() {
  let initialData: EpsGrowthData[] = [];
  let initialTotal = 10;

  try {
    const result = await getInitialEpsData();
    initialData = result.data;
    initialTotal = result.total;
  } catch (error) {
    console.error("Error in HomeView:", error);
    // The ErrorBoundary will handle the display
  }

  // Transform data for StockRankTable
  const tableData = initialData.map((item, index) => ({
    key: index,
    symbol: item.symbol,
    companyName: item.company_name,
    currentEps: item.eps_diluted?.toFixed(2) ?? "N/A",
    previousEps: item.previous_eps_diluted?.toFixed(6) ?? "N/A",
    epsGrowth: item.eps_growth
      ? item.eps_growth > 999999
        ? ">999999"
        : item.eps_growth.toFixed(2)
      : "N/A",
    reportDate: item.report_date
      ? new Date(item.report_date).toLocaleDateString()
      : "N/A",
    market: item.market_code,
    quarter: item.quarter,
    year: item.year,
  }));

  return (
    <div className="min-h-screen bg-gradient-to-b from-blue-500/5 via-transparent to-transparent">
      <div className="max-w-7xl mx-auto px-4 sm:px-6">
        <div className="flex flex-col gap-16 py-10">
          <ErrorBoundary>
            <div className="w-full">
              <Suspense fallback={<HeroSkeleton />}>
                <HeroSection className="min-h-[400px]" />
              </Suspense>
            </div>
          </ErrorBoundary>

          <ErrorBoundary>
            <div className="w-full space-y-6">
              <h2 className="text-3xl font-bold bg-gradient-to-r from-blue-500 to-purple-500 bg-clip-text text-transparent inline-block">
                Top Performers
              </h2>
              <Suspense fallback={<EpsCardsSkeleton />}>
                <ClientEpsCardSection
                  className="min-h-[300px]"
                  initialData={initialData}
                  initialTotal={initialTotal}
                />
              </Suspense>
            </div>
          </ErrorBoundary>

          <ErrorBoundary>
            <div className="w-full space-y-6">
              <h2 className="text-3xl font-bold bg-gradient-to-r from-blue-500 to-purple-500 bg-clip-text text-transparent inline-block">
                Rankings
              </h2>
              <Suspense fallback={<TableRowsSkeleton />}>
                <StockRankTable data={tableData} className="min-h-[500px]" />
              </Suspense>
            </div>
          </ErrorBoundary>

          <ErrorBoundary>
            <div className="w-full">
              <Suspense
                fallback={
                  <div className="w-full h-[500px] animate-pulse bg-card/50 rounded-lg" />
                }
              >
                {/* ChatSection is a client component */}
                <ChatSection />
              </Suspense>
            </div>
          </ErrorBoundary>
        </div>
      </div>
    </div>
  );
}
