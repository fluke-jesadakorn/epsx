import * as React from "react";
import { SkeletonLoader } from "@/components/common/Skeleton";
import { fetchStockScreenerData } from "@/app/actions/stockData";
import RankingClient from "./RankingClient";

// Define columns for ranking page - showing all columns
export const rankingColumns = [
  { key: "number" as const, header: "No." },
  { key: "symbol" as const, header: "Symbol" },
  { key: "name" as const, header: "Name" },
  { key: "valueIndex" as const, header: "Value Index" },
  {
    key: "growthRate" as const,
    header: "Growth Rate",
    tooltip: "Value Change Percentage",
  },
  {
    key: "activityScore" as const,
    header: "Activity Score",
    tooltip: "Engagement Level",
  },
  {
    key: "marketSize" as const,
    header: "Market Size",
    tooltip: "Total Market Presence",
  },
  {
    key: "growthFactor" as const,
    header: "Growth Factor",
    tooltip: "Growth Potential Indicator",
  },
  { key: "sector" as const, header: "Sector" },
  { key: "country" as const, header: "Country" },
  { key: "exchange" as const, header: "Exchange" },
  {
    key: "entryPhase" as const,
    header: "Entry Phase",
    tooltip: "Optimal Entry Time",
  },
  {
    key: "phaseStatus" as const,
    header: "Phase Status",
    tooltip: "Current Phase Status",
  },
  { key: "chart" as const, header: "Analytics", tooltip: "Open Analytics View" },
];

async function RankingPage() {
  const initialData = await fetchStockScreenerData();

  return (
    <React.Suspense fallback={<SkeletonLoader />}>
      <RankingClient initialData={initialData} />
    </React.Suspense>
  );
}

export default RankingPage;

// Revalidate page every 5 minutes
export const revalidate = 300;
