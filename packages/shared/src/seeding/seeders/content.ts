import { Timestamp } from 'firebase/firestore';
import { BaseSeeder } from './base';
import type { Content, Media, Category, SeedResult } from '../types';

export class ContentSeeder extends BaseSeeder {
  get collectionName(): string {
    return 'content';
  }

  async seed(): Promise<SeedResult> {
    try {
      await this.seedCategories();
      await this.seedMedia();
      await this.seedContent();

      return {
        success: true,
        collection: 'content',
        count: 5 + 4 + 6 // categories + media + content
      };
    } catch (error) {
      return {
        success: false,
        collection: 'content',
        count: 0,
        error: error instanceof Error ? error.message : 'Unknown error'
      };
    }
  }

  private async seedCategories() {
    this.log('Seeding categories...');
    
    const categories: Category[] = [
      {
        id: 'cat_001',
        name: 'Documentation',
        slug: 'documentation',
        description: 'Platform documentation and user guides',
        parentId: null,
        sortOrder: 1,
        metadata: {
          color: '#3b82f6',
          icon: 'book-open',
          isVisible: true
        },
        createdAt: Timestamp.now(),
        updatedAt: Timestamp.now()
      },
      {
        id: 'cat_002',
        name: 'Getting Started',
        slug: 'getting-started',
        description: 'Beginner guides and tutorials',
        parentId: 'cat_001',
        sortOrder: 1,
        metadata: {
          color: '#10b981',
          icon: 'play-circle',
          isVisible: true
        },
        createdAt: Timestamp.now(),
        updatedAt: Timestamp.now()
      },
      {
        id: 'cat_003',
        name: 'API Reference',
        slug: 'api-reference',
        description: 'Complete API documentation',
        parentId: 'cat_001',
        sortOrder: 2,
        metadata: {
          color: '#8b5cf6',
          icon: 'code',
          isVisible: true
        },
        createdAt: Timestamp.now(),
        updatedAt: Timestamp.now()
      },
      {
        id: 'cat_004',
        name: 'News & Updates',
        slug: 'news-updates',
        description: 'Latest platform news and feature updates',
        parentId: null,
        sortOrder: 2,
        metadata: {
          color: '#f59e0b',
          icon: 'newspaper',
          isVisible: true
        },
        createdAt: Timestamp.now(),
        updatedAt: Timestamp.now()
      },
      {
        id: 'cat_005',
        name: 'Tutorials',
        slug: 'tutorials',
        description: 'Step-by-step tutorials and how-to guides',
        parentId: null,
        sortOrder: 3,
        metadata: {
          color: '#ef4444',
          icon: 'academic-cap',
          isVisible: true
        },
        createdAt: Timestamp.now(),
        updatedAt: Timestamp.now()
      }
    ];

    await this.seedCollection('categories', categories, 'id');
  }

  private async seedMedia() {
    this.log('Seeding media...');
    
    const media: Media[] = [
      {
        id: 'media_001',
        filename: 'epsx-logo.png',
        originalName: 'epsx-logo.png',
        mimeType: 'image/png',
        size: 25600,
        url: '/uploads/media/epsx-logo.png',
        thumbnailUrl: '/uploads/media/thumbs/epsx-logo-thumb.png',
        metadata: {
          width: 200,
          height: 80,
          uploadedBy: 'admin-001',
          folder: 'logos',
          alt: 'EPSX Platform Logo',
          caption: 'Official EPSX platform logo'
        },
        isPublic: true,
        createdAt: Timestamp.now(),
        updatedAt: Timestamp.now()
      },
      {
        id: 'media_002',
        filename: 'dashboard-screenshot.jpg',
        originalName: 'dashboard-screenshot.jpg',
        mimeType: 'image/jpeg',
        size: 156800,
        url: '/uploads/media/dashboard-screenshot.jpg',
        thumbnailUrl: '/uploads/media/thumbs/dashboard-screenshot-thumb.jpg',
        metadata: {
          width: 1920,
          height: 1080,
          uploadedBy: 'manager-001',
          folder: 'screenshots',
          alt: 'Dashboard Interface Screenshot',
          caption: 'Main dashboard interface showing analytics overview'
        },
        isPublic: true,
        createdAt: Timestamp.now(),
        updatedAt: Timestamp.now()
      },
      {
        id: 'media_003',
        filename: 'tutorial-video.mp4',
        originalName: 'getting-started-tutorial.mp4',
        mimeType: 'video/mp4',
        size: 15728640,
        url: '/uploads/media/tutorial-video.mp4',
        thumbnailUrl: '/uploads/media/thumbs/tutorial-video-thumb.jpg',
        metadata: {
          width: 1280,
          height: 720,
          duration: 180,
          uploadedBy: 'admin-001',
          folder: 'videos',
          alt: 'Getting Started Tutorial Video',
          caption: '3-minute introduction to EPSX platform features'
        },
        isPublic: true,
        createdAt: Timestamp.now(),
        updatedAt: Timestamp.now()
      },
      {
        id: 'media_004',
        filename: 'api-diagram.svg',
        originalName: 'api-architecture-diagram.svg',
        mimeType: 'image/svg+xml',
        size: 8192,
        url: '/uploads/media/api-diagram.svg',
        metadata: {
          uploadedBy: 'admin-001',
          folder: 'diagrams',
          alt: 'API Architecture Diagram',
          caption: 'Visual representation of API architecture and data flow'
        },
        isPublic: true,
        createdAt: Timestamp.now(),
        updatedAt: Timestamp.now()
      }
    ];

    await this.seedCollection('media', media, 'id');
  }

  private async seedContent() {
    this.log('Seeding content...');
    
    const now = new Date();
    const content: Content[] = [
      {
        id: 'content_001',
        type: 'documentation',
        title: 'Welcome to EPSX Platform',
        slug: 'welcome-to-epsx',
        content: {
          body: `# Welcome to EPSX Platform

Welcome to the EPSX platform - your comprehensive solution for enterprise data management and analytics.

## Getting Started

This guide will help you get up and running with EPSX in just a few minutes:

1. **Set up your account** - Complete your profile and organization settings
2. **Explore the dashboard** - Familiarize yourself with the main interface
3. **Connect your data** - Start importing your data sources
4. **Create your first report** - Generate insights from your data

## Key Features

- **Real-time Analytics**: Monitor your data in real-time with live dashboards
- **Advanced Reporting**: Create custom reports with our powerful query builder
- **Team Collaboration**: Share insights and collaborate with your team
- **API Access**: Integrate with existing systems using our REST API

## Need Help?

If you need assistance, check out our comprehensive documentation or contact our support team.`,
          excerpt: 'Get started with EPSX platform - your comprehensive enterprise data solution.',
          featuredImage: 'media_002'
        },
        metadata: {
          author: 'admin-001',
          category: 'cat_002',
          tags: ['welcome', 'getting-started', 'introduction'],
          seo: {
            metaTitle: 'Welcome to EPSX Platform - Getting Started Guide',
            metaDescription: 'Learn how to get started with EPSX platform for enterprise data management and analytics.',
            keywords: ['epsx', 'platform', 'getting started', 'tutorial']
          }
        },
        status: 'published',
        publishedAt: Timestamp.now(),
        version: 1,
        createdAt: Timestamp.now(),
        updatedAt: Timestamp.now()
      },
      {
        id: 'content_002',
        type: 'documentation',
        title: 'API Authentication Guide',
        slug: 'api-authentication',
        content: {
          body: `# API Authentication

Learn how to authenticate with the EPSX API using various methods.

## API Keys

The most common way to authenticate with our API is using API keys:

\`\`\`bash
curl -H "Authorization: Bearer YOUR_API_KEY" https://api.epsx.com/v1/data
\`\`\`

## OAuth 2.0

For applications that need to access user data on behalf of users:

1. Register your application
2. Redirect users to authorize
3. Exchange authorization code for access token
4. Use access token in API requests

## Rate Limits

- **Free Tier**: 100 requests per hour
- **Bronze**: 1,000 requests per hour
- **Silver**: 5,000 requests per hour
- **Gold**: 25,000 requests per hour
- **Enterprise**: Unlimited

## Error Handling

Common error codes and their meanings:

- \`401\`: Unauthorized - Invalid or missing authentication
- \`403\`: Forbidden - Valid auth but insufficient permissions
- \`429\`: Too Many Requests - Rate limit exceeded`,
          excerpt: 'Complete guide to authenticating with the EPSX API using API keys and OAuth.',
          featuredImage: 'media_004'
        },
        metadata: {
          author: 'admin-001',
          category: 'cat_003',
          tags: ['api', 'authentication', 'oauth', 'api-keys'],
          seo: {
            metaTitle: 'EPSX API Authentication Guide',
            metaDescription: 'Learn how to authenticate with EPSX API using API keys, OAuth 2.0, and handle rate limits.',
            keywords: ['api', 'authentication', 'oauth', 'api keys', 'rate limits']
          }
        },
        status: 'published',
        publishedAt: Timestamp.fromDate(this.addDays(now, -5)),
        version: 2,
        createdAt: Timestamp.fromDate(this.addDays(now, -7)),
        updatedAt: Timestamp.fromDate(this.addDays(now, -1))
      },
      {
        id: 'content_003',
        type: 'article',
        title: 'Announcing EPSX 2.0: Enhanced Analytics and Performance',
        slug: 'epsx-2-0-announcement',
        content: {
          body: `# Announcing EPSX 2.0

We're excited to announce the release of EPSX 2.0 with major improvements to analytics and performance.

## What's New

### Enhanced Analytics Engine
- 10x faster query processing
- Real-time data streaming
- Advanced machine learning insights

### Improved User Interface
- Redesigned dashboard with better visualization
- Mobile-responsive design
- Dark mode support

### New Integrations
- Salesforce connector
- Slack notifications
- Microsoft Teams integration

## Upgrade Guide

Existing users will be automatically upgraded over the next week. No action is required on your part.

## Feedback

We'd love to hear your thoughts on the new features. Join our beta testing program to help shape future releases.`,
          excerpt: 'Major release with enhanced analytics, improved UI, and new integrations.',
          featuredImage: 'media_002'
        },
        metadata: {
          author: 'manager-001',
          category: 'cat_004',
          tags: ['release', 'announcement', 'v2.0', 'analytics'],
          seo: {
            metaTitle: 'EPSX 2.0 Release - Enhanced Analytics and Performance',
            metaDescription: 'Discover the new features in EPSX 2.0 including enhanced analytics, improved UI, and new integrations.',
            keywords: ['epsx 2.0', 'release', 'analytics', 'performance', 'features']
          }
        },
        status: 'published',
        publishedAt: Timestamp.fromDate(this.addDays(now, -2)),
        version: 1,
        createdAt: Timestamp.fromDate(this.addDays(now, -3)),
        updatedAt: Timestamp.fromDate(this.addDays(now, -2))
      },
      {
        id: 'content_004',
        type: 'article',
        title: 'Building Custom Dashboards: A Step-by-Step Tutorial',
        slug: 'custom-dashboards-tutorial',
        content: {
          body: `# Building Custom Dashboards

Learn how to create powerful custom dashboards that meet your specific needs.

## Prerequisites

Before starting, make sure you have:
- Access to EPSX platform (Silver tier or higher)
- Data sources configured
- Basic understanding of your data structure

## Step 1: Planning Your Dashboard

Consider these questions:
- What key metrics do you want to track?
- Who will be using this dashboard?
- How often will it be viewed?

## Step 2: Creating Your First Widget

1. Navigate to the Dashboard builder
2. Click "Add Widget"
3. Select your data source
4. Choose visualization type
5. Configure display options

## Step 3: Layout and Design

- Use the drag-and-drop interface to arrange widgets
- Apply consistent color schemes
- Add filters for interactivity

## Best Practices

- Keep it simple and focused
- Use meaningful titles and labels
- Test with actual users
- Regular review and updates`,
          excerpt: 'Step-by-step guide to creating powerful custom dashboards in EPSX.',
          featuredImage: 'media_002'
        },
        metadata: {
          author: 'beta-001',
          category: 'cat_005',
          tags: ['tutorial', 'dashboards', 'visualization', 'custom'],
          seo: {
            metaTitle: 'Custom Dashboards Tutorial - EPSX Platform',
            metaDescription: 'Learn how to build custom dashboards in EPSX with this comprehensive step-by-step tutorial.',
            keywords: ['custom dashboards', 'tutorial', 'data visualization', 'epsx']
          }
        },
        status: 'published',
        publishedAt: Timestamp.fromDate(this.addDays(now, -10)),
        version: 1,
        createdAt: Timestamp.fromDate(this.addDays(now, -12)),
        updatedAt: Timestamp.fromDate(this.addDays(now, -8))
      },
      {
        id: 'content_005',
        type: 'documentation',
        title: 'Data Import and Export Guide',
        slug: 'data-import-export',
        content: {
          body: `# Data Import and Export

Comprehensive guide for importing and exporting data in EPSX.

## Supported Formats

### Import
- CSV files
- JSON data
- Excel spreadsheets
- Database connections (MySQL, PostgreSQL, MongoDB)
- API integrations

### Export
- CSV format
- PDF reports
- Excel workbooks
- JSON data
- API endpoints

## Import Process

1. **Prepare your data**
   - Ensure clean, consistent formatting
   - Remove duplicates and errors
   - Validate data types

2. **Upload your file**
   - Use the web interface or API
   - Support for files up to 100MB
   - Batch processing for larger datasets

3. **Map your fields**
   - Match source columns to EPSX fields
   - Set data types and validation rules
   - Preview before final import

## Export Options

Configure exports with:
- Custom date ranges
- Filtered datasets
- Scheduled exports
- Multiple output formats`,
          excerpt: 'Complete guide for importing and exporting data in various formats.',
          featuredImage: 'media_004'
        },
        metadata: {
          author: 'admin-001',
          category: 'cat_001',
          tags: ['data', 'import', 'export', 'csv', 'api'],
          seo: {
            metaTitle: 'Data Import Export Guide - EPSX Platform',
            metaDescription: 'Learn how to import and export data in EPSX platform with support for CSV, JSON, Excel and more.',
            keywords: ['data import', 'data export', 'csv', 'excel', 'api integration']
          }
        },
        status: 'published',
        publishedAt: Timestamp.fromDate(this.addDays(now, -15)),
        version: 3,
        createdAt: Timestamp.fromDate(this.addDays(now, -20)),
        updatedAt: Timestamp.fromDate(this.addDays(now, -3))
      },
      {
        id: 'content_006',
        type: 'page',
        title: 'Privacy Policy',
        slug: 'privacy-policy',
        content: {
          body: `# Privacy Policy

Last updated: ${new Date().toLocaleDateString()}

## Introduction

EPSX Corporation ("we," "our," or "us") respects your privacy and is committed to protecting your personal data.

## Information We Collect

### Personal Information
- Name and contact information
- Account credentials
- Usage data and analytics
- Communication preferences

### Technical Information
- IP addresses and device information
- Browser type and version
- Operating system
- Cookies and tracking technologies

## How We Use Your Information

We use your information to:
- Provide and improve our services
- Communicate with you
- Ensure security and prevent fraud
- Comply with legal obligations

## Data Sharing

We do not sell your personal information. We may share data with:
- Service providers and partners
- Legal authorities when required
- With your explicit consent

## Your Rights

You have the right to:
- Access your personal data
- Correct inaccurate information
- Delete your account and data
- Export your data
- Opt-out of marketing communications

## Contact Us

If you have questions about this privacy policy, contact us at privacy@epsx.com.`,
          excerpt: 'Our commitment to protecting your privacy and personal data.'
        },
        metadata: {
          author: 'admin-001',
          category: 'cat_001',
          tags: ['privacy', 'policy', 'legal', 'gdpr'],
          seo: {
            metaTitle: 'Privacy Policy - EPSX Platform',
            metaDescription: 'Read our privacy policy to understand how EPSX protects and uses your personal data.',
            keywords: ['privacy policy', 'data protection', 'gdpr', 'personal data']
          }
        },
        status: 'published',
        publishedAt: Timestamp.fromDate(this.addDays(now, -30)),
        version: 1,
        createdAt: Timestamp.fromDate(this.addDays(now, -35)),
        updatedAt: Timestamp.fromDate(this.addDays(now, -25))
      }
    ];

    await this.seedCollection('content', content, 'id');
  }
}
