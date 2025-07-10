import FinancialDataTable from '@/components/home/FinancialDataTable';

import { financialColumns } from './page';

import type { StockFinancialData } from '@/types/financialChartData';

interface RankingClientProps {
  initialData: StockFinancialData[];
}

export default function RankingClient({
  initialData,
}: RankingClientProps): React.JSX.Element {
  return (
    <div className="p-4 sm:p-6 lg:p-8 mx-auto">
      <FinancialDataTable
        data={initialData}
        columns={financialColumns}
        defaultView="table"
      />
    </div>
  );
}
