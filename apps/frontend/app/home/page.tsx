import StockRankTable from "@/components/home/StockRankTable";
import HeroSection from "@/components/home/HeroSection";
import { fetchStockScreenerData } from "@/app/actions/stockData";
import { Suspense } from "react";

// Define columns for home page - showing a compact view
const homeColumns = [
  { key: "number" as const, header: "No." },
  { key: "symbol" as const, header: "Symbol" },
  { key: "name" as const, header: "Name" },
  { key: "price" as const, header: "Price" },
  {
    key: "changePercent" as const,
    header: "Change %",
    tooltip: "Price Change Percentage",
  },
  {
    key: "marketCap" as const,
    header: "Market Cap",
    tooltip: "Market Capitalization",
  },
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

async function HomePage() {
  const data = await fetchStockScreenerData();
  // Get the last 10 items from the data
  const lastTenItems = data.slice(-10);

  return (
    <div>
      <HeroSection />
      <Suspense fallback={<div>Loading...</div>}>
        <div className="p-4 sm:p-6 lg:p-8 max-w-7xl mx-auto">
          <StockRankTable data={lastTenItems} columns={homeColumns} />
        </div>
      </Suspense>
    </div>
  );
}

export default HomePage;

// Revalidate page every 5 minutes
export const revalidate = 300;
