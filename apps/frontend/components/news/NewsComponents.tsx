import { ReactNode } from 'react';
import Image from 'next/image';

// Stock Chart Component (placeholder - would integrate with real charting library)
export function StockChart({ symbol, period = '1M', height = 400 }: {
  symbol: string;
  period?: string;
  height?: number;
}) {
  return (
    <div className="my-8 p-6 bg-gray-50 rounded-lg border border-gray-200">
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-lg font-semibold">{symbol} Stock Chart</h3>
        <span className="text-sm text-gray-600">{period}</span>
      </div>
      <div 
        className="bg-white rounded-lg border border-gray-200 flex items-center justify-center"
        style={{ height: `${height}px` }}
      >
        <div className="text-center text-gray-500">
          <svg className="w-12 h-12 mx-auto mb-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
          </svg>
          <p>Interactive {symbol} chart would appear here</p>
          <p className="text-xs mt-1">Integrated with TradingView or similar service</p>
        </div>
      </div>
    </div>
  );
}

// EPS Table Component
export function EPSTable({ symbols, quarters = 4 }: {
  symbols: string[];
  quarters?: number;
}) {
  // Mock data for demonstration
  const mockData = symbols.map(symbol => ({
    symbol,
    quarters: Array.from({ length: quarters }, (_, i) => ({
      quarter: `Q${quarters - i} 2024`,
      eps: (Math.random() * 5 + 1).toFixed(2),
      growth: (Math.random() * 30 - 10).toFixed(1),
    })),
  }));

  return (
    <div className="my-8 overflow-x-auto">
      <h3 className="text-lg font-semibold mb-4">EPS Comparison</h3>
      <table className="w-full border border-gray-200 rounded-lg overflow-hidden">
        <thead className="bg-gray-50">
          <tr>
            <th className="px-4 py-3 text-left text-sm font-medium text-gray-700">Symbol</th>
            {Array.from({ length: quarters }, (_, i) => (
              <th key={i} className="px-4 py-3 text-center text-sm font-medium text-gray-700">
                Q{quarters - i} 2024
              </th>
            ))}
          </tr>
        </thead>
        <tbody className="divide-y divide-gray-200">
          {mockData.map((stock) => (
            <tr key={stock.symbol} className="hover:bg-gray-50">
              <td className="px-4 py-3 font-medium text-gray-900">{stock.symbol}</td>
              {stock.quarters.map((quarter, i) => (
                <td key={i} className="px-4 py-3 text-center">
                  <div className="text-sm">
                    <div className="font-medium">${quarter.eps}</div>
                    <div className={`text-xs ${
                      parseFloat(quarter.growth) > 0 ? 'text-green-600' : 'text-red-600'
                    }`}>
                      {quarter.growth}%
                    </div>
                  </div>
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

// Info Box Component
export function InfoBox({ type = 'info', title, content }: {
  type?: 'info' | 'warning' | 'success' | 'error';
  title?: string;
  content: ReactNode;
}) {
  const styles = {
    info: 'bg-blue-50 border-blue-200 text-blue-800',
    warning: 'bg-yellow-50 border-yellow-200 text-yellow-800',
    success: 'bg-green-50 border-green-200 text-green-800',
    error: 'bg-red-50 border-red-200 text-red-800',
  };

  const icons = {
    info: (
      <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
      </svg>
    ),
    warning: (
      <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.966-.833-2.732 0L3.732 16.5c-.77.833.192 2.5 1.732 2.5z" />
      </svg>
    ),
    success: (
      <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
      </svg>
    ),
    error: (
      <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
      </svg>
    ),
  };

  return (
    <div className={`my-6 p-4 border rounded-lg ${styles[type]}`}>
      <div className="flex items-start">
        <div className="flex-shrink-0 mr-3">
          {icons[type]}
        </div>
        <div className="flex-1">
          {title && (
            <h4 className="text-sm font-medium mb-2">{title}</h4>
          )}
          <div className="text-sm">{content}</div>
        </div>
      </div>
    </div>
  );
}

// Quote Component
export function Quote({ text, author, role }: {
  text: string;
  author?: string;
  role?: string;
}) {
  return (
    <blockquote className="my-8 p-6 bg-gray-50 border-l-4 border-blue-500 italic">
      <p className="text-lg text-gray-700 mb-4">"{text}"</p>
      {(author || role) && (
        <footer className="text-sm text-gray-600">
          {author && <cite className="font-medium not-italic">{author}</cite>}
          {author && role && <span>, </span>}
          {role && <span>{role}</span>}
        </footer>
      )}
    </blockquote>
  );
}

// Export all components for TinaMarkdown
export const NewsComponents = {
  StockChart,
  EPSTable,
  InfoBox,
  Quote,
  // Standard HTML elements with custom styling
  img: (props: any) => (
    <Image
      {...props}
      width={props.width || 800}
      height={props.height || 400}
      className="rounded-lg shadow-lg my-6"
      alt={props.alt || 'Article image'}
    />
  ),
  a: (props: any) => (
    <a
      {...props}
      className="text-blue-600 hover:text-blue-800 underline"
      target={props.href?.startsWith('http') ? '_blank' : undefined}
      rel={props.href?.startsWith('http') ? 'noopener noreferrer' : undefined}
    />
  ),
  blockquote: (props: any) => (
    <blockquote className="border-l-4 border-gray-300 pl-4 italic text-gray-700 my-4">
      {props.children}
    </blockquote>
  ),
  code: (props: any) => (
    <code className="bg-gray-100 px-2 py-1 rounded text-sm font-mono">
      {props.children}
    </code>
  ),
  pre: (props: any) => (
    <pre className="bg-gray-900 text-white p-4 rounded-lg overflow-x-auto my-6">
      {props.children}
    </pre>
  ),
};