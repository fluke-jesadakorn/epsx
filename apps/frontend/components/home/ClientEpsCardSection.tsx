"use client";

import React from "react";
import type { CSSProperties } from "react";
import { TableDataMetrics } from "@/types/stockFetchData";
import { Card, CardHeader, CardContent } from "@/components/ui/card";

interface Props {
  style?: CSSProperties;
  className?: string;
  initialData: TableDataMetrics[];
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
      className={`flex flex-col gap-6 w-full ${className || ""}`}
      style={style}
    >
      {/* Section Header */}
      <div className="text-center space-y-4 mb-4 animate-fade-in">
        <h2 className="text-3xl font-bold bg-gradient-to-r from-blue-500 via-purple-500 to-pink-500 bg-clip-text text-transparent">
          Top Performing Companies
        </h2>
        <div className="w-24 h-1 bg-gradient-to-r from-blue-500 to-purple-500 mx-auto rounded-full" />
      </div>
      
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6 w-full animate-fade-in-delayed">
        {initialData.slice(0, 3).map((item, index) => {
          if (!item?.symbol || !item?.name) {
            console.error("Missing required data for item:", item);
            return null;
          }

          return (
            <Card
              key={item.symbol || index}
              className="overflow-hidden transition-all duration-300 hover:shadow-xl hover:-translate-y-2 bg-gradient-to-br from-card via-card/80 to-card/50 border border-blue-500/10 hover:border-blue-500/30 group"
            >
              <CardHeader className="pb-3 group-hover:bg-gradient-to-br from-blue-500/5 to-purple-500/5 transition-colors duration-300">
                <div className="flex flex-col space-y-2">
                  <div className="flex justify-between items-center">
                      <span
                        className={`${getMarketColor(item.exchange)} font-medium px-3 py-1 rounded-full bg-primary/5 group-hover:scale-105 transition-transform duration-300`}
                    >
                      {item.exchange || "N/A"}
                    </span>
                    {index < 3 && (
                      <span
                        className={`
                            font-bold text-sm px-3 py-1 rounded-full shadow-sm
                            ${index === 0 ? "bg-gradient-to-r from-yellow-300 to-yellow-400 text-yellow-900" : ""}
                            ${index === 1 ? "bg-gradient-to-r from-gray-300 to-gray-400 text-gray-900" : ""}
                            ${index === 2 ? "bg-gradient-to-r from-amber-300 to-amber-400 text-amber-900" : ""}
                            group-hover:scale-105 transition-transform duration-300
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
              <CardContent className="group-hover:bg-gradient-to-br from-blue-500/5 to-purple-500/5 transition-colors duration-300">
                <div className="space-y-4">
                  <div className="space-y-2">
                    <div className="flex justify-between items-center">
                      <p className="text-sm text-muted-foreground">Signal</p>
                      <p
                        className={`font-semibold transition-colors ${
                          item.startBuy?.active
                            ? "text-green-500 group-hover:text-green-400"
                            : item.startAction?.type === "sell" &&
                                item.startAction?.active
                              ? "text-rose-500 group-hover:text-rose-400"
                              : "text-yellow-500 group-hover:text-yellow-400"
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
                        className={`font-semibold transition-colors ${
                          parseFloat(item.epsGrowth || "0") >= 0 
                            ? "text-green-500 group-hover:text-green-400" 
                            : "text-rose-500 group-hover:text-rose-400"
                        }`}
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
                  <div className="pt-3 mt-2 border-t border-blue-500/10 flex justify-between items-center">
                    <p className="text-xs text-muted-foreground">
                      Last Report: {item.lastEarningsDate || "N/A"}
                    </p>
                    <a
                      href={`https://www.tradingview.com/symbols/${item.symbol}`}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="text-xs text-blue-500 hover:text-blue-400 transition-colors flex items-center gap-1 group-hover:gap-2"
                    >
                      View Chart <span className="transition-transform group-hover:translate-x-1">→</span>
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
