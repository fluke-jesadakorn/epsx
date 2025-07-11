import FinancialDataTable from '@/components/home/FinancialDataTable';

import type { StockFinancialData } from '@/types/financialChartData';

interface RankingClientProps {
  initialData: StockFinancialData[];
}

export default function RankingClient({
  initialData,
}: RankingClientProps): React.JSX.Element {
  return <FinancialDataTable data={initialData} />;
}
