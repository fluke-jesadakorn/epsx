// Common utility functions
export { BaseHttpService, HttpServiceError } from "./http.service";
export type { FetchConfig } from "./http.service";

export const isProduction = process.env.NODE_ENV === "production";

export const sleep = (ms: number) =>
  new Promise((resolve) => setTimeout(resolve, ms));

export const formatError = (error: any): string => {
  if (typeof error === "string") return error;
  if (error instanceof Error) return error.message;
  if (typeof error === "object" && error !== null) {
    return error.message || JSON.stringify(error);
  }
  return "An unknown error occurred";
};

// Add more utility functions as needed
