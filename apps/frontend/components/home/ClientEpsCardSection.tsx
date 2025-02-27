"use client";

import React from "react";
import type { CSSProperties } from "react";
import type { EpsGrowthData } from "@/types/epsGrowthRanking";
import {
  Card,
  CardHeader,
  CardContent,
} from "@/components/ui/card";

interface ClientEpsCardSectionProps {
  style?: CSSProperties;
  className?: string;
  initialData: EpsGrowthData[];
  initialTotal: number;
}

export default function ClientEpsCardSection({
  style,
  className,
  initialData,
}: ClientEpsCardSectionProps) {
  const getMarketColor = (marketCode: string | undefined) => {
    switch (marketCode) {
      case "TYO":
        return "text-blue-500"; // Blue
      case "BOM":
        return "text-green-500"; // Green
      case "OTC":
        return "text-purple-600"; // Purple
      default:
        return "text-gray-400"; // Grey
    }
  };

  const getRankColor = (index: number) => {
    switch (index) {
      case 0:
        return "text-yellow-400"; // Gold
      case 1:
        return "text-gray-400"; // Silver
      case 2:
        return "text-amber-600"; // Bronze
      default:
        return "text-gray-500";
    }
  };

  return (
    <div className={`flex flex-col gap-4 w-full ${className || ''}`} style={style}>
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4 w-full">
        {initialData.map((item, index) => {
          // Skip rendering if item or required properties are missing
          if (!item?.symbol || !item?.company_name) {
            console.error("Missing required data for item:", item);
            return null;
          }

          return (
            <Card key={item.symbol || index}>
              <CardHeader className="pb-2">
                <div className="flex justify-between items-center">
                  <span className={getMarketColor(item.market_code)}>
                    {item.market_code || "N/A"}
                  </span>
                  {index < 3 && (
                    <span className={`font-bold ${getRankColor(index)}`}>
                      #{index + 1}
                    </span>
                  )}
                </div>
              </CardHeader>
              <CardContent>
                <div className="space-y-2">
                  <h4 className="text-xl font-bold">
                    {item.company_name || "Unknown Company"}
                  </h4>
                  <p className="text-gray-600">
                    Symbol: {item.symbol || "N/A"}
                  </p>
                  <p className="text-gray-600">
                    EPS:{" "}
                    {typeof item.eps_diluted === "number"
                      ? item.eps_diluted.toFixed(2)
                      : "N/A"}
                  </p>
                  <p className={`font-bold ${(item.eps_growth || 0) >= 0 ? 'text-green-500' : 'text-red-500'}`}>
                    Growth:{" "}
                    {typeof item.eps_growth === "number"
                      ? item.eps_growth.toFixed(2)
                      : "0"}
                    %
                  </p>
                  <p className="text-gray-400">
                    Last Report:{" "}
                    {item.report_date
                      ? new Date(item.report_date).toLocaleDateString()
                      : "N/A"}
                  </p>
                </div>
              </CardContent>
            </Card>
          );
        })}
      </div>
    </div>
  );
}
