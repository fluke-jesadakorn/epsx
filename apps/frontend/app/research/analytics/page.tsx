import StockGrowthTable from "@/components/home/StockGrowthTable";
import { fetchEpsGrowthRanking } from "@/app/actions/stockData";
import { Suspense } from "react";

async function AnalyticsPage() {
  const { data } = await fetchEpsGrowthRanking({
    sortBy: "growthIndicator",
    limit: 1000 // Show more results for analysis
  });

  return (
    <Suspense fallback={<div>Loading...</div>}>
      <StockGrowthTable data={data} />
    </Suspense>
  );
}

export default AnalyticsPage;

// Revalidate page every 5 minutes
export const revalidate = 300;
