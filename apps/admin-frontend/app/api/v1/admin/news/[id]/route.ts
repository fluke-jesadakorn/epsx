import { NextRequest, NextResponse } from 'next/server';

interface RouteParams {
  params: {
    id: string;
  };
}

// GET /api/v1/admin/news/[id] - Get single article
export async function GET(request: NextRequest, { params }: RouteParams) {
  try {
    // TODO: Add authentication when ready
    // const session = await getSession();
    // if (!session?.user) {
    //   return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    // }

    const articleId = decodeURIComponent(params.id);

    // TODO: Replace with actual TinaCMS query
    // const articleQuery = await client.queries.news({
    //   relativePath: articleId.endsWith('.mdx') ? articleId : `${articleId}.mdx`,
    // });

    // Mock response for now
    if (articleId.includes('welcome-to-epsx-news')) {
      const mockArticle = {
        id: '1',
        title: 'Welcome to EPSX News - Your Source for Market Intelligence',
        excerpt: 'Introducing EPSX News, your comprehensive source for market analysis...',
        status: 'published',
        author: 'admin-user',
        category: 'platform-updates',
        tags: ['platform', 'announcement', 'news', 'launch'],
        featuredImage: '/content/media/news/2025/welcome-banner.jpg',
        imageAlt: 'Welcome to EPSX News banner',
        featured: false,
        publishedAt: '2025-01-25T10:00:00.000Z',
        seo: {
          title: 'Welcome to EPSX News - Market Intelligence Hub',
          description: 'Stay updated with the latest market analysis from EPSX',
          keywords: ['EPSX news', 'market analysis', 'earnings reports'],
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
              children: [{ type: 'text', text: 'Content would be here...' }],
            },
          ],
        },
        _sys: {
          filename: '2025/01/welcome-to-epsx-news.mdx',
        },
      };

      return NextResponse.json({ article: mockArticle });
    }

    return NextResponse.json({ error: 'Article not found' }, { status: 404 });

  } catch (error) {
    console.error('Error fetching article:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}

// PUT /api/v1/admin/news/[id] - Update article
export async function PUT(request: NextRequest, { params }: RouteParams) {
  try {
    // TODO: Add authentication when ready
    // const session = await getSession();
    // if (!session?.user) {
    //   return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    // }

    // TODO: Check if user has news edit permissions
    // const hasPermission = await checkPermission(session.user, 'news.edit');
    // if (!hasPermission) {
    //   return NextResponse.json({ error: 'Insufficient permissions' }, { status: 403 });
    // }

    const articleId = decodeURIComponent(params.id);
    const body = await request.json();

    // TODO: Replace with actual TinaCMS mutation
    // const result = await client.mutations.updateNews({
    //   relativePath: articleId.endsWith('.mdx') ? articleId : `${articleId}.mdx`,
    //   params: body,
    // });

    // Mock response for now
    console.log('Updating article:', articleId, body);

    return NextResponse.json({
      message: 'Article updated successfully',
      article: { ...body, id: articleId },
    });

  } catch (error) {
    console.error('Error updating article:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}

// DELETE /api/v1/admin/news/[id] - Delete article
export async function DELETE(request: NextRequest, { params }: RouteParams) {
  try {
    // TODO: Add authentication when ready
    // const session = await getSession();
    // if (!session?.user) {
    //   return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    // }

    // TODO: Check if user has news delete permissions
    // const hasPermission = await checkPermission(session.user, 'news.delete');
    // if (!hasPermission) {
    //   return NextResponse.json({ error: 'Insufficient permissions' }, { status: 403 });
    // }

    const articleId = decodeURIComponent(params.id);

    // TODO: Replace with actual TinaCMS deletion
    // const result = await client.mutations.deleteNews({
    //   relativePath: articleId.endsWith('.mdx') ? articleId : `${articleId}.mdx`,
    // });

    // Mock response for now
    console.log('Deleting article:', articleId);

    return NextResponse.json({
      message: 'Article deleted successfully',
    });

  } catch (error) {
    console.error('Error deleting article:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}

// PATCH /api/v1/admin/news/[id] - Update article status
export async function PATCH(request: NextRequest, { params }: RouteParams) {
  try {
    // TODO: Add authentication when ready
    // const session = await getSession();
    // if (!session?.user) {
    //   return NextResponse.json({ error: 'Unauthorized' }, { status: 401 });
    // }

    // TODO: Check if user has news publish permissions
    // const hasPermission = await checkPermission(session.user, 'news.publish');
    // if (!hasPermission) {
    //   return NextResponse.json({ error: 'Insufficient permissions' }, { status: 403 });
    // }

    const articleId = decodeURIComponent(params.id);
    const { status } = await request.json();

    if (!['draft', 'published', 'archived'].includes(status)) {
      return NextResponse.json(
        { error: 'Invalid status' },
        { status: 400 }
      );
    }

    // TODO: Replace with actual TinaCMS mutation
    // const result = await client.mutations.updateNews({
    //   relativePath: articleId.endsWith('.mdx') ? articleId : `${articleId}.mdx`,
    //   params: { 
    //     status,
    //     ...(status === 'published' && { publishedAt: new Date().toISOString() }),
    //   },
    // });

    // Mock response for now
    console.log('Updating article status:', articleId, status);

    return NextResponse.json({
      message: `Article ${status} successfully`,
      status,
    });

  } catch (error) {
    console.error('Error updating article status:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}