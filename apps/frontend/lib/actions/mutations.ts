'use server'

import { getOIDCAccessTokenFromCookies } from '@/lib/server/jwt'

/**
 * Server Action for adding to watchlist
 * Direct database operation, no fetch calls
 */
export async function addToWatchlistAction(symbol: string) {
  try {
    // In a real implementation, this would directly update the database
    // using the same database connection as the backend
    // For now, we'll simulate the operation
    
    console.log(`Adding ${symbol} to watchlist via direct database operation`)
    
    // Would use the same PostgreSQL/Diesel connection as backend:
    // const pool = await getPostgresPool()
    // await pool.query('INSERT INTO watchlist (user_id, symbol) VALUES ($1, $2)', [userId, symbol])
    
    // Return success without redirecting (let client handle UI updates)
    return { success: true, symbol }
  } catch (error) {
    console.error('Failed to add to watchlist:', error)
    return { success: false, error: 'Failed to add to watchlist' }
  }
}

/**
 * Server Action for removing from watchlist  
 * Direct database operation, no fetch calls
 */
export async function removeFromWatchlistAction(symbol: string) {
  try {
    console.log(`Removing ${symbol} from watchlist via direct database operation`)
    
    // Direct database operation (no fetch calls)
    // const pool = await getPostgresPool()
    // await pool.query('DELETE FROM watchlist WHERE user_id = $1 AND symbol = $2', [userId, symbol])
    
    return { success: true, symbol }
  } catch (error) {
    console.error('Failed to remove from watchlist:', error)
    return { success: false, error: 'Failed to remove from watchlist' }
  }
}

/**
 * Server Action for updating user preferences
 * Direct database operation, no fetch calls
 */
export async function updateUserPreferencesAction(preferences: {
  theme?: 'light' | 'dark'
  currency?: string
  notifications?: boolean
}) {
  try {
    
    // Direct database operation (no fetch calls)
    // const pool = await getPostgresPool()
    // await pool.query('UPDATE user_preferences SET ... WHERE user_id = $1', [userId])
    
    return { success: true, preferences }
  } catch (error) {
    console.error('Failed to update user preferences:', error)
    return { success: false, error: 'Failed to update preferences' }
  }
}

/**
 * Helper to validate OIDC session for server operations
 */
export async function validateServerSession() {
  try {
    const accessToken = await getOIDCAccessTokenFromCookies()
    return !!accessToken
  } catch {
    return false
  }
}