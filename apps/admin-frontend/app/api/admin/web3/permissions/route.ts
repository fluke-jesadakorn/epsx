import { NextRequest, NextResponse } from 'next/server';
import { cookies } from 'next/headers';

// GET /api/admin/web3/permissions - Get wallet permissions with filtering
export async function GET(request: NextRequest) {
  try {
    // Get search params for filtering
    const searchParams = request.nextUrl.searchParams;
    const walletAddress = searchParams.get('wallet_address');
    const permission = searchParams.get('permission');
    const source = searchParams.get('source');
    const limitParam = searchParams.get('limit');
    const offsetParam = searchParams.get('offset');

    // Verify admin session - simple wallet_address check
    const cookieStore = await cookies();
    const sessionWalletAddress = cookieStore.get('wallet_address')?.value;
    
    if (!sessionWalletAddress) {
      return NextResponse.json(
        { error: 'No admin session found' },
        { status: 401 }
      );
    }

    console.log('🔍 Admin: Fetching Web3 permissions with filters:', {
      walletAddress,
      permission,
      source,
      limit: limitParam,
      offset: offsetParam
    });

    // Query database directly for better reliability
    const { Pool } = require('pg');
    const databaseUrl = process.env.DATABASE_URL || 'postgresql://postgres:password@localhost:5432/epsx_db';
    const pool = new Pool({
      connectionString: databaseUrl,
      max: 5,
      idleTimeoutMillis: 30000,
      connectionTimeoutMillis: 2000,
      native: false
    });

    let permissions: any[] = [];
    
    try {
      const client = await pool.connect();
      try {
        // Build WHERE conditions
        const conditions: string[] = [];
        const params: any[] = [];
        let paramIndex = 1;

        if (walletAddress) {
          conditions.push(`wallet_address = $${paramIndex}`);
          params.push(walletAddress.toLowerCase());
          paramIndex++;
        }

        if (permission) {
          conditions.push(`permission ILIKE $${paramIndex}`);
          params.push(`%${permission}%`);
          paramIndex++;
        }

        if (source) {
          conditions.push(`source = $${paramIndex}`);
          params.push(source);
          paramIndex++;
        }

        const whereClause = conditions.length > 0 ? `WHERE ${conditions.join(' AND ')}` : '';
        
        // Parse pagination
        const limit = limitParam ? parseInt(limitParam) : 50;
        const offset = offsetParam ? parseInt(offsetParam) : 0;

        // Build query
        const query = `
          SELECT 
            id,
            wallet_address,
            permission,
            source,
            expires_at,
            granted_at,
            granted_by,
            metadata,
            is_active
          FROM wallet_permissions 
          ${whereClause}
          ORDER BY granted_at DESC
          LIMIT $${paramIndex} OFFSET $${paramIndex + 1}
        `;
        
        params.push(limit, offset);

        const result = await client.query(query, params);
        permissions = result.rows;

        // Get total count for pagination
        const countQuery = `
          SELECT COUNT(*) as total
          FROM wallet_permissions 
          ${whereClause}
        `;
        
        const countResult = await client.query(countQuery, params.slice(0, -2)); // Remove limit/offset params
        const totalCount = parseInt(countResult.rows[0].total);

        console.log('✅ Admin: Retrieved permissions from database:', {
          count: permissions.length,
          total: totalCount,
          sessionWallet: sessionWalletAddress
        });

        return NextResponse.json({
          permissions: permissions,
          total_count: totalCount,
          limit: limit,
          offset: offset
        });

      } finally {
        client.release();
      }
    } catch (dbError) {
      console.error('❌ Admin: Database query failed:', dbError);
      return NextResponse.json(
        { error: 'Failed to fetch permissions from database' },
        { status: 500 }
      );
    } finally {
      await pool.end();
    }

  } catch (error) {
    console.error('❌ Admin: Web3 permissions GET error:', error);
    return NextResponse.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}

// POST /api/admin/web3/permissions - Not used (permissions are granted via specific endpoints)
export async function POST(request: NextRequest) {
  return NextResponse.json(
    { error: 'Use specific permission grant endpoints instead' },
    { status: 405 }
  );
}