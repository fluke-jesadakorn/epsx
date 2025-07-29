import { z } from 'zod';

export const StockItemSchema = z.object({
  symbol: z.string(),
  name: z.string(),
  price: z.number(),
  change: z.number(),
  changePercent: z.number(),
  volume: z.number().nonnegative(),
  marketCap: z.number().nonnegative(),
  eps: z.number().optional(),
  peRatio: z.number().optional(),
  sector: z.string().optional(),
  industry: z.string().optional(),
  lastUpdated: z.date(),
});

export const StockRankingSchema = z.object({
  rank: z.number().positive(),
  symbol: z.string(),
  name: z.string(),
  score: z.number(),
  metrics: z.record(z.number()),
  change: z.number(),
  changePercent: z.number(),
  category: z.string(),
});

export const StockFinancialDataSchema = z.object({
  symbol: z.string(),
  revenue: z.number(),
  earnings: z.number(),
  eps: z.number(),
  pe: z.number(),
  pb: z.number(),
  roe: z.number(),
  debtToEquity: z.number(),
  currentRatio: z.number(),
  quickRatio: z.number(),
  profitMargin: z.number(),
  operatingMargin: z.number(),
  asOfDate: z.date(),
});

export const PortfolioItemSchema = z.object({
  id: z.string().uuid(),
  userId: z.string().uuid(),
  symbol: z.string(),
  shares: z.number().positive(),
  avgPrice: z.number().positive(),
  currentPrice: z.number().nonnegative(),
  totalValue: z.number(),
  gainLoss: z.number(),
  gainLossPercent: z.number(),
  addedAt: z.date(),
  updatedAt: z.date(),
});

export const WatchlistAddRequestSchema = z.object({
  symbol: z.string(),
  notes: z.string().optional(),
});

export const PriceAlertSchema = z.object({
  id: z.string().uuid(),
  userId: z.string().uuid(),
  symbol: z.string(),
  targetPrice: z.number().positive(),
  condition: z.enum(['above', 'below']),
  isActive: z.boolean(),
  isTriggered: z.boolean(),
  createdAt: z.date(),
  triggeredAt: z.date().optional(),
});

export const PriceAlertCreateRequestSchema = z.object({
  symbol: z.string(),
  targetPrice: z.number().positive(),
  condition: z.enum(['above', 'below']),
});