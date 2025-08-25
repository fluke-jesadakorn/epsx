import { NextRequest, NextResponse } from 'next/server';
import { getServerSession } from 'next-auth';

// This will be replaced with actual TinaCMS client
// import { client } from '@/lib/tina-client';

interface NewsArticle {
  id: string;
  title: string;
  excerpt?: string;
  status: 'draft' | 'published' | 'archived';
  author?: string;
  category?: string;
  publishedAt: string;
  updatedAt?: string;
  _sys: {
    filename: string;
  };
}

// Mock data for development
const mockArticles: NewsArticle[] = [
  {
    id: '1',
    title: 'Welcome to EPSX News - Your Source for Market Intelligence',
    excerpt: 'Introducing EPSX News, your comprehensive source for market analysis...',
    status: 'published',
    author: 'admin-user',
    category: 'platform-updates',
    publishedAt: '2025-01-25T10:00:00.000Z',
    updatedAt: '2025-01-25T10:00:00.000Z',
    _sys: {
      filename: '2025/01/welcome-to-epsx-news.mdx'
    }
  }
];

// GET /api/v1/admin/news - List all articles
export async function GET(request: NextRequest) {
  try {
    // Check authentication and permissions
    const session = await getServerSession();
    if (!session?.user) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    // TODO: Check if user has news management permissions
    // const hasPermission = await checkPermission(session.user, 'news.view');
    // if (!hasPermission) {
    //   return NextResponse.json({ error: 'Insufficient permissions' }, { status: 403 });
    // }

    const { searchParams } = new URL(request.url);
    const status = searchParams.get('status');
    const category = searchParams.get('category');
    const search = searchParams.get('search');
    const page = parseInt(searchParams.get('page') || '1');
    const limit = parseInt(searchParams.get('limit') || '10');

    // TODO: Replace with actual TinaCMS query
    // const articlesQuery = await client.queries.newsConnection({
    //   filter: {
    //     ...(status && status !== 'all' && { status: { eq: status } }),
    //     ...(category && category !== 'all' && { category: { eq: category } }),
    //   },
    //   sort: 'publishedAt',
    //   first: limit,
    //   after: page > 1 ? Buffer.from(((page - 1) * limit).toString()).toString('base64') : undefined,
    // });

    // Mock response for now
    let filteredArticles = mockArticles;

    if (status && status !== 'all') {
      filteredArticles = filteredArticles.filter(a => a.status === status);
    }

    if (category && category !== 'all') {
      filteredArticles = filteredArticles.filter(a => a.category === category);
    }

    if (search) {
      filteredArticles = filteredArticles.filter(a => 
        a.title.toLowerCase().includes(search.toLowerCase()) ||
        a.excerpt?.toLowerCase().includes(search.toLowerCase())
      );
    }

    const startIndex = (page - 1) * limit;
    const endIndex = startIndex + limit;
    const paginatedArticles = filteredArticles.slice(startIndex, endIndex);

    return NextResponse.json({
      articles: paginatedArticles,
      pagination: {
        page,
        limit,
        total: filteredArticles.length,
        totalPages: Math.ceil(filteredArticles.length / limit),
        hasNext: endIndex < filteredArticles.length,
        hasPrev: page > 1,
      },
    });

  } catch (error) {
    console.error('Error fetching news articles:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}

// POST /api/v1/admin/news - Create new article
export async function POST(request: NextRequest) {
  try {
    const session = await getServerSession();
    if (!session?.user) {
      return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    }

    // TODO: Check if user has news creation permissions
    // const hasPermission = await checkPermission(session.user, 'news.create');
    // if (!hasPermission) {
    //   return NextResponse.json({ error: 'Insufficient permissions' }, { status: 403 });
    // }

    const body = await request.json();
    const {
      title,
      excerpt,
      content,
      author,
      category,
      tags,
      featuredImage,
      imageAlt,
      status,
      featured,
      publishedAt,
      seo,
    } = body;

    // Validate required fields
    if (!title || !content) {
      return NextResponse.json(
        { error: 'Title and content are required' },
        { status: 400 }
      );
    }

    // Generate filename
    const date = new Date(publishedAt || new Date());
    const year = date.getFullYear();
    const month = String(date.getMonth() + 1).padStart(2, '0');
    const slug = title
      .toLowerCase()
      .replace(/[^\w\s-]/g, '')
      .replace(/\s+/g, '-')
      .substring(0, 50);
    
    const filename = `${year}/${month}/${slug}.mdx`;

    // TODO: Replace with actual TinaCMS mutation
    // const result = await client.mutations.createNews({
    //   relativePath: filename,
    //   params: {
    //     title,
    //     excerpt,
    //     body: content,
    //     author,
    //     category,
    //     tags,
    //     featuredImage,
    //     imageAlt,
    //     status,
    //     featured,
    //     publishedAt,
    //     seo,
    //   },
    // });

    // Mock response for now
    const newArticle = {
      id: Date.now().toString(),
      title,
      excerpt,
      status,
      author,
      category,
      publishedAt,
      _sys: { filename },
    };

    return NextResponse.json({
      message: 'Article created successfully',
      article: newArticle,
    });

  } catch (error) {
    console.error('Error creating news article:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}