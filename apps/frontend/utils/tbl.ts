// Table utilities
export interface Column {
  label: string
  metric: string
}

export interface Row {
  asset: {
    ticker: string
  }
  data: Array<{
    value: string | number
  }>
}

export interface TableApiResponse {
  columns: Column[]
  rows: Row[]
}

// Create table columns from API response
export const cols = (res: TableApiResponse) =>
  res.columns.map((col, idx) => ({
    title: col.label,
    dataIndex: col.metric,
    key: col.metric,
    ...(idx < 2 && {
      fixed: 'left' as const,
      width: 100
    })
  }))

// Apply access control masking
export const mask = (val: string | number, lvl: number): string | number => {
  const n = Number(val)
  if (lvl === 1 && n < 21) {return 'Subscribe for access'}
  if (lvl === 2 && n < 11) {return 'Premium content'}
  return val || '-'
}

// Create table data with access control
export const rows = (res: TableApiResponse, lvl = 1) =>
  res.rows.map((row, idx) => {
    const data: Record<string, string | number> = { key: idx, ticker: row.asset.ticker }
    row.data.forEach((item, i) => {
      const metric = res.columns[i]?.metric
      if (metric) {
        data[metric] = metric === 'stock_number' ? mask(item.value, lvl) : (item.value || '-')
      }
    })
    return data
  })

// Backward compatibility
export const createTableColumns = cols
export const createTableData = rows
export const applyStockNumberAccessControl = mask
