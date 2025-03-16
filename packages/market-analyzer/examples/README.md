# Market Analyzer Examples

This directory contains example code demonstrating various features of the market analyzer package.

## Examples

### Price Data (price-example.ts)
Shows how to fetch current price data for multiple stocks. This example demonstrates:
- Using the `getPrice()` function
- Fetching data for multiple symbols
- Error handling

### Financial Data (financials-example.ts)
Demonstrates how to retrieve financial data for stocks. This example shows:
- Using the `getFinancials()` function
- Accessing fundamental data
- Error handling

### WebSocket Updates (websocket-example.ts)
Shows how to use WebSocket connections for real-time market data. This example demonstrates:
- Initializing WebSocket connections
- Subscribing to real-time price updates for multiple symbols
- Getting real-time financial data
- Proper connection cleanup

## Running the Examples

Each example can be run independently using the TypeScript compiler:

```bash
# Compile and run price example
npx ts-node price-example.ts

# Compile and run financials example
npx ts-node financials-example.ts

# Compile and run websocket example
npx ts-node websocket-example.ts
```

Note: Make sure you have the required dependencies installed before running the examples.
