import { NextResponse } from 'next/server';
import type { FilterOptions } from '@/types/analytics';
import { serverConfig } from '@/config/env';

// Interface for backend country data
interface CountryData {
  value: string;  // API value (lowercase)
  label: string;  // Display name
}

export async function GET() {
  try {
    const apiUrl = serverConfig.backendUrl;
    
    // Configure fetch options with SSL handling for development
    const fetchOptions = {
      method: 'GET',
      headers: { 'Accept': 'application/json' },
      next: { revalidate: 3600 }, // Cache for 1 hour
    };
    
    // Fetch countries, sectors, exchanges, and stock types in parallel
    const [countriesResponse, sectorsResponse, exchangesResponse, stockTypesResponse] = await Promise.all([
      fetch(`${apiUrl}/api/v1/analytics/eps-rankings/countries`, fetchOptions),
      fetch(`${apiUrl}/api/v1/analytics/eps-rankings/sectors`, fetchOptions),
      fetch(`${apiUrl}/api/v1/analytics/eps-rankings/exchanges`, fetchOptions),
      fetch(`${apiUrl}/api/v1/analytics/eps-rankings/stock-types`, fetchOptions),
    ]);

    let countries: string[] = [];
    let sectors: string[] = [];
    let exchanges: string[] = [];
    let stock_types: string[] = [];

    if (countriesResponse.ok) {
      const countriesData = await countriesResponse.json();
      if (Array.isArray(countriesData.countries)) {
        // Handle new format with value/label objects
        if (countriesData.countries.length > 0 && typeof countriesData.countries[0] === 'object') {
          countries = (countriesData.countries as CountryData[]).map((c: CountryData) => c.label);
        } else {
          // Fallback for old format (just strings)
          countries = countriesData.countries;
        }
      }
    }

    if (sectorsResponse.ok) {
      const sectorsData = await sectorsResponse.json();
      sectors = Array.isArray(sectorsData.sectors) ? sectorsData.sectors : [];
    }

    if (exchangesResponse.ok) {
      const exchangesData = await exchangesResponse.json();
      exchanges = Array.isArray(exchangesData.exchanges) ? exchangesData.exchanges : [];
    }

    if (stockTypesResponse.ok) {
      const stockTypesData = await stockTypesResponse.json();
      stock_types = Array.isArray(stockTypesData.stock_types) ? stockTypesData.stock_types : [];
    }

    // Fallback data if API calls fail
    if (countries.length === 0) {
      countries = ['United States', 'Canada', 'United Kingdom', 'Germany', 'France', 'Japan', 'Australia'];
    }

    if (sectors.length === 0) {
      sectors = [
        'Technology',
        'Healthcare',
        'Financial Services',
        'Consumer Discretionary',
        'Industrials',
        'Energy',
        'Telecommunications',
        'Real Estate',
      ];
    }

    if (exchanges.length === 0) {
      exchanges = ['NASDAQ', 'NYSE', 'LSE', 'TSX', 'ASX', 'HKEX', 'TSE', 'EURONEXT'];
    }

    if (stock_types.length === 0) {
      stock_types = ['common', 'preferred', 'reit', 'etf'];
    }

    const filterOptions: FilterOptions = { countries, sectors, exchanges, stock_types };
    return NextResponse.json(filterOptions);
  } catch (error) {
    console.error('Error fetching filter options:', error);
    
    // Return fallback data with proper display names
    const fallbackOptions: FilterOptions = {
      countries: ['United States', 'Canada', 'United Kingdom', 'Germany', 'France', 'Japan', 'Australia'],
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
    
    return NextResponse.json(fallbackOptions);
  }
}