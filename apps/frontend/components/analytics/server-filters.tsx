import { getServerFilterOptions, type EPSQueryParams, type FilterOptions } from '@/lib/unified-server-data';
import FilterForm from './filter-form';

interface ServerFiltersProps {
  currentParams: EPSQueryParams;
}

export default async function ServerFilters({ currentParams }: ServerFiltersProps) {
  // Get filter options from the backend or fallback to static options
  let filterOptions: FilterOptions;

  try {
    filterOptions = await getServerFilterOptions();

    // Minimal fallback if TradingView API returns empty data
    if (!filterOptions.countries.length && !filterOptions.sectors.length) {
      filterOptions = {
        countries: [{ value: 'america', label: 'United States' }],
        sectors: ['Technology', 'Financial'],
        exchanges: ['NASDAQ', 'NYSE'],
        stock_types: ['common'],
      };
    }
  } catch (_error) {
    // Minimal emergency fallback
    filterOptions = {
      countries: [{ value: 'america', label: 'United States' }],
      sectors: ['Technology', 'Financial'],
      exchanges: ['NASDAQ', 'NYSE'],
      stock_types: ['common'],
    };
  }

  return (
    <FilterForm
      filterOptions={filterOptions}
      currentParams={currentParams}
    />
  );
}