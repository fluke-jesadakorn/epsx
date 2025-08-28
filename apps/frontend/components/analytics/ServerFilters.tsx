import { getFilterOptions, type EPSQueryParams } from '@/lib/analytics-server';
import FilterForm from './FilterForm';

interface ServerFiltersProps {
  currentParams: EPSQueryParams;
}

export default async function ServerFilters({ currentParams }: ServerFiltersProps) {
  const filterOptions = await getFilterOptions();

  return (
    <FilterForm 
      filterOptions={filterOptions}
      currentParams={currentParams}
    />
  );
}