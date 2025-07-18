'use client';

import type { User } from "@/types/auth/user";
import type { USDTDetails } from "@/types/userLevel";

interface DashboardViewProps {
  user: User & { usdtDetails?: USDTDetails } | null;
}

export function DashboardView({ user }: DashboardViewProps) {
  if (!user) {
    return <div>Loading...</div>;
  }

  return (
    <div className="container mx-auto px-4 py-8">
      <h1 className="text-2xl font-bold mb-6">Welcome, {user.email}</h1>
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        <div className="bg-white p-6 rounded-lg shadow">
          <h2 className="text-xl font-semibold mb-4">Account Overview</h2>
          <p>Member since: {new Date(user.createdAt).toLocaleDateString()}</p>
          {user.usdtDetails && (
            <>
              <p className="mt-4">User Level: Level {user.usdtDetails.userLevel ? 
                (() => {
                  const levelMap = {
                    'BRONZE': 0,
                    'SILVER': 1,
                    'GOLD': 2,
                    'PLATINUM': 3,
                    'DIAMOND': 4,
                    'VIP': 5,
                    'API_PERSONAL': 6,
                    'API_COMPANY': 7,
                    'API_PARTNER': 6
                  };
                  return levelMap[user.usdtDetails.userLevel as keyof typeof levelMap] || 0;
                })() : 0}
              </p>
              <p>Expiration Date: {new Date(user.usdtDetails.paymentStatus.expirationDate).toLocaleDateString()}</p>
            </>
          )}
        </div>

        {user.usdtDetails && (
          <div className="bg-white p-6 rounded-lg shadow">
            <h2 className="text-xl font-semibold mb-4">Payment Details</h2>
            <p>Payment Method: {user.usdtDetails.paymentStatus.paymentMethod}</p>
            <p>Last Payment: {new Date(user.usdtDetails.paymentStatus.lastPaymentDate).toLocaleDateString()}</p>
            <p>Amount: ${user.usdtDetails.paymentStatus.amount}</p>
            <p>Network: {user.usdtDetails.network}</p>
            <p>Wallet Address: {user.usdtDetails.walletAddress}</p>
          </div>
        )}
      </div>
    </div>
  );
}
