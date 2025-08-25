import type { Collection } from 'tinacms';

export const authorSchema: Collection = {
  name: 'authors',
  label: 'Authors',
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
      name: 'authors',
      type: 'object',
      label: 'Authors',
      list: true,
      ui: {
        itemProps: (item) => ({
          label: item?.name || 'New Author',
        }),
      },
      fields: [
        {
          name: 'id',
          type: 'string',
          label: 'Author ID',
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
          label: 'Full Name',
          required: true,
        },
        {
          name: 'email',
          type: 'string',
          label: 'Email',
          required: true,
          ui: {
            validate: (value) => {
              if (value && !value.includes('@')) {
                return 'Please enter a valid email address';
              }
            },
          },
        },
        {
          name: 'avatar',
          type: 'image',
          label: 'Avatar Image',
        },
        {
          name: 'bio',
          type: 'string',
          label: 'Biography',
          ui: {
            component: 'textarea',
          },
        },
        {
          name: 'role',
          type: 'string',
          label: 'Role',
          options: [
            { value: 'admin', label: 'Administrator' },
            { value: 'editor', label: 'Editor' },
            { value: 'analyst', label: 'Market Analyst' },
            { value: 'writer', label: 'Content Writer' },
          ],
          required: true,
        },
        {
          name: 'socialLinks',
          type: 'object',
          label: 'Social Links',
          fields: [
            {
              name: 'twitter',
              type: 'string',
              label: 'Twitter URL',
            },
            {
              name: 'linkedin',
              type: 'string',
              label: 'LinkedIn URL',
            },
            {
              name: 'website',
              type: 'string',
              label: 'Personal Website',
            },
          ],
        },
        {
          name: 'active',
          type: 'boolean',
          label: 'Active Author',
          ui: {
            defaultValue: true,
          },
        },
      ],
    },
  ],
};