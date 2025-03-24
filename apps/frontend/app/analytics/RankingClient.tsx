"use client";

import { useEffect, useState } from "react";
import DataRankTable from "@/components/home/DataRankTable";
import { useLoadingFetch } from "@/hooks/useLoadingFetch";
import { rankingColumns } from "./page";
import type { TableDataMetrics } from "@/types/stockFetchData";

interface RankingClientProps {
  initialData: TableDataMetrics[];
}

export default function RankingClient({ initialData }: RankingClientProps): React.JSX.Element {
  const [data, setData] = useState<TableDataMetrics[]>(initialData);
  const { fetchWithLoading } = useLoadingFetch();

  useEffect(() => {
    const fetchData = async () => {
      try {
        const newData = await fetchWithLoading(async () => {
          const response = await fetch("/api/stock-screener");
          if (!response.ok) {
            throw new Error("Failed to fetch data");
          }
          return response.json();
        });
        setData(newData);
      } catch (error) {
        console.error("Error fetching data:", error);
      }
    };

    // Set up polling interval (every 5 minutes)
    const interval = setInterval(fetchData, 5 * 60 * 1000);

    // Initial fetch
    fetchData();

    // Cleanup interval on unmount
    return () => clearInterval(interval);
  }, [fetchWithLoading]);

  return (
    <div className="p-4 sm:p-6 lg:p-8 mx-auto">
      <DataRankTable data={data} columns={rankingColumns} defaultView="table" />
    </div>
  );
}
