"use client";

import React from "react";
import type { CSSProperties } from "react";
import type { EpsGrowthData } from "@/types/epsGrowthRanking";
import { Card, CardHeader, CardContent } from "@/components/ui/card";

interface Props {
  style?: CSSProperties;
  className?: string;
  initialData: EpsGrowthData[];
  initialTotal: number;
}

export default function ClientEpsCardSection({
  style,
  className,
  initialData,
}: Props) {
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

  return (
    <div
      className={`flex flex-col gap-4 w-full ${className || ""}`}
      style={style}
    >
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4 w-full">
        {initialData.slice(0, 3).map((item, index) => {
          if (!item?.symbol || !item?.company_name) {
            console.error("Missing required data for item:", item);
            return null;
          }

          return (
            <Card
              key={item.symbol || index}
              className="overflow-hidden transition-all hover:shadow-lg hover:-translate-y-1 bg-gradient-to-br from-card via-card/80 to-card/50"
            >
              <CardHeader className="pb-3">
                <div className="flex justify-between items-center">
                  <span
                    className={`${getMarketColor(item.market_code)} font-medium px-3 py-1 rounded-full bg-primary/5`}
                  >
                    {item.market_code || "N/A"}
                  </span>
                  {index < 3 && (
                    <span
                      className={`
                          font-bold text-sm px-3 py-1 rounded-full
                          ${index === 0 ? "bg-yellow-100 text-yellow-700" : ""}
                          ${index === 1 ? "bg-gray-100 text-gray-700" : ""}
                          ${index === 2 ? "bg-amber-100 text-amber-700" : ""}
                        `}
                    >
                      #{index + 1}
                    </span>
                  )}
                </div>
              </CardHeader>
              <CardContent>
                <div className="space-y-3">
                  <div>
                    <h4 className="text-lg font-bold truncate">
                      {item.company_name || "Unknown Company"}
                    </h4>
                    <p className="text-muted-foreground text-sm">
                      {item.symbol || "N/A"}
                    </p>
                  </div>
                  <div className="grid grid-cols-2 gap-2">
                    <div className="space-y-1">
                      <p className="text-sm text-muted-foreground">
                        Current EPS
                      </p>
                      <p className="font-semibold">
                        {typeof item.eps_diluted === "number"
                          ? item.eps_diluted.toFixed(2)
                          : "N/A"}
                      </p>
                    </div>
                    <div className="space-y-1">
                      <p className="text-sm text-muted-foreground">Growth</p>
                      <p
                        className={`font-semibold ${(item.eps_growth || 0) >= 0 ? "text-green-500" : "text-rose-500"}`}
                      >
                        {typeof item.eps_growth === "number"
                          ? `${item.eps_growth >= 0 ? "+" : ""}${item.eps_growth.toFixed(2)}%`
                          : "0%"}
                      </p>
                    </div>
                  </div>
                  <div className="pt-2 border-t">
                    <p className="text-xs text-muted-foreground">
                      Last Report:{" "}
                      {item.report_date
                        ? new Date(item.report_date).toLocaleDateString()
                        : "N/A"}
                    </p>
                  </div>
                </div>
              </CardContent>
            </Card>
          );
        })}
      </div>
    </div>
  );
}
