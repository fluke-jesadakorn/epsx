import { NextRequest, NextResponse } from 'next/server';
import { getStockFinancialData } from '@/lib/services/stock.service';
import { validateRankingAccess } from '@/middleware/rankingAccess';
import { cookies } from 'next/headers';

export async function GET(request: NextRequest) {
  try {
    const cookieStore = await cookies();
    const userLevel = (cookieStore.get('userLevel')?.value || 'BASIC') as any;
    const isExpired = cookieStore.get('isExpired')?.value === 'true';
    
    // Validate and potentially modify the request
    const accessValidation = validateRankingAccess(request, userLevel, isExpired);
    
    const { searchParams } = new URL(accessValidation.modifiedUrl || request.url);
    const limit = parseInt(searchParams.get('limit') || '10');
    const skip = parseInt(searchParams.get('skip') || '0');
    
    // Get stock data and extract symbols
    const stockData = await getStockFinancialData(skip, limit);
    
    // Extract just the symbols
    const symbols = stockData.map(item => item.symbol);
    
    return NextResponse.json({ 
      symbols, 
      count: symbols.length,
      userAccess: {
        level: accessValidation.userLevel,
        maxAllowed: accessValidation.maxAllowed,
        wasLimited: accessValidation.wasLimited,
        upgradeAvailable: accessValidation.userLevel === 'BRONZE' || isExpired
      }
    });
  } catch (error) {
    console.error('Error fetching symbols:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}
