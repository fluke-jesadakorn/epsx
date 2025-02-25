import { PaginationParams } from "../interfaces/common.interfaces";

interface PaginationOptions {
  skip: number;
  take: number;
}

export const DEFAULT_PAGE_SIZE = 10;
export const MAX_PAGE_SIZE = 100;

export function getPaginationOptions(
  params: PaginationParams = {}
): PaginationOptions {
  const page = Math.max(1, params.page || 1);
  const take = Math.min(
    MAX_PAGE_SIZE,
    Math.max(1, params.limit || DEFAULT_PAGE_SIZE)
  );
  const skip = (page - 1) * take;

  return { skip, take };
}

// Future enhancement: Add support for cursor-based pagination for better performance with large datasets
// Future enhancement: Add support for different sorting options
// Future enhancement: Add support for filtering options
