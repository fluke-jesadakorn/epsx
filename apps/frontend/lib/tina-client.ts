// Stub implementation for TinaCMS client (packages removed to fix axios conflicts)

// Mock database implementation
export const database = {
  queries: {
    news: async ({ relativePath }: { relativePath: string }) => {
      return {
        data: {
          news: null // Stub - news functionality disabled
        }
      };
    },
    newsConnection: async ({ first, filter, sort }: any) => {
      return {
        data: {
          newsConnection: {
            edges: [] // Stub - no articles available
          }
        }
      };
    }
  }
};

// Mock TinaCMS React hooks
export const tinaField = (field: any) => ({});
export const useTina = (props: any) => ({ data: props.data });

// Stub types for news functionality
export interface NewsQuery {
  data: {
    news: any;
  };
}

export interface NewsConnectionQuery {
  data: {
    newsConnection: {
      edges: any[];
    };
  };
}