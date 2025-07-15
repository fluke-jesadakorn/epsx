import StockRankingTable from '@/components/shared/StockRankingTable';

import type { StockFinancialData } from '@/types/financialChartData';

interface RankingClientProps {
  initialData: StockFinancialData[];
}

export default function RankingClient({
  initialData,
}: RankingClientProps): React.JSX.Element {
  return (
    <StockRankingTable 
      data={initialData}
      title="🍯 Sweet Financial Rankings 📊"
      subtitle="Discover the most delicious investment opportunities with our comprehensive analytics"
      showRank={true}
      rankShift={0}
    />
  );
}
