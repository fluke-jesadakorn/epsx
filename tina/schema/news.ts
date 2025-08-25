import type { Collection } from 'tinacms';

export const newsSchema: Collection = {
  name: 'news',
  label: 'News Articles',
  path: 'content/news',
  format: 'mdx',
  
  ui: {
    router: ({ document }) => {
      const pathParts = document._sys.filename.split('/');
      return `/news/${pathParts.join('/')}`;
    },
    filename: {
      readonly: false,
      slugify: (values) => {
        const date = new Date();
        const year = date.getFullYear();
        const month = String(date.getMonth() + 1).padStart(2, '0');
        const slug = values?.title
          ?.toLowerCase()
          ?.replace(/ /g, '-')
          ?.replace(/[^\w-]+/g, '')
          ?.substring(0, 50);
        return `${year}/${month}/${slug}`;
      },
    },
  },
  
  fields: [
    {
      name: 'title',
      type: 'string',
      label: 'Title',
      isTitle: true,
      required: true,
      ui: {
        validate: (value) => {
          if (value && value.length > 200) {
            return 'Title must be less than 200 characters';
          }
        },
      },
    },
    {
      name: 'excerpt',
      type: 'string',
      label: 'Excerpt',
      required: true,
      ui: {
        component: 'textarea',
        validate: (value) => {
          if (value && value.length > 300) {
            return 'Excerpt must be less than 300 characters';
          }
        },
      },
    },
    {
      name: 'publishedAt',
      type: 'datetime',
      label: 'Published Date',
      required: true,
      ui: {
        dateFormat: 'MMMM DD YYYY',
        timeFormat: 'hh:mm A',
      },
    },
    {
      name: 'author',
      type: 'reference',
      label: 'Author',
      collections: ['authors'],
      required: true,
    },
    {
      name: 'category',
      type: 'reference',
      label: 'Category',
      collections: ['categories'],
      required: true,
    },
    {
      name: 'tags',
      type: 'string',
      label: 'Tags',
      list: true,
      ui: {
        component: 'tags',
      },
    },
    {
      name: 'featuredImage',
      type: 'image',
      label: 'Featured Image',
      ui: {
        previewSrc: (fullSrc) => fullSrc?.replace('/content/', '/'),
      },
    },
    {
      name: 'imageAlt',
      type: 'string',
      label: 'Featured Image Alt Text',
    },
    {
      name: 'seo',
      type: 'object',
      label: 'SEO Settings',
      ui: {
        defaultItem: {
          title: '',
          description: '',
          keywords: [],
        },
      },
      fields: [
        {
          name: 'title',
          type: 'string',
          label: 'SEO Title',
          ui: {
            validate: (value) => {
              if (value && value.length > 60) {
                return 'SEO title should be less than 60 characters';
              }
            },
          },
        },
        {
          name: 'description',
          type: 'string',
          label: 'SEO Description',
          ui: {
            component: 'textarea',
            validate: (value) => {
              if (value && value.length > 160) {
                return 'SEO description should be less than 160 characters';
              }
            },
          },
        },
        {
          name: 'keywords',
          type: 'string',
          label: 'SEO Keywords',
          list: true,
        },
      ],
    },
    {
      name: 'status',
      type: 'string',
      label: 'Status',
      options: [
        { value: 'draft', label: 'Draft' },
        { value: 'published', label: 'Published' },
        { value: 'archived', label: 'Archived' },
      ],
      required: true,
      ui: {
        defaultValue: 'draft',
      },
    },
    {
      name: 'featured',
      type: 'boolean',
      label: 'Featured Article',
      description: 'Show this article in featured sections',
    },
    {
      name: 'readTime',
      type: 'number',
      label: 'Estimated Read Time (minutes)',
      ui: {
        description: 'Leave empty to auto-calculate',
      },
    },
    {
      name: 'body',
      type: 'rich-text',
      label: 'Content',
      isBody: true,
      templates: [
        {
          name: 'StockChart',
          label: 'Stock Chart',
          ui: {
            defaultItem: {
              symbol: 'AAPL',
              period: '1M',
              height: 400,
            },
          },
          fields: [
            {
              name: 'symbol',
              type: 'string',
              label: 'Stock Symbol',
              required: true,
            },
            {
              name: 'period',
              type: 'string',
              label: 'Time Period',
              options: ['1D', '1W', '1M', '3M', '1Y', '5Y'],
            },
            {
              name: 'height',
              type: 'number',
              label: 'Chart Height',
            },
          ],
        },
        {
          name: 'EPSTable',
          label: 'EPS Comparison Table',
          ui: {
            defaultItem: {
              symbols: ['AAPL', 'GOOGL', 'MSFT'],
              quarters: 4,
            },
          },
          fields: [
            {
              name: 'symbols',
              type: 'string',
              label: 'Stock Symbols',
              list: true,
              required: true,
            },
            {
              name: 'quarters',
              type: 'number',
              label: 'Number of Quarters',
              ui: {
                parse: (val) => Number(val),
              },
            },
          ],
        },
        {
          name: 'InfoBox',
          label: 'Info Box',
          ui: {
            defaultItem: {
              type: 'info',
              title: 'Important Information',
            },
          },
          fields: [
            {
              name: 'type',
              type: 'string',
              label: 'Box Type',
              options: ['info', 'warning', 'success', 'error'],
            },
            {
              name: 'title',
              type: 'string',
              label: 'Title',
            },
            {
              name: 'content',
              type: 'rich-text',
              label: 'Content',
            },
          ],
        },
        {
          name: 'Quote',
          label: 'Quote Block',
          fields: [
            {
              name: 'text',
              type: 'string',
              label: 'Quote Text',
              ui: {
                component: 'textarea',
              },
            },
            {
              name: 'author',
              type: 'string',
              label: 'Quote Author',
            },
            {
              name: 'role',
              type: 'string',
              label: 'Author Role/Title',
            },
          ],
        },
      ],
    },
  ],
};