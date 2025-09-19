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
    
    // If we get empty arrays, fall back to static options
    if (!filterOptions.countries.length && !filterOptions.sectors.length) {
      filterOptions = {
        countries: [
          'United States',
          'Canada', 
          'United Kingdom',
          'Germany',
          'France',
          'Japan',
          'Australia'
        ],
        sectors: [
          'Technology',
          'Healthcare', 
          'Financial Services',
          'Consumer Discretionary',
          'Industrials',
          'Energy',
          'Telecommunications',
          'Real Estate',
        ],
        exchanges: ['NASDAQ', 'NYSE', 'LSE', 'TSX', 'ASX', 'HKEX', 'TSE', 'EURONEXT'],
        stock_types: ['common', 'preferred', 'reit', 'etf'],
      };
    }
  } catch (error) {
    console.error('Failed to get filter options, using fallback:', error);
    
    // Fallback to static options
    filterOptions = {
      countries: [
        'United States',
        'Canada', 
        'United Kingdom',
        'Germany',
        'France',
        'Japan',
        'Australia'
      ],
      sectors: [
        'Technology',
        'Healthcare', 
        'Financial Services',
        'Consumer Discretionary',
        'Industrials',
        'Energy',
        'Telecommunications',
        'Real Estate',
      ],
      exchanges: ['NASDAQ', 'NYSE', 'LSE', 'TSX', 'ASX', 'HKEX', 'TSE', 'EURONEXT'],
      stock_types: ['common', 'preferred', 'reit', 'etf'],
    };
  }

  return (
    <FilterForm 
      filterOptions={filterOptions}
      currentParams={currentParams}
    />
  );
}