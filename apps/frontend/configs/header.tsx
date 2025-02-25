// Header configuration for API requests
export const API_HEADERS = {
  'Content-Type': 'application/json',
  'Cache-Control': 'no-cache',
  'Accept': 'application/json'
};

// Auth headers for Supabase
export const AUTH_HEADERS = (token: string) => ({
  'Authorization': `Bearer ${token}`,
  ...API_HEADERS
});

// CORS headers for API responses
export const CORS_HEADERS = {
  'Access-Control-Allow-Origin': '*',
  'Access-Control-Allow-Methods': 'GET, POST, PUT, DELETE, OPTIONS',
  'Access-Control-Allow-Headers': 'Content-Type, Authorization'
};

// TODO: Add more specific headers for different endpoints
// TODO: Implement header versioning for API changes
// TODO: Add rate limiting headers
