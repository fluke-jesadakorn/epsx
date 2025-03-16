/**
 * Delay execution for specified milliseconds
 */
export async function delay(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

/**
 * Validate and format TradingView symbol
 * @param symbol Full symbol in EXCHANGE:SYMBOL format (e.g., 'NASDAQ:AAPL', 'SET:PTT')
 * @throws Error if symbol format is invalid
 */
export function formatSymbol(symbol: string): string {
  if (!symbol.includes(':')) {
    throw new Error('Symbol must be in EXCHANGE:SYMBOL format (e.g., NASDAQ:AAPL)');
  }
  return symbol.toUpperCase();
}

/**
 * Convert date string to Unix timestamp (midnight UTC)
 */
export function dateToTimestamp(date: string): number {
  return Math.floor(new Date(date).getTime() / 1000);
}

/**
 * Generate a random session ID
 */
export function generateSessionId(): string {
  return Math.random().toString(36).substring(2, 15);
}

/**
 * Parse WebSocket messages
 */
export function parseWebSocketMessage(message: string): Array<{m: string; p: any[]}> {
  const messages = [];
  const parts = message.split(/(~m~\d+~m~)/).filter(Boolean);

  for (let i = 0; i < parts.length; i++) {
    const frameMatch = parts[i].match(/~m~(\d+)~m~/);
    if (!frameMatch) continue;

    const payload = parts[++i];
    if (!payload || payload.length !== parseInt(frameMatch[1], 10)) continue;

    if (payload.includes('~h~')) {
      messages.push({ m: 'heartbeat', p: [payload.split('~h~')[1]] });
      continue;
    }

    try {
      const parsed = JSON.parse(payload);
      if (typeof parsed === 'object' && parsed !== null) {
        messages.push(parsed);
      }
    } catch (e) {
      console.error('Failed to parse message payload:', payload);
    }
  }
  
  return messages;
}

/**
 * Format WebSocket message for sending
 */
export function formatWebSocketMessage(func: string, args: any[]): string {
  const jsonStr = JSON.stringify({ m: func, p: args });
  const msgLen = Buffer.from(jsonStr).length;
  return `~m~${msgLen}~m~${jsonStr}`;
}

/**
 * Handle exponential backoff for retries
 */
export function calculateBackoff(attempt: number, baseDelay: number): number {
  return baseDelay * Math.pow(2, attempt);
}

/**
 * Clean and validate market data response
 */
export function validateMarketData<T>(data: T | null | undefined): T | null {
  if (!data) return null;
  if (typeof data !== 'object') return null;
  return data;
}

/**
 * Format error message from various error types
 */
export function formatErrorMessage(error: unknown): string {
  if (error instanceof Error) return error.message;
  if (typeof error === 'string') return error;
  return 'Unknown error occurred';
}
