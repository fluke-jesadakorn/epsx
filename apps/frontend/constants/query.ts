export enum SortDirection {
  ASC = "ASC",
  DESC = "DESC",
}

export enum Market {
  US = "US",
  TH = "TH",
}

export enum Metrics {
  EPS_DILUTED_QTR_GROWTH = "eps_diluted_qtr_growth",
  ANALYST_TARGET_UPSIDE = "analyst_target_upside",
  PE_LTM_LATEST = "pe_ltm_latest",
  EPS_BASIC_GROWTH = "eps_basic_growth",
  TOTAL_REV_QTR_GROWTH = "total_rev_qtr_growth",
  TOTAL_REV_GROWTH = "total_rev_growth",
  TOTAL_REV_CAGR_5Y = "total_rev_cagr_5y",
  MARKETCAP_ADJ_LATEST = "marketcap_adj_latest",
  INVESTING_EXCHANGE = "investing_exchange",
  INVESTING_SECTOR = "investing_sector",
  INVESTING_INDUSTRY = "investing_industry",
  PEG_LTM = "peg_ltm",
  ASSET_PRICE_LATEST = "asset_price_latest",
  ASSET_PRICE_LATEST_CHANGE_PCT = "asset_price_latest_change_pct",
  FAIR_VALUE = "fair_value",
  FAIR_VALUE_UPSIDE = "fair_value_upside",
  FAIR_VALUE_LABEL = "fair_value_label",
  ANALYST_TARGET = "analyst_target",
  FIN_HEALTH_OVERALL_LABEL = "fin_health_overall_label",
}

const MetricsDescription = {
  EPS_DILUTED_QTR_GROWTH: {
    label: Metrics.EPS_DILUTED_QTR_GROWTH,
    preset: "",
    description: "",
  },
  ANALYST_TARGET_UPSIDE: {
    label: Metrics.ANALYST_TARGET_UPSIDE,
    preset: "",
    description: "",
  },
  PE_LTM_LATEST: {
    label: Metrics.PE_LTM_LATEST,
    preset: "",
    description: "",
  },
  EPS_BASIC_GROWTH: {
    label: Metrics.EPS_BASIC_GROWTH,
    preset: "",
    description: "",
  },
  TOTAL_REV_QTR_GROWTH: {
    label: Metrics.TOTAL_REV_QTR_GROWTH,
    preset: "",
    description: "",
  },
  TOTAL_REV_GROWTH: {
    label: Metrics.TOTAL_REV_GROWTH,
    preset: "",
    description: "",
  },
  TOTAL_REV_CAGR_5Y: {
    label: Metrics.TOTAL_REV_CAGR_5Y,
    preset: "",
    description: "",
  },
  MARKETCAP_ADJ_LATEST: {
    label: Metrics.MARKETCAP_ADJ_LATEST,
    preset: "",
    description: "",
  },
  INVESTING_EXCHANGE: {
    label: Metrics.INVESTING_EXCHANGE,
    preset: "",
    description: "",
  },
  INVESTING_SECTOR: {
    label: Metrics.INVESTING_SECTOR,
    preset: "",
    description: "",
  },
  INVESTING_INDUSTRY: {
    label: Metrics.INVESTING_INDUSTRY,
    preset: "",
    description: "",
  },
  PEG_LTM: {
    label: Metrics.PEG_LTM,
    preset: "",
    description: "",
  },
  ASSET_PRICE_LATEST: {
    label: Metrics.ASSET_PRICE_LATEST,
    preset: "",
    description: "",
  },
  ASSET_PRICE_LATEST_CHANGE_PCT: {
    label: Metrics.ASSET_PRICE_LATEST_CHANGE_PCT,
    preset: "",
    description: "",
  },
  FAIR_VALUE: {
    label: Metrics.FAIR_VALUE,
    preset: "",
    description: "",
  },
  FAIR_VALUE_UPSIDE: {
    label: Metrics.FAIR_VALUE_UPSIDE,
    preset: "",
    description: "",
  },
  FAIR_VALUE_LABEL: {
    label: Metrics.FAIR_VALUE_LABEL,
    preset: "",
    description: "",
  },
  ANALYST_TARGET: {
    label: Metrics.ANALYST_TARGET,
    preset: "",
    description: "",
  },
  FIN_HEALTH_OVERALL_LABEL: {
    label: Metrics.FIN_HEALTH_OVERALL_LABEL,
    preset: "",
    description: "",
  },
};

export interface Query {
  metrics: Metrics;
  direction: SortDirection;
  query: {
    prefilters: { primaryOnly: boolean; market: Market };
    filters: { metric: Metrics; preset: string }[];
    sort: { metric: Metrics; direction: SortDirection };
  };
  page: { skip: number; limit: number };
}