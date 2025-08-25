import type { Collection } from 'tinacms';

export const categorySchema: Collection = {
  name: 'categories',
  label: 'Categories',
  path: 'content',
  format: 'json',
  ui: {
    allowedActions: {
      create: true,
      delete: true,
    },
  },
  fields: [
    {
      name: 'categories',
      type: 'object',
      label: 'Categories',
      list: true,
      ui: {
        itemProps: (item) => ({
          label: item?.name || 'New Category',
        }),
      },
      fields: [
        {
          name: 'id',
          type: 'string',
          label: 'Category ID',
          required: true,
          ui: {
            validate: (value) => {
              if (!value?.match(/^[a-z0-9-]+$/)) {
                return 'ID must contain only lowercase letters, numbers, and hyphens';
              }
            },
          },
        },
        {
          name: 'name',
          type: 'string',
          label: 'Category Name',
          required: true,
        },
        {
          name: 'slug',
          type: 'string',
          label: 'URL Slug',
          required: true,
          ui: {
            validate: (value) => {
              if (!value?.match(/^[a-z0-9-]+$/)) {
                return 'Slug must contain only lowercase letters, numbers, and hyphens';
              }
            },
          },
        },
        {
          name: 'description',
          type: 'string',
          label: 'Description',
          ui: {
            component: 'textarea',
          },
        },
        {
          name: 'color',
          type: 'string',
          label: 'Category Color',
          ui: {
            component: 'color',
            colorFormat: 'hex',
          },
        },
        {
          name: 'icon',
          type: 'string',
          label: 'Icon Name',
          description: 'Lucide icon name (e.g., trending-up, bar-chart)',
        },
        {
          name: 'sortOrder',
          type: 'number',
          label: 'Sort Order',
          description: 'Lower numbers appear first',
        },
        {
          name: 'featured',
          type: 'boolean',
          label: 'Featured Category',
          description: 'Show in featured sections',
        },
        {
          name: 'active',
          type: 'boolean',
          label: 'Active Category',
          ui: {
            defaultValue: true,
          },
        },
      ],
    },
  ],
};