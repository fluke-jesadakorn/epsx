import { z } from 'zod';

export const ApiResponseSchema = <T extends z.ZodTypeAny>(dataSchema: T): z.ZodObject<{
  data: z.ZodOptional<T>;
  error: z.ZodOptional<z.ZodString>;
  details: z.ZodOptional<z.ZodString>;
  message: z.ZodOptional<z.ZodString>;
}> =>
  z.object({
    data: dataSchema.optional(),
    error: z.string().optional(),
    details: z.string().optional(),
    message: z.string().optional(),
  });

export const ApiErrorSchema = z.object({
  error: z.string(),
  details: z.string().optional(),
  status: z.number().optional(),
});

export const PaginationSchema = z.object({
  page: z.number().positive(),
  limit: z.number().positive(),
  total: z.number().nonnegative(),
  totalPages: z.number().nonnegative(),
  hasNext: z.boolean(),
  hasPrev: z.boolean(),
});

export const PaginatedResponseSchema = <T extends z.ZodTypeAny>(itemSchema: T): z.ZodObject<{
  data: z.ZodArray<T>;
  pagination: typeof PaginationSchema;
}> =>
  z.object({
    data: z.array(itemSchema),
    pagination: PaginationSchema,
  });

export const CountResponseSchema = z.object({
  count: z.number().nonnegative(),
});

export const SortOptionsSchema = z.object({
  field: z.string(),
  direction: z.enum(['asc', 'desc']),
});

export const FilterOptionsSchema = z.object({
  field: z.string(),
  operator: z.enum(['eq', 'ne', 'gt', 'gte', 'lt', 'lte', 'in', 'like']),
  value: z.unknown(),
});

export const SearchOptionsSchema = z.object({
  query: z.string(),
  fields: z.array(z.string()).optional(),
  exact: z.boolean().optional(),
});

export const ListOptionsSchema = z.object({
  pagination: PaginationSchema.partial().optional(),
  sort: z.array(SortOptionsSchema).optional(),
  filters: z.array(FilterOptionsSchema).optional(),
  search: SearchOptionsSchema.optional(),
});