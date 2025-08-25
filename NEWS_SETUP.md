# EPSX News System Setup Guide

## Overview

The EPSX News System is built using TinaCMS for content management with Git-based storage. Content is stored as MDX files in the repository and can be managed through a visual editor in the admin interface.

## Architecture

```
┌─────────────────────────────────┐
│         GitHub Repository       │
│    /content/news/*.mdx files    │
└─────────────┬───────────────────┘
              │
    ┌─────────┴──────────┐
    │    TinaCMS Admin   │
    │   Visual Editor    │
    └─────────┬──────────┘
              │
    ┌─────────┴──────────┐
    │   Frontend (3000)  │
    │   News Display     │
    │   - /news          │
    │   - /news/[...]    │
    │   - /news/category │
    └────────────────────┘
```

## Environment Variables Setup

### 1. Root Configuration (.env)
```env
# TinaCMS Configuration
TINA_CLIENT_ID="your-tina-client-id"
TINA_TOKEN="your-tina-token"
TINA_BRANCH="main"
TINA_SEARCH_TOKEN="your-tina-search-token"

# Content Management
CONTENT_GITHUB_REPO="your-org/your-repo"
CONTENT_GITHUB_TOKEN="your-github-token"
```

### 2. Frontend Configuration (apps/frontend/.env.local)
```env
# TinaCMS Configuration
NEXT_PUBLIC_TINA_CLIENT_ID="your-tina-client-id"
NEXT_PUBLIC_TINA_BRANCH="main"
TINA_PUBLIC_IS_LOCAL="true"

# News & Content Management
CONTENT_GITHUB_REPO="your-org/your-repo"
CONTENT_GITHUB_TOKEN="your-github-token"
```

### 3. Admin Frontend Configuration (apps/admin-frontend/.env.local)
```env
# TinaCMS Configuration (Admin)
TINA_CLIENT_ID="your-tina-client-id"
TINA_TOKEN="your-tina-token"
TINA_BRANCH="main"
NEXT_PUBLIC_TINA_CLIENT_ID="your-tina-client-id"
TINA_PUBLIC_IS_LOCAL="true"

# News & Content Management
CONTENT_GITHUB_REPO="your-org/your-repo"
CONTENT_GITHUB_TOKEN="your-github-token"
```

## TinaCMS Setup

### 1. Create TinaCMS Account
1. Visit [tina.io](https://tina.io)
2. Sign up or log in with your GitHub account
3. Create a new project
4. Get your `TINA_CLIENT_ID` and `TINA_TOKEN`

### 2. GitHub Integration
1. Install TinaCMS GitHub app on your repository
2. Grant necessary permissions for content management
3. Generate a GitHub personal access token with repo permissions

### 3. Local Development Setup

```bash
# Install dependencies (already done)
pnpm install

# Generate TinaCMS types
npx tinacms dev -c "pnpm dev"

# Start development servers
pnpm dev:frontend  # Frontend on port 3000
pnpm dev:admin     # Admin on port 3001
```

## Content Structure

The news content follows this structure:

```
/content/
├── news/
│   └── 2025/
│       └── 01/
│           └── article-slug.mdx
├── authors.json
├── categories.json
└── media/
    └── news/
        └── 2025/
            └── images/
```

## Features Implemented

### Frontend (apps/frontend/)
- ✅ News listing page (`/news`)
- ✅ Individual article pages (`/news/[year]/[month]/[slug]`)
- ✅ Category filtering (`/news/category/[category]`)
- ✅ Search functionality
- ✅ SEO optimization with metadata
- ✅ Server-side rendering (SSR)
- ✅ Custom MDX components (StockChart, EPSTable, InfoBox, Quote)

### Admin Frontend (apps/admin-frontend/)
- ✅ News dashboard (`/news`)
- ✅ Article creation (`/news/create`)
- ✅ Article editing (`/news/edit/[id]`)
- ✅ Content management with TinaCMS
- ✅ Publishing workflow
- ✅ Author and category management

### Features Available
- ✅ **Rich Text Editing**: Visual editor with custom components
- ✅ **Git-based Storage**: Version control for all content
- ✅ **SEO Optimization**: Meta tags, Open Graph, structured data
- ✅ **Mobile-First Design**: Responsive layouts
- ✅ **Custom Components**: StockChart, EPSTable, InfoBox, Quote
- ✅ **Category System**: Organized content categories
- ✅ **Tag System**: Flexible content tagging
- ✅ **Author Management**: Multiple authors with profiles
- ✅ **Publishing Workflow**: Draft → Published → Archived

## Usage Instructions

### For Content Editors (Non-Technical)

1. **Access Admin Panel**:
   - Navigate to `http://localhost:3001/news` (dev) or your admin URL
   - Log in with your admin credentials

2. **Create New Article**:
   - Click "Create Article" button
   - Fill in title, excerpt, and content
   - Use the rich text editor for formatting
   - Add custom components via the editor
   - Set category, tags, and SEO settings
   - Save as draft or publish immediately

3. **Edit Existing Article**:
   - Find article in the news dashboard
   - Click "Edit" to modify content
   - Changes are saved to Git automatically

### For Developers

1. **Adding Custom Components**:
   - Add component to `apps/frontend/components/news/NewsComponents.tsx`
   - Update TinaCMS schema in `tina/schema/news.ts`
   - Component will be available in the rich text editor

2. **Customizing Schemas**:
   - Edit schemas in `tina/schema/` directory
   - Run `pnpm build` to regenerate types
   - New fields will appear in the admin interface

3. **Styling and Themes**:
   - Update components in `apps/frontend/components/news/`
   - Modify Tailwind classes for styling
   - Add new layouts as needed

## Permissions Setup

Add the following permission to your IAM system:

```typescript
// In your IAM configuration
{
  permission: 'content_manager',
  description: 'Manage news articles and content',
  routes: ['/news', '/news/*'],
  actions: ['create', 'read', 'update', 'delete', 'publish']
}
```

## Production Deployment

### 1. Environment Variables
Set all required environment variables in your production environment:
- TinaCMS credentials
- GitHub repository access
- Domain configurations

### 2. Build Process
```bash
# Build all applications
pnpm build

# Generate TinaCMS production build
npx tinacms build
```

### 3. Static File Serving
Ensure your deployment serves:
- `/content/media/` for uploaded images
- TinaCMS admin interface at `/admin`

## Troubleshooting

### Common Issues

1. **TinaCMS not loading**:
   - Check `TINA_CLIENT_ID` and `TINA_TOKEN` are set
   - Verify GitHub repository permissions
   - Ensure `TINA_PUBLIC_IS_LOCAL=true` for development

2. **Articles not displaying**:
   - Check file format is `.mdx`
   - Verify frontmatter format matches schema
   - Check `status: published` in article metadata

3. **Image uploads not working**:
   - Verify GitHub token has write permissions
   - Check media directory exists: `/content/media/`
   - Ensure proper file paths in TinaCMS config

### Debug Commands

```bash
# Check TinaCMS status
npx tinacms audit

# Validate content structure
npx tinacms dev --verbose

# Check file permissions
ls -la content/news/

# Test API endpoints
curl http://localhost:3000/api/tina
```

## Next Steps

1. **Content Migration**: Import existing content to MDX format
2. **SEO Enhancement**: Add structured data for articles
3. **Performance**: Implement ISR (Incremental Static Regeneration)
4. **Analytics**: Track article views and engagement
5. **Newsletter**: Integrate with email service for article notifications
6. **Comments**: Add comment system (optional)
7. **Search**: Implement full-text search with Algolia or similar

## Support

For issues with the news system:
1. Check this documentation first
2. Review TinaCMS documentation at [tina.io/docs](https://tina.io/docs)
3. Check the implementation in `/apps/frontend/app/news/` and `/apps/admin-frontend/app/news/`