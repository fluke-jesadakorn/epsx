import { NextResponse } from 'next/server';
import { getFirestoreAdmin } from '../../../../../lib/firebase-admin';
import type { TableDataMetrics } from '../../../../../types/stockFetchData';

// Firestore collection name for EPS growth ranking data
const COLLECTION_NAME = 'epsGrowthRankingData';

// Cache duration in seconds (5 minutes)
const CACHE_DURATION = 300;

export async function GET(request: Request) {
  try {
    // Initialize Firestore
    const db = getFirestoreAdmin();

    // Extract query parameters
    const { searchParams } = new URL(request.url);
    const limit = searchParams.get('limit') ? parseInt(searchParams.get('limit') as string) : undefined;
    const skip = searchParams.get('skip') ? parseInt(searchParams.get('skip') as string) : undefined;
    const sortBy = searchParams.get('sort_by') as 'growthIndicator' | 'activityScore' | undefined;

    // Create a cache key based on query parameters
    const cacheKey = `latest_${limit || 'default'}_${skip || 'default'}_${sortBy || 'default'}`;
    const cacheRef = db.collection(COLLECTION_NAME).doc(cacheKey);
    const cacheDoc = await cacheRef.get();

    const now = Date.now() / 1000; // Current time in seconds

    if (cacheDoc.exists) {
      const cachedData = cacheDoc.data();
      if (cachedData) {
        const cacheTimestamp = cachedData.timestamp || 0;

        // Check if cached data is still valid (within 5 minutes)
        if (now - cacheTimestamp < CACHE_DURATION) {
          return NextResponse.json({ data: cachedData.data }, {
            headers: {
              'Cache-Control': `public, max-age=${CACHE_DURATION}`,
            },
          });
        }
      }
    }

    // If no valid cache, fetch new data from TradingView
    const newData = await fetchEpsGrowthRankingFromTradingView({ limit, skip, sortBy });

    // Store the new data in Firestore with timestamp
    await cacheRef.set({
      data: newData,
      timestamp: now,
    });

    return NextResponse.json({ data: newData }, {
      headers: {
        'Cache-Control': `public, max-age=${CACHE_DURATION}`,
      },
    });
  } catch (error) {
    console.error('Error fetching EPS growth ranking data:', error);
    return NextResponse.json(
      { error: 'Failed to fetch EPS growth ranking data' },
      { status: 500 }
    );
  }
}

// Function to fetch EPS growth ranking data from TradingView
async function fetchEpsGrowthRankingFromTradingView(params: {
  limit?: number;
  skip?: number;
  sortBy?: 'growthIndicator' | 'activityScore';
}): Promise<TableDataMetrics[]> {
  // This is a placeholder for the actual implementation
  // In a real scenario, you would use a library like Puppeteer or a third-party API to scrape data from TradingView
  // For now, return mock data or an empty array
  console.log('Fetching EPS growth ranking data from TradingView with params:', params);
  
  // Example of how the data fetching could be implemented:
  // const response = await fetch('https://api.tradingview.com/...', {
  //   headers: {
  //     'Authorization': `Bearer ${process.env.TRADINGVIEW_AUTH_TOKEN}`,
  //   },
  //   body: JSON.stringify(params),
  // });
  // const data = await response.json();
  // return processTradingViewData(data);

  // Returning empty array as placeholder
  return [];
}
