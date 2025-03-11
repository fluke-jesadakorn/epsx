import StockRankTable from "@/components/home/StockRankTable";
import { fetchStockScreenerData } from "@/app/actions/stockData";
import { Suspense } from "react";

// Define columns for ranking page - showing all columns
const rankingColumns = [
  { key: "number" as const, header: "No." },
  { key: "symbol" as const, header: "Symbol" },
  { key: "name" as const, header: "Name" },
  { key: "price" as const, header: "Price" },
  {
    key: "changePercent" as const,
    header: "Change %",
    tooltip: "Price Change Percentage",
  },
  { key: "volume" as const, header: "Volume", tooltip: "Trading Volume" },
  {
    key: "marketCap" as const,
    header: "Market Cap",
    tooltip: "Market Capitalization",
  },
  {
    key: "peRatio" as const,
    header: "P/E",
    tooltip: "Price to Earnings Ratio",
  },
  { key: "sector" as const, header: "Sector" },
  { key: "country" as const, header: "Country" },
  { key: "exchange" as const, header: "Exchange" },
  {
    key: "startBuy" as const,
    header: "Start Buy",
    tooltip: "When to start buying",
  },
  {
    key: "startAction" as const,
    header: "Hold or Sell",
    tooltip: "When to start holding/selling",
  },
  { key: "chart" as const, header: "Chart", tooltip: "Open TradingView Chart" },
];

async function RankingPage() {
  const data = await fetchStockScreenerData();

  return (
    <Suspense fallback={<div>Loading...</div>}>
      <div className="p-4 sm:p-6 lg:p-8 mx-auto">
        <StockRankTable data={data} columns={rankingColumns} />
      </div>
    </Suspense>
  );
}

export default RankingPage;

// Revalidate page every 5 minutes
export const revalidate = 300;
