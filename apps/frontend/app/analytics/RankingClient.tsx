import LazyStockRankingTable from '@/components/shared/LazyStockRankingTable';

import type { StockFinancialData } from '@/types/financialChartData';

interface RankingClientProps {
  initialData: StockFinancialData[];
}

export default function RankingClient({
  initialData,
}: RankingClientProps): React.JSX.Element {
  return (
    <LazyStockRankingTable 
      data={initialData}
      useLazyLoading={true}
      maxCards={initialData.length || 20}
      title="🍯 Sweet Financial Rankings 📊"
      subtitle="Discover the most delicious investment opportunities with our comprehensive analytics"
      showRank={true}
      rankShift={0}
    />
  );
}
