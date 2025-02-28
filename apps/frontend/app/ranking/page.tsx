"use client";

import { useEffect, useState } from "react";
import RankingView from "@/components/home/StockRankTable";
import { fetchEpsGrowthRanking } from "@/app/actions/stockData";
import { EpsGrowthData } from "@/types/epsGrowthRanking";

export default function Ranking() {
  const [tableData, setTableData] = useState<any[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    const fetchData = async () => {
      try {
        const response = await fetchEpsGrowthRanking({ limit: 20 });
        const transformedData = response.data.map((item: EpsGrowthData, index: number) => ({
          key: index,
          symbol: item.symbol,
          companyName: item.company_name,
          currentEps: item.eps_diluted.toFixed(2),
          previousEps: item.previous_eps_diluted.toFixed(2),
          epsGrowth: item.eps_growth === Infinity ? '>999999' : 
                     isNaN(item.eps_growth) ? 'N/A' : 
                     item.eps_growth.toFixed(2),
          reportDate: new Date(item.report_date).toLocaleDateString(),
          market: item.market_code,
          quarter: item.quarter,
          year: item.year
        }));
        setTableData(transformedData);
      } catch (error) {
        console.error('Error fetching ranking data:', error);
      } finally {
        setLoading(false);
      }
    };

    fetchData();
  }, []);

  // Map user role to access level
  // const getAccessLevel = () => {
  //   if (!user) return undefined; // Public access (rank 21+)
  //   switch (user.role) {
  //     case ROLES.ADMIN:
  //       return 3; // Can see all ranks
  //     case ROLES.PREMIUM:
  //       return 2; // Can see ranks 1+
  //     case ROLES.BASIC:
  //       return 1; // Can see ranks 11+
  //     default:
  //       return undefined;
  //   }
  // };

  if (loading) {
    return <div>Loading...</div>;
  }

  return <RankingView accessLevel={1} data={tableData} />;
}
