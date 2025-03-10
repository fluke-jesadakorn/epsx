import RankingView from "@/components/home/StockRankTable";
import { fetchStockScreenerData } from "@/app/actions/stockData";
import { Suspense } from "react";

async function RankingPage() {
  const data = await fetchStockScreenerData();

  return (
    <Suspense fallback={<div>Loading...</div>}>
      <RankingView data={data} />
    </Suspense>
  );
}

export default RankingPage;

// Revalidate page every 5 minutes
export const revalidate = 300;
