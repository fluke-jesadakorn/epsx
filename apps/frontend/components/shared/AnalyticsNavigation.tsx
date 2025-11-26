// Shared navigation component for analytics-related pages
import Link from 'next/link';

interface AnalyticsNavigationProps {
  currentPage: 'analytics' | 'portfolio';
}

export function AnalyticsNavigation({ currentPage }: AnalyticsNavigationProps) {
  return (
    <div className="mb-6 flex justify-center">
      <div className="inline-flex rounded-full bg-white/80 p-1 backdrop-blur-md dark:bg-slate-800/80">
        <Link 
          href="/analytics"
          className={`rounded-full px-4 py-2 text-sm font-medium transition-all duration-300 ${
            currentPage === 'analytics' 
              ? 'bg-gradient-to-r from-orange-500 to-yellow-500 text-white shadow-lg' 
              : 'text-gray-700 hover:bg-gray-100 dark:text-gray-300 dark:hover:bg-slate-700'
          }`}
        >
          <svg className="mr-2 inline h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
          </svg>
          Analytics Dashboard
        </Link>
        <Link 
          href="/portfolio"
          className={`rounded-full px-4 py-2 text-sm font-medium transition-all duration-300 ${
            currentPage === 'portfolio' 
              ? 'bg-gradient-to-r from-blue-500 to-blue-600 text-white shadow-lg' 
              : 'text-gray-700 hover:bg-gray-100 dark:text-gray-300 dark:hover:bg-slate-700'
          }`}
        >
          <svg className="mr-2 inline h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 7h8m0 0v8m0-8l-8 8-4-4-6 6" />
          </svg>
          Portfolio
        </Link>
      </div>
    </div>
  );
}