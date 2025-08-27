import { Metadata } from 'next';
import { notFound } from 'next/navigation';
// Stub TinaMarkdown component (TinaCMS removed to fix axios conflicts)
const TinaMarkdown = ({ content, components }: { content: any; components?: any }) => {
  return <div>News content not available (TinaCMS disabled)</div>;
};
import { database } from '@/lib/tina-client';
import { NewsArticleLayout } from '@/components/news/NewsArticleLayout';
import { RelatedArticles } from '@/components/news/RelatedArticles';
import { NewsComponents } from '@/components/news/NewsComponents';

interface ArticlePageProps {
  params: {
    year: string;
    month: string;
    slug: string;
  };
}

export async function generateStaticParams() {
  try {
    const newsQuery = await database.queries.newsConnection({
      first: 100,
      filter: {
        status: { eq: 'published' },
      },
    });

    return newsQuery.data.newsConnection.edges?.map((article) => {
      const pathParts = article.node._sys.filename.replace('.mdx', '').split('/');
      return {
        year: pathParts[0] || '2025',
        month: pathParts[1] || '01',
        slug: pathParts[2] || 'article',
      };
    }) || [];
  } catch (error) {
    console.error('Error generating static params:', error);
    return [];
  }
}

export async function generateMetadata({ params }: ArticlePageProps): Promise<Metadata> {
  try {
    const relativePath = `${params.year}/${params.month}/${params.slug}.mdx`;
    const newsQuery = await database.queries.news({
      relativePath,
    });

    if (!newsQuery.data.news) {
      return {
        title: 'Article Not Found | EPSX News',
      };
    }

    const article = newsQuery.data.news;

    return {
      title: article.seo?.title || `${article.title} | EPSX News`,
      description: article.seo?.description || article.excerpt,
      keywords: article.seo?.keywords || article.tags,
      authors: [{ name: 'EPSX Team' }], // Will be updated with actual author data
      publishedTime: article.publishedAt,
      openGraph: {
        title: article.seo?.title || article.title,
        description: article.seo?.description || article.excerpt,
        type: 'article',
        publishedTime: article.publishedAt,
        images: article.featuredImage ? [
          {
            url: article.featuredImage,
            alt: article.imageAlt || article.title,
          },
        ] : [],
        tags: article.tags || [],
      },
      twitter: {
        card: 'summary_large_image',
        title: article.seo?.title || article.title,
        description: article.seo?.description || article.excerpt,
        images: article.featuredImage ? [article.featuredImage] : [],
      },
    };
  } catch (error) {
    console.error('Error generating metadata:', error);
    return {
      title: 'Article Not Found | EPSX News',
    };
  }
}

export default async function ArticlePage({ params }: ArticlePageProps) {
  try {
    const relativePath = `${params.year}/${params.month}/${params.slug}.mdx`;
    
    const newsQuery = await database.queries.news({
      relativePath,
    });

    if (!newsQuery.data.news) {
      notFound();
    }

    const article = newsQuery.data.news;

    // Fetch related articles based on tags
    const relatedQuery = await database.queries.newsConnection({
      first: 3,
      filter: {
        status: { eq: 'published' },
        _sys: {
          filename: {
            ne: relativePath,
          },
        },
        // TODO: Add tag-based filtering when TinaCMS supports it
      },
      sort: 'publishedAt',
    });

    const relatedArticles = relatedQuery.data.newsConnection.edges || [];

    return (
      <NewsArticleLayout article={article}>
        <article className="max-w-4xl mx-auto px-4 py-8">
          {/* Article Header */}
          <header className="mb-8">
            {article.category && (
              <div className="mb-4">
                <span className="inline-block px-3 py-1 text-sm bg-blue-100 text-blue-800 rounded-full">
                  {article.category}
                </span>
              </div>
            )}
            
            <h1 className="text-4xl md:text-5xl font-bold mb-4 leading-tight">
              {article.title}
            </h1>
            
            {article.excerpt && (
              <p className="text-xl text-gray-600 mb-6 leading-relaxed">
                {article.excerpt}
              </p>
            )}
            
            <div className="flex flex-wrap items-center gap-4 text-sm text-gray-500 border-b border-gray-200 pb-6">
              <time dateTime={article.publishedAt}>
                {new Date(article.publishedAt).toLocaleDateString('en-US', {
                  year: 'numeric',
                  month: 'long',
                  day: 'numeric',
                })}
              </time>
              
              {article.author && (
                <span>By {article.author}</span>
              )}
              
              {article.readTime && (
                <span>{article.readTime} min read</span>
              )}
            </div>
          </header>

          {/* Featured Image */}
          {article.featuredImage && (
            <div className="mb-8">
              <img
                src={article.featuredImage}
                alt={article.imageAlt || article.title}
                className="w-full h-auto rounded-lg shadow-lg"
              />
            </div>
          )}

          {/* Article Content */}
          <div className="prose prose-lg prose-blue max-w-none">
            <TinaMarkdown 
              content={article.body} 
              components={NewsComponents}
            />
          </div>

          {/* Tags */}
          {article.tags && article.tags.length > 0 && (
            <div className="mt-12 pt-8 border-t border-gray-200">
              <h3 className="text-sm font-medium text-gray-500 mb-4">Tags</h3>
              <div className="flex flex-wrap gap-2">
                {article.tags.map((tag) => (
                  <a
                    key={tag}
                    href={`/news?tag=${encodeURIComponent(tag)}`}
                    className="inline-block px-3 py-1 text-sm bg-gray-100 text-gray-700 rounded-full hover:bg-gray-200 transition-colors"
                  >
                    {tag}
                  </a>
                ))}
              </div>
            </div>
          )}
        </article>

        {/* Related Articles */}
        {relatedArticles.length > 0 && (
          <RelatedArticles 
            articles={relatedArticles}
            currentArticleId={article.id}
          />
        )}
      </NewsArticleLayout>
    );
  } catch (error) {
    console.error('Error loading article:', error);
    notFound();
  }
}