import { Metadata } from 'next';
import { notFound } from 'next/navigation';
import { database } from '@/lib/tina-client';
import { NewsGrid } from '@/components/news/NewsGrid';
import { NewsFilters } from '@/components/news/NewsFilters';
import { NewsHero } from '@/components/news/NewsHero';

export const metadata: Metadata = {
  title: 'Market News & Analysis | EPSX',
  description: 'Stay updated with the latest market analysis, earnings reports, and EPS insights from EPSX platform experts.',
  keywords: ['market news', 'earnings analysis', 'EPS insights', 'stock analysis', 'trading platform'],
  openGraph: {
    title: 'Market News & Analysis | EPSX',
    description: 'Your source for market intelligence and earnings analysis',
    images: ['/images/news-og.jpg'],
  },
};

export const revalidate = 3600; // ISR: revalidate every hour

interface NewsPageProps {
  searchParams: Promise<{
    category?: string;
    tag?: string;
    page?: string;
  }>;
}

export default async function NewsPage({ searchParams }: NewsPageProps) {
  try {
    const params = await searchParams;
    const page = parseInt(params.page || '1');
    const perPage = 12;
    const offset = (page - 1) * perPage;

    // Build filter conditions
    const filters: any = {
      status: { eq: 'published' },
    };

    if (params.category) {
      filters.category = { category: { eq: params.category } };
    }

    if (params.tag) {
      filters.tags = { in: [params.tag] };
    }

    // Mock data for testing (replace with TinaCMS queries once properly configured)
    const mockArticle = {
      id: '1',
      title: 'Sample EPS Analysis Report',
      excerpt: 'A comprehensive analysis of recent earnings per share trends in the tech sector.',
      content: 'This is a sample news article demonstrating the EPS news system functionality.',
      publishedAt: '2025-01-25T10:00:00Z',
      category: 'earnings',
      tags: ['EPS', 'technology', 'analysis'],
      author: 'EPSX Research Team',
      image: '/images/news/eps-analysis.jpg',
      featured: true,
      status: 'published'
    };

    const articles = [
      { node: mockArticle },
      { node: { ...mockArticle, id: '2', title: 'Market Trends Update', featured: false } },
      { node: { ...mockArticle, id: '3', title: 'Sector Analysis', featured: false } }
    ];

    const featuredArticle = { node: mockArticle };
    const pageInfo = {
      hasPreviousPage: false,
      hasNextPage: false,
      startCursor: '',
      endCursor: ''
    };

    return (
      <div className="container mx-auto px-4 py-8">
        {featuredArticle && (
          <NewsHero article={featuredArticle.node} />
        )}
        
        <div className="mt-12">
          <div className="flex justify-between items-center mb-8">
            <h1 className="text-4xl font-bold">Market News & Analysis</h1>
          </div>
          
          <NewsFilters 
            currentCategory={params.category}
            currentTag={params.tag}
          />
          
          <NewsGrid articles={articles} />
          
          {/* Pagination */}
          {(pageInfo.hasPreviousPage || pageInfo.hasNextPage) && (
            <div className="flex justify-center items-center mt-12 space-x-4">
              {pageInfo.hasPreviousPage && (
                <a
                  href={`/news?${new URLSearchParams({
                    ...params,
                    page: (page - 1).toString(),
                  }).toString()}`}
                  className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
                >
                  Previous
                </a>
              )}
              
              <span className="px-4 py-2 text-gray-600">
                Page {page}
              </span>
              
              {pageInfo.hasNextPage && (
                <a
                  href={`/news?${new URLSearchParams({
                    ...params,
                    page: (page + 1).toString(),
                  }).toString()}`}
                  className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
                >
                  Next
                </a>
              )}
            </div>
          )}
        </div>
      </div>
    );
  } catch (error) {
    console.error('Error loading news:', error);
    notFound();
  }
}