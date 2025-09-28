import { getServerFilterOptions, type EPSQueryParams, type FilterOptions } from '@/lib/server-data';
import FilterForm from './FilterForm';

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
      console.warn('⚠️ TradingView API returned empty filter options, using minimal fallback');
      filterOptions = {
        countries: ['United States'],
        sectors: ['Technology', 'Financial'],
        exchanges: ['NASDAQ', 'NYSE'],
        stock_types: ['common'],
      };
    }
  } catch (error) {
    console.error('❌ TradingView filter API completely failed, using minimal fallback:', error);
    
    // Minimal emergency fallback
    filterOptions = {
      countries: ['United States'],
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