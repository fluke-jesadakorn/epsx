// Define currency formatting function directly to avoid import issues
const fmtCurrency = (amount: number, currency: string = 'USD'): string => {
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency
  }).format(amount);
};

// Specialized financial formatting utilities
// Use shared utility with legacy naming
export const cur = (amt: number, currency = 'USD'): string => 
  fmtCurrency(amt, currency)

export const dt = (date: string | Date, fmt: 'short' | 'medium' | 'long' = 'medium'): string => {
  const d = typeof date === 'string' ? new Date(date) : date
  const fmts = {
    short: { month: 'short' as const, day: 'numeric' as const },
    medium: { month: 'short' as const, day: 'numeric' as const, year: 'numeric' as const },
    long: { month: 'long' as const, day: 'numeric' as const, year: 'numeric' as const }
  }
  return new Intl.DateTimeFormat('en-US', fmts[fmt]).format(d)
}

export const pct = (val: number, dec = 2): string => `${(val * 100).toFixed(dec)}%`

export const prc = (price: number | null): string => {
  if (price === null) return 'N/A'
  if (price >= 1000) return price.toLocaleString('en-US', { maximumFractionDigits: 2 })
  if (price >= 1) return price.toFixed(2)
  return price.toFixed(4)
}

export const epsGr = (growth: number | null | undefined): string => 
  growth === null || growth === undefined ? 'N/A' : `${growth > 0 ? '+' : ''}${growth}%`

export const dtFmt = (date: string): string => 
  new Date(date).toLocaleDateString('en-US', { year: 'numeric', month: 'short', day: 'numeric' })

// Backward compatibility
export const formatCurrency = cur
export const formatDate = dt
export const formatPercentage = pct
export const formatPrice = prc
export const formatEpsGrowth = epsGr
