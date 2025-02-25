export interface Response {
  columns: Column[];
  rows: Row[];
  page: Page;
}

type Column = {
  metric: string;
  label: string;
  description: string;
  format: string;
  sortable: boolean;
};

type Row = {
  asset: Asset;
  data: RowData[];
};

export type Query = {
  query: {
    filters: {
      metric: string;
      preset: string;
    }[];
    sort: {
      metric: string;
      direction: string;
    };
    prefilters: {
      primaryOnly: boolean;
      market: string;
    };
  };
  metrics: string[];
  page: {
    skip: number;
    limit: number;
  };
};

type RowData = {
  hidden: boolean;
  value: string;
  raw: string | number | null;
  sentiment: number;
};

type Asset = {
  hidden: boolean;
  uid: string;
  pairID: number;
  ticker: string;
  name: string;
  primary: string;
  path: string;
  logo: string;
  logoDark: string;
  sector: string;
};

type Page = {
  hasPreviousPage: boolean;
  hasNextPage: boolean;
  pageSize: number;
  totalItems: number;
};

type Sort = {
  metric: string;
  direction: "ASC" | "DESC";
};

type Data = {
  page: Page;
  sort: Sort;
  columns: Column[];
  rows: Row[];
  ssid: string;
  allowCustom: boolean;
  currency: string;
};
