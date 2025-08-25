import Link from 'next/link';
import Image from 'next/image';

interface NewsHeroProps {
  article: {
    id: string;
    title: string;
    excerpt?: string;
    publishedAt: string;
    author?: string;
    category?: string;
    featuredImage?: string;
    imageAlt?: string;
    readTime?: number;
    _sys?: {
      filename: string;
    };
  };
}

export function NewsHero({ article }: NewsHeroProps) {
  // Create URL path from article ID or filename  
  const urlPath = article._sys?.filename 
    ? `/news/${article._sys.filename.replace('.mdx', '')}` 
    : `/news/2025/01/article-${article.id}`;
  
  const publishDate = new Date(article.publishedAt).toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'long',
    day: 'numeric',
  });

  return (
    <section className="relative bg-gradient-to-r from-blue-600 to-blue-800 text-white rounded-2xl overflow-hidden">
      <div className="absolute inset-0 bg-black bg-opacity-20"></div>
      
      <div className="relative">
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-8 items-center">
          {/* Content */}
          <div className="p-8 lg:p-12">
            <div className="mb-4">
              <span className="inline-block px-3 py-1 bg-white bg-opacity-20 rounded-full text-sm font-medium">
                Featured Story
              </span>
            </div>
            
            <h1 className="text-3xl lg:text-4xl xl:text-5xl font-bold mb-4 leading-tight">
              {article.title}
            </h1>
            
            {article.excerpt && (
              <p className="text-xl text-blue-100 mb-6 leading-relaxed line-clamp-3">
                {article.excerpt}
              </p>
            )}
            
            <div className="flex items-center space-x-6 text-sm text-blue-100 mb-8">
              <time dateTime={article.publishedAt}>
                {publishDate}
              </time>
              
              {article.readTime && (
                <>
                  <span>•</span>
                  <span>{article.readTime} min read</span>
                </>
              )}
              
              {article.author && (
                <>
                  <span>•</span>
                  <span>By {article.author}</span>
                </>
              )}
            </div>
            
            <Link 
              href={urlPath}
              className="inline-flex items-center px-6 py-3 bg-white text-blue-600 font-semibold rounded-lg hover:bg-blue-50 transition-colors group"
            >
              Read Full Article
              <svg 
                className="ml-2 w-4 h-4 group-hover:translate-x-1 transition-transform" 
                fill="none" 
                stroke="currentColor" 
                viewBox="0 0 24 24"
              >
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
              </svg>
            </Link>
          </div>
          
          {/* Featured Image */}
          {(article.featuredImage || article.image) && (
            <div className="relative h-64 lg:h-96 overflow-hidden">
              <Image
                src={article.featuredImage || article.image || '/images/news/default.jpg'}
                alt={article.imageAlt || article.title}
                fill
                className="object-cover"
              />
              <div className="absolute inset-0 bg-gradient-to-l from-transparent via-transparent to-blue-600 opacity-60"></div>
            </div>
          )}
        </div>
      </div>
    </section>
  );
}