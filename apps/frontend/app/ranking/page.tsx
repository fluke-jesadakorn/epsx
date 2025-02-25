"use client";

import RankingView from "@/components/home/StockRankTable";
import { useAuth } from "@/hooks/useAuth";
import { ROLES } from "@/constants/roles";

export default function Ranking() {
  const { user } = useAuth();
  
  // Map user role to access level
  const getAccessLevel = () => {
    if (!user) return undefined; // Public access (rank 21+)
    switch (user.role) {
      case ROLES.ADMIN:
        return 3; // Can see all ranks
      case ROLES.PREMIUM:
        return 2; // Can see ranks 1+
      case ROLES.BASIC:
        return 1; // Can see ranks 11+
      default:
        return undefined;
    }
  };

  return <RankingView accessLevel={getAccessLevel()} />;
}
