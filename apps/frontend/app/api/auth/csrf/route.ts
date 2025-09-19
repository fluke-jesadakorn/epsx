import { NextRequest } from 'next/server';
import { getCSRFTokenAPI } from '@/lib/csrf';

export async function GET(request: NextRequest) {
  return getCSRFTokenAPI(request);
}