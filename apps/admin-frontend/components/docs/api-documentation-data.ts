import { ModuleDocumentation } from './api-documentation-types';

export const moduleDocumentation: ModuleDocumentation[] = [
    {
        name: 'stock-ranking',
        displayName: 'Stock Ranking',
        description: 'Advanced stock ranking and analysis tools with AI-powered insights',
        category: 'Analytics',
        endpoints: [
            {
                method: 'GET',
                path: '/api/stock-ranking/rankings',
                description: 'Get basic stock rankings with filtering options',
                accessLevel: 'Bronze+',
                parameters: [
                    { name: 'sector', type: 'string', required: false, description: 'Filter by sector (e.g., "technology")' },
                    { name: 'limit', type: 'integer', required: false, description: 'Number of results (max 100)' },
                    { name: 'sort', type: 'string', required: false, description: 'Sort by: rank, performance, volume' },
                ],
                response: `{
  "data": [
    {
      "symbol": "AAPL",
      "rank": 1,
      "score": 95.2,
      "sector": "Technology",
      "performance": {
        "1d": 0.025,
        "1w": -0.012,
        "1m": 0.087
      }
    }
  ],
  "total": 50,
  "quota_remaining": 95
}`,
            },
            {
                method: 'GET',
                path: '/api/stock-ranking/rankings/ai-insights',
                description: 'Get AI-powered insights and predictions',
                accessLevel: 'Silver+',
                parameters: [
                    { name: 'symbol', type: 'string', required: true, description: 'Stock symbol (e.g., "AAPL")' },
                    { name: 'horizon', type: 'string', required: false, description: 'Prediction horizon: 1d, 1w, 1m' },
                ],
                response: `{
  "symbol": "AAPL",
  "insights": {
    "sentiment": "bullish",
    "confidence": 0.87,
    "price_target": 185.50,
    "key_factors": [
      "Strong earnings growth",
      "Positive analyst sentiment"
    ]
  },
  "quota_remaining": 18
}`,
            },
            {
                method: 'POST',
                path: '/api/stock-ranking/rankings/custom',
                description: 'Create custom ranking algorithm',
                accessLevel: 'Gold+',
                parameters: [
                    { name: 'algorithm', type: 'object', required: true, description: 'Custom algorithm configuration' },
                    { name: 'name', type: 'string', required: true, description: 'Algorithm name' },
                ],
                response: `{
  "algorithm_id": "custom_123",
  "name": "My Custom Algorithm",
  "status": "created",
  "estimated_runtime": "2-5 minutes"
}`,
            },
        ],
    },
    {
        name: 'market-data',
        displayName: 'Market Data',
        description: 'Real-time and historical market data with technical indicators',
        category: 'Data',
        endpoints: [
            {
                method: 'GET',
                path: '/api/market-data/quotes/{symbol}',
                description: 'Get current quote for a symbol (15-minute delay for Bronze)',
                accessLevel: 'Bronze+',
                parameters: [{ name: 'symbol', type: 'string', required: true, description: 'Stock symbol in path (e.g., "AAPL")' }],
                response: `{
  "symbol": "AAPL",
  "price": 182.45,
  "change": 2.15,
  "change_percent": 1.19,
  "volume": 45678901,
  "timestamp": "2024-01-15T15:30:00Z",
  "delay_minutes": 15
}`,
            },
            {
                method: 'GET',
                path: '/api/market-data/quotes/{symbol}/live',
                description: 'Get real-time quote (no delay)',
                accessLevel: 'Silver+',
                parameters: [{ name: 'symbol', type: 'string', required: true, description: 'Stock symbol in path' }],
                response: `{
  "symbol": "AAPL",
  "price": 182.47,
  "bid": 182.46,
  "ask": 182.48,
  "volume": 45678901,
  "timestamp": "2024-01-15T15:30:05Z",
  "real_time": true
}`,
            },
            {
                method: 'GET',
                path: '/api/market-data/indicators/{symbol}/rsi',
                description: 'Get Relative Strength Index (RSI) indicator',
                accessLevel: 'Silver+',
                parameters: [
                    { name: 'symbol', type: 'string', required: true, description: 'Stock symbol in path' },
                    { name: 'period', type: 'integer', required: false, description: 'RSI period (default: 14)' },
                ],
                response: `{
  "symbol": "AAPL",
  "indicator": "RSI",
  "value": 67.2,
  "signal": "neutral",
  "period": 14,
  "timestamp": "2024-01-15T15:30:00Z"
}`,
            },
        ],
    },
    {
        name: 'portfolio-analysis',
        displayName: 'Portfolio Analysis',
        description: 'Comprehensive portfolio management and risk analysis tools',
        category: 'Analytics',
        endpoints: [
            {
                method: 'POST',
                path: '/api/portfolio-analysis/portfolios',
                description: 'Create a new portfolio for analysis',
                accessLevel: 'Bronze+',
                parameters: [
                    { name: 'name', type: 'string', required: true, description: 'Portfolio name' },
                    { name: 'holdings', type: 'array', required: true, description: 'Array of stock holdings' },
                ],
                response: `{
  "portfolio_id": "port_123",
  "name": "My Portfolio",
  "total_value": 125000.00,
  "holdings_count": 8,
  "created_at": "2024-01-15T15:30:00Z"
}`,
            },
            {
                method: 'GET',
                path: '/api/portfolio-analysis/portfolios/{id}/risk',
                description: 'Get comprehensive risk analysis',
                accessLevel: 'Silver+',
                parameters: [
                    { name: 'id', type: 'string', required: true, description: 'Portfolio ID in path' },
                    { name: 'timeframe', type: 'string', required: false, description: 'Analysis timeframe: 1m, 3m, 6m, 1y' },
                ],
                response: `{
  "portfolio_id": "port_123",
  "risk_metrics": {
    "beta": 1.12,
    "sharpe_ratio": 1.47,
    "max_drawdown": -0.085,
    "var_95": -0.032
  },
  "risk_level": "moderate"
}`,
            },
        ],
    },
    {
        name: 'market-signals',
        displayName: 'Market Signals',
        description: 'AI-powered market signals and strategy management',
        category: 'Analytics',
        endpoints: [
            {
                method: 'GET',
                path: '/api/market-signals/signals',
                description: 'Get current market signals',
                accessLevel: 'Silver+',
                parameters: [
                    {
                        name: 'symbols',
                        type: 'string',
                        required: false,
                        description: 'Comma-separated symbols (e.g., "AAPL,MSFT")',
                    },
                    { name: 'signal_type', type: 'string', required: false, description: 'Filter by: buy, sell, hold' },
                ],
                response: `{
  "signals": [
    {
      "symbol": "AAPL",
      "signal": "buy",
      "confidence": 0.87,
      "price_target": 185.50,
      "stop_loss": 175.00,
      "generated_at": "2024-01-15T15:30:00Z"
    }
  ],
  "quota_remaining": 15
}`,
            },
            {
                method: 'POST',
                path: '/api/market-signals/strategies/{id}/backtest',
                description: 'Run backtesting on a market strategy',
                accessLevel: 'Gold+',
                parameters: [
                    { name: 'id', type: 'string', required: true, description: 'Strategy ID in path' },
                    { name: 'start_date', type: 'string', required: true, description: 'Backtest start date (YYYY-MM-DD)' },
                    { name: 'end_date', type: 'string', required: true, description: 'Backtest end date (YYYY-MM-DD)' },
                ],
                response: `{
  "backtest_id": "bt_456",
  "strategy_id": "strat_123",
  "results": {
    "total_return": 0.247,
    "sharpe_ratio": 1.83,
    "max_drawdown": -0.065,
    "win_rate": 0.68
  },
  "status": "completed"
}`,
            },
        ],
    },
];
