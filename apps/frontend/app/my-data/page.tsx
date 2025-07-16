'use client';
import React, { useState } from "react";

type Asset = {
  symbol: string;
  name: string;
  amount?: number;
};

type MarketData = {
  symbol: string;
  price: number;
  mtd: number;
  ytd: number;
};

const MOCK_MARKET_DATA: MarketData[] = [
  { symbol: "AAPL", price: 195.5, mtd: 2.1, ytd: 12.3 },
  { symbol: "GOOGL", price: 2850.2, mtd: 1.5, ytd: 8.7 },
  { symbol: "TSLA", price: 720.8, mtd: -0.8, ytd: 5.2 },
];

export default function MarketDataSyncPage() {
  const [assets, setAssets] = useState<Asset[]>([]);
  const [input, setInput] = useState("");
  const [step, setStep] = useState<1 | 2>(1);

  const handleAddAsset = () => {
    if (!input.trim()) return;
    setAssets([...assets, { symbol: input.trim().toUpperCase(), name: input.trim() }]);
    setInput("");
  };

  const handleRemoveAsset = (symbol: string) => {
    setAssets(assets.filter((a) => a.symbol !== symbol));
  };

  const getMarketData = (symbol: string) =>
    MOCK_MARKET_DATA.find((d) => d.symbol === symbol);

  return (
    <div className="max-w-2xl mx-auto py-10 px-4">
      <h1 className="text-2xl font-bold mb-6">My Data</h1>
      {step === 1 && (
        <div>
          <h2 className="text-lg font-semibold mb-2">Step 1: Fill My Asset</h2>
          <div className="flex gap-2 mb-4">
            <input
              className="border px-2 py-1 rounded w-full"
              placeholder="Enter asset symbol (e.g. AAPL)"
              value={input}
              onChange={(e) => setInput(e.target.value)}
              onKeyDown={(e) => {
                if (e.key === "Enter") handleAddAsset();
              }}
            />
            <button
              className="bg-blue-600 text-white px-4 py-1 rounded"
              onClick={handleAddAsset}
            >
              Add
            </button>
          </div>
          <ul className="mb-4">
            {assets.map((asset) => (
              <li key={asset.symbol} className="flex items-center gap-2">
                <span>{asset.symbol}</span>
                <button
                  className="text-red-500"
                  onClick={() => handleRemoveAsset(asset.symbol)}
                >
                  Remove
                </button>
              </li>
            ))}
          </ul>
          <button
            className="bg-green-600 text-white px-6 py-2 rounded"
            disabled={assets.length === 0}
            onClick={() => setStep(2)}
          >
            Compare to Market
          </button>
        </div>
      )}
      {step === 2 && (
        <div>
          <h2 className="text-lg font-semibold mb-2">Step 2: Compare to Market</h2>
          <table className="w-full border mb-4">
            <thead>
              <tr className="bg-gray-100">
                <th className="border px-2 py-1">Asset</th>
                <th className="border px-2 py-1">Price</th>
                <th className="border px-2 py-1">MTD (%)</th>
                <th className="border px-2 py-1">YTD (%)</th>
              </tr>
            </thead>
            <tbody>
              {assets.map((asset) => {
                const data = getMarketData(asset.symbol);
                return (
                  <tr key={asset.symbol}>
                    <td className="border px-2 py-1">{asset.symbol}</td>
                    <td className="border px-2 py-1">
                      {data ? data.price : "N/A"}
                    </td>
                    <td className="border px-2 py-1">
                      {data ? data.mtd : "N/A"}
                    </td>
                    <td className="border px-2 py-1">
                      {data ? data.ytd : "N/A"}
                    </td>
                  </tr>
                );
              })}
            </tbody>
          </table>
          <button
            className="bg-gray-500 text-white px-4 py-1 rounded"
            onClick={() => setStep(1)}
          >
            Back
          </button>
        </div>
      )}
    </div>
  );
}
