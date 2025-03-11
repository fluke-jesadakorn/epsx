"use client";

import React from "react";
import type { CSSProperties } from "react";
import { TableStockData } from "@/types/stockFetchData";
import { Card, CardHeader, CardContent } from "@/components/ui/card";

interface Props {
  style?: CSSProperties;
  className?: string;
  initialData: TableStockData[];
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
          if (!item?.symbol || !item?.name) {
            console.error("Missing required data for item:", item);
            return null;
          }

          return (
            <Card
              key={item.symbol || index}
              className="overflow-hidden transition-all hover:shadow-lg hover:-translate-y-1 bg-gradient-to-br from-card via-card/80 to-card/50"
            >
              <CardHeader className="pb-3">
                <div className="flex flex-col space-y-2">
                  <div className="flex justify-between items-center">
                    <span
                      className={`${getMarketColor(item.exchange)} font-medium px-3 py-1 rounded-full bg-primary/5`}
                    >
                      {item.exchange || "N/A"}
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
                        Rank #{index + 1}
                      </span>
                    )}
                  </div>
                  <div className="flex items-center space-x-2">
                    <p className="font-semibold">{item.symbol || "N/A"}</p>
                    <span className="text-muted-foreground">•</span>
                    <p className="text-muted-foreground truncate">
                      {item.name || "Unknown Company"}
                    </p>
                  </div>
                </div>
              </CardHeader>
              <CardContent>
                <div className="space-y-3">
                  <div className="space-y-2">
                    <div className="flex justify-between items-center">
                      <p className="text-sm text-muted-foreground">Signal</p>
                      <p
                        className={`font-semibold ${
                          item.startBuy?.active
                            ? "text-green-500"
                            : item.startAction?.type === "sell" &&
                                item.startAction?.active
                              ? "text-rose-500"
                              : "text-yellow-500"
                        }`}
                      >
                        {item.startBuy?.active
                          ? "Buy"
                          : item.startAction?.type === "sell" &&
                              item.startAction?.active
                            ? "Sell"
                            : "Hold"}
                      </p>
                    </div>

                    <div className="flex justify-between items-center">
                      <p className="text-sm text-muted-foreground">Growth</p>
                      <p
                        className={`font-semibold ${parseFloat(item.epsGrowth) >= 0 ? "text-green-500" : "text-rose-500"}`}
                      >
                        {item.epsGrowth || "0%"}
                      </p>
                    </div>

                    <div className="flex justify-between items-center">
                      <p className="text-sm text-muted-foreground">Status</p>
                      <p className="text-sm">
                        {item.startAction?.type === "hold" &&
                        item.startAction?.active
                          ? "Waiting for Hold"
                          : item.startAction?.type === "sell" &&
                              !item.startAction?.active
                            ? "Waiting for Sell"
                            : "Active"}
                      </p>
                    </div>
                  </div>
                  <div className="pt-2 border-t flex justify-between items-center">
                    <p className="text-xs text-muted-foreground">
                      Last Report: {item.lastEarningsDate || "N/A"}
                    </p>
                    <a
                      href={`https://www.tradingview.com/symbols/${item.symbol}`}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="text-xs text-blue-500 hover:text-blue-700 transition-colors"
                    >
                      View Chart →
                    </a>
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
