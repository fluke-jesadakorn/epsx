import { Metadata } from 'next';
import { notFound } from 'next/navigation';
import { database } from '@/lib/tina-client';
import { NewsGrid } from '@/components/news/NewsGrid';

interface CategoryPageProps {
  params: {
    category: string;
  };
  searchParams: {
    page?: string;
  };
}

export async function generateStaticParams() {
  try {
    // This would be enhanced to read from categories.json
    const categories = ['market-analysis', 'earnings-reports', 'eps-insights', 'platform-updates'];
    
    return categories.map((category) => ({
      category,
    }));
  } catch (error) {
    console.error('Error generating category params:', error);
    return [];
  }
}

export async function generateMetadata({ params }: CategoryPageProps): Promise<Metadata> {
  const categoryNames: Record<string, string> = {
    'market-analysis': 'Market Analysis',
    'earnings-reports': 'Earnings Reports',
    'eps-insights': 'EPS Insights',
    'platform-updates': 'Platform Updates',
  };

  const categoryName = categoryNames[params.category] || params.category;

  return {
    title: `${categoryName} | EPSX News`,
    description: `Latest ${categoryName.toLowerCase()} articles from EPSX market experts.`,
    openGraph: {
      title: `${categoryName} | EPSX News`,
      description: `Stay updated with the latest ${categoryName.toLowerCase()} from EPSX platform.`,
    },
  };
}

export default async function CategoryPage({ params, searchParams }: CategoryPageProps) {
  try {
    const page = parseInt(searchParams.page || '1');
    const perPage = 12;
    const offset = (page - 1) * perPage;

    const newsQuery = await database.queries.newsConnection({
      first: perPage,
      after: offset > 0 ? Buffer.from(offset.toString()).toString('base64') : undefined,
      filter: {
        status: { eq: 'published' },
        category: { category: { eq: params.category } },
      },
      sort: 'publishedAt',
    });

    if (!newsQuery.data.newsConnection) {
      notFound();
    }

    const articles = newsQuery.data.newsConnection.edges || [];
    const pageInfo = newsQuery.data.newsConnection.pageInfo;

    const categoryNames: Record<string, string> = {
      'market-analysis': 'Market Analysis',
      'earnings-reports': 'Earnings Reports',
      'eps-insights': 'EPS Insights',
      'platform-updates': 'Platform Updates',
    };

    const categoryName = categoryNames[params.category] || params.category;

    if (articles.length === 0 && page === 1) {
      return (
        <div className="container mx-auto px-4 py-8">
          <h1 className="text-4xl font-bold mb-8">{categoryName}</h1>
          <div className="text-center py-16">
            <p className="text-gray-600 text-lg">
              No articles found in this category yet.
            </p>
            <a 
              href="/news"
              className="inline-block mt-4 px-6 py-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
            >
              View All Articles
            </a>
          </div>
        </div>
      );
    }

    return (
      <div className="container mx-auto px-4 py-8">
        <div className="mb-8">
          <nav className="text-sm text-gray-600 mb-4">
            <a href="/news" className="hover:text-blue-600">News</a>
            <span className="mx-2">/</span>
            <span>{categoryName}</span>
          </nav>
          
          <h1 className="text-4xl font-bold">{categoryName}</h1>
          <p className="mt-4 text-gray-600 text-lg">
            Latest {categoryName.toLowerCase()} from our expert team.
          </p>
        </div>

        <NewsGrid articles={articles} />

        {/* Pagination */}
        {(pageInfo.hasPreviousPage || pageInfo.hasNextPage) && (
          <div className="flex justify-center items-center mt-12 space-x-4">
            {pageInfo.hasPreviousPage && (
              <a
                href={`/news/category/${params.category}?page=${page - 1}`}
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
                href={`/news/category/${params.category}?page=${page + 1}`}
                className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
              >
                Next
              </a>
            )}
          </div>
        )}
      </div>
    );
  } catch (error) {
    console.error('Error loading category:', error);
    notFound();
  }
}