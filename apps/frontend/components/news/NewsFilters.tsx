'use client';

import { useState, useEffect } from 'react';
import { useRouter, useSearchParams } from 'next/navigation';

interface NewsFiltersProps {
  currentCategory?: string;
  currentTag?: string;
}

export function NewsFilters({ currentCategory, currentTag }: NewsFiltersProps) {
  const router = useRouter();
  const searchParams = useSearchParams();
  const [searchQuery, setSearchQuery] = useState('');

  const categories = [
    { id: 'all', name: 'All Categories', slug: '' },
    { id: 'market-analysis', name: 'Market Analysis', slug: 'market-analysis' },
    { id: 'earnings-reports', name: 'Earnings Reports', slug: 'earnings-reports' },
    { id: 'eps-insights', name: 'EPS Insights', slug: 'eps-insights' },
    { id: 'platform-updates', name: 'Platform Updates', slug: 'platform-updates' },
  ];

  const handleCategoryChange = (categorySlug: string) => {
    const params = new URLSearchParams(searchParams);
    
    if (categorySlug) {
      params.set('category', categorySlug);
    } else {
      params.delete('category');
    }
    
    // Reset to first page when filtering
    params.delete('page');
    
    router.push(`/news?${params.toString()}`);
  };

  const handleSearch = (e: React.FormEvent) => {
    e.preventDefault();
    
    if (searchQuery.trim()) {
      const params = new URLSearchParams(searchParams);
      params.set('search', searchQuery.trim());
      params.delete('page');
      router.push(`/news?${params.toString()}`);
    }
  };

  const clearFilters = () => {
    router.push('/news');
  };

  const hasActiveFilters = currentCategory || currentTag || searchParams.get('search');

  return (
    <div className="bg-white border border-gray-200 rounded-lg p-6 mb-8">
      <div className="flex flex-col lg:flex-row lg:items-center lg:justify-between gap-6">
        {/* Category Filter */}
        <div className="flex-1">
          <label className="block text-sm font-medium text-gray-700 mb-2">
            Filter by Category
          </label>
          <div className="flex flex-wrap gap-2">
            {categories.map((category) => (
              <button
                key={category.id}
                onClick={() => handleCategoryChange(category.slug)}
                className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${
                  (currentCategory === category.slug) || (!currentCategory && !category.slug)
                    ? 'bg-blue-600 text-white'
                    : 'bg-gray-100 text-gray-700 hover:bg-gray-200'
                }`}
              >
                {category.name}
              </button>
            ))}
          </div>
        </div>

        {/* Search */}
        <div className="lg:w-80">
          <form onSubmit={handleSearch}>
            <label className="block text-sm font-medium text-gray-700 mb-2">
              Search Articles
            </label>
            <div className="flex gap-2">
              <input
                type="text"
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                placeholder="Search by title or content..."
                className="flex-1 px-4 py-2 border border-gray-300 rounded-lg focus:ring-2 focus:ring-blue-500 focus:border-blue-500"
              />
              <button
                type="submit"
                className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
              >
                <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                </svg>
              </button>
            </div>
          </form>
        </div>
      </div>

      {/* Active Filters & Clear */}
      {hasActiveFilters && (
        <div className="mt-6 pt-6 border-t border-gray-200 flex items-center justify-between">
          <div className="flex flex-wrap items-center gap-2">
            <span className="text-sm text-gray-600">Active filters:</span>
            
            {currentCategory && (
              <span className="inline-flex items-center px-3 py-1 bg-blue-100 text-blue-800 text-sm rounded-full">
                Category: {categories.find(c => c.slug === currentCategory)?.name}
                <button
                  onClick={() => handleCategoryChange('')}
                  className="ml-2 hover:text-blue-600"
                >
                  <svg className="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                  </svg>
                </button>
              </span>
            )}
            
            {currentTag && (
              <span className="inline-flex items-center px-3 py-1 bg-green-100 text-green-800 text-sm rounded-full">
                Tag: {currentTag}
                <button
                  onClick={() => {
                    const params = new URLSearchParams(searchParams);
                    params.delete('tag');
                    router.push(`/news?${params.toString()}`);
                  }}
                  className="ml-2 hover:text-green-600"
                >
                  <svg className="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                  </svg>
                </button>
              </span>
            )}
            
            {searchParams.get('search') && (
              <span className="inline-flex items-center px-3 py-1 bg-yellow-100 text-yellow-800 text-sm rounded-full">
                Search: "{searchParams.get('search')}"
                <button
                  onClick={() => {
                    const params = new URLSearchParams(searchParams);
                    params.delete('search');
                    router.push(`/news?${params.toString()}`);
                  }}
                  className="ml-2 hover:text-yellow-600"
                >
                  <svg className="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                  </svg>
                </button>
              </span>
            )}
          </div>
          
          <button
            onClick={clearFilters}
            className="text-sm text-gray-600 hover:text-gray-800 font-medium"
          >
            Clear All
          </button>
        </div>
      )}
    </div>
  );
}