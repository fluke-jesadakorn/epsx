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
      useLazyLoading
      maxCards={initialData.length || 20}
      title="🍯 Sweet Performance Rankings 📊"
      subtitle="Discover the most delicious data insights with our comprehensive analytics"
      showRank
      rankShift={0}
    />
  );
}
