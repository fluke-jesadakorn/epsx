import DataRankTable from "@/components/home/DataRankTable";

import { rankingColumns } from "./page";

import type { TableDataMetrics } from "@/types/stockFetchData";

interface RankingClientProps {
  initialData: TableDataMetrics[];
}

export default function RankingClient({ initialData }: RankingClientProps): React.JSX.Element {
  return (
    <div className="p-4 sm:p-6 lg:p-8 mx-auto">
      <DataRankTable data={initialData} columns={rankingColumns} defaultView="table" />
    </div>
  );
}
