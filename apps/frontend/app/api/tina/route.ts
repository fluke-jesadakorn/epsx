import { NextRequest, NextResponse } from 'next/server';

// TinaCMS API proxy for frontend
export async function GET(request: NextRequest) {
  try {
    // This will proxy TinaCMS GraphQL requests
    const { searchParams } = new URL(request.url);
    const query = searchParams.get('query');
    const variables = searchParams.get('variables');

    if (!query) {
      return NextResponse.json({ error: 'No query provided' }, { status: 400 });
    }

    // TODO: Implement actual TinaCMS GraphQL proxy
    // For now, return mock data based on query type
    
    if (query.includes('newsConnection')) {
      const mockResponse = {
        data: {
          newsConnection: {
            edges: [
              {
                node: {
                  id: '1',
                  title: 'Welcome to EPSX News - Your Source for Market Intelligence',
                  excerpt: 'Introducing EPSX News, your comprehensive source for market analysis, earnings insights, and platform updates.',
                  publishedAt: '2025-01-25T10:00:00.000Z',
                  author: 'admin-user',
                  category: 'platform-updates',
                  tags: ['platform', 'announcement', 'news', 'launch'],
                  featuredImage: '/content/media/news/2025/welcome-banner.jpg',
                  imageAlt: 'Welcome to EPSX News',
                  status: 'published',
                  featured: true,
                  readTime: 5,
                  _sys: {
                    filename: '2025/01/welcome-to-epsx-news.mdx',
                  },
                },
              },
            ],
            pageInfo: {
              hasNextPage: false,
              hasPreviousPage: false,
            },
          },
        },
      };

      return NextResponse.json(mockResponse);
    }

    if (query.includes('news(')) {
      const mockResponse = {
        data: {
          news: {
            id: '1',
            title: 'Welcome to EPSX News - Your Source for Market Intelligence',
            excerpt: 'Introducing EPSX News, your comprehensive source for market analysis, earnings insights, and platform updates.',
            publishedAt: '2025-01-25T10:00:00.000Z',
            author: 'admin-user',
            category: 'platform-updates',
            tags: ['platform', 'announcement', 'news', 'launch'],
            featuredImage: '/content/media/news/2025/welcome-banner.jpg',
            imageAlt: 'Welcome to EPSX News',
            status: 'published',
            featured: true,
            readTime: 5,
            seo: {
              title: 'Welcome to EPSX News - Market Intelligence Hub',
              description: 'Stay updated with the latest market analysis, earnings reports, and EPS insights from EPSX platform experts.',
              keywords: ['EPSX news', 'market analysis', 'earnings reports', 'EPS growth'],
            },
            body: {
              type: 'root',
              children: [
                {
                  type: 'h1',
                  children: [{ type: 'text', text: 'Welcome to EPSX News' }],
                },
                {
                  type: 'p',
                  children: [
                    { 
                      type: 'text', 
                      text: "We're excited to announce the launch of EPSX News, your comprehensive source for market intelligence, earnings analysis, and platform updates." 
                    },
                  ],
                },
                {
                  type: 'h2',
                  children: [{ type: 'text', text: 'What You\'ll Find Here' }],
                },
                {
                  type: 'p',
                  children: [
                    { type: 'text', text: 'Our news section brings you:' },
                  ],
                },
                {
                  type: 'ul',
                  children: [
                    {
                      type: 'li',
                      children: [
                        { type: 'text', text: '**Market Analysis**: Deep dives into market trends and opportunities' },
                      ],
                    },
                    {
                      type: 'li',
                      children: [
                        { type: 'text', text: '**Earnings Reports**: Comprehensive analysis of company earnings and forecasts' },
                      ],
                    },
                    {
                      type: 'li',
                      children: [
                        { type: 'text', text: '**EPS Insights**: Expert analysis on earnings per share growth patterns' },
                      ],
                    },
                    {
                      type: 'li',
                      children: [
                        { type: 'text', text: '**Platform Updates**: Latest features and improvements to the EPSX platform' },
                      ],
                    },
                  ],
                },
              ],
            },
            _sys: {
              filename: '2025/01/welcome-to-epsx-news.mdx',
            },
          },
        },
      };

      return NextResponse.json(mockResponse);
    }

    return NextResponse.json({ error: 'Unknown query' }, { status: 400 });

  } catch (error) {
    console.error('TinaCMS API error:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}

export async function POST(request: NextRequest) {
  try {
    const body = await request.json();
    const { query, variables } = body;

    // TODO: Forward to actual TinaCMS endpoint
    console.log('TinaCMS POST request:', { query, variables });

    // Mock response for mutations
    return NextResponse.json({
      data: {
        success: true,
        message: 'Operation completed successfully',
      },
    });

  } catch (error) {
    console.error('TinaCMS POST error:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}