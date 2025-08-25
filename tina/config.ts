import { defineConfig } from 'tinacms';
import { newsSchema } from './schema/news';
import { authorSchema } from './schema/author';
import { categorySchema } from './schema/category';

export default defineConfig({
  branch: process.env.TINA_BRANCH || 'main',
  clientId: process.env.TINA_CLIENT_ID!,
  token: process.env.TINA_TOKEN!,
  
  build: {
    outputFolder: "admin",
    publicFolder: "apps/frontend/public",
    host: process.env.NODE_ENV === 'development' ? 'localhost' : undefined,
  },
  
  media: {
    tina: {
      mediaRoot: "content/media",
      publicFolder: "apps/frontend/public",
      static: true,
    },
  },
  
  search: {
    tina: {
      indexerToken: process.env.TINA_SEARCH_TOKEN,
      stopwordLanguages: ['eng'],
    },
  },
  
  schema: {
    collections: [
      newsSchema,
      authorSchema,
      categorySchema,
    ],
  },
  
  admin: {
    auth: {
      customAuthProvider: () => {
        return {
          authenticate: async () => {
            // Integration with existing auth system will be added later
            const response = await fetch('/api/auth/session');
            if (response.ok) {
              const session = await response.json();
              return {
                id: session.user.id,
                name: session.user.name,
                email: session.user.email,
              };
            }
            throw new Error('Not authenticated');
          },
          getUser: async () => {
            const response = await fetch('/api/auth/session');
            if (response.ok) {
              const session = await response.json();
              return session.user;
            }
            return null;
          },
        };
      },
    },
  },
});