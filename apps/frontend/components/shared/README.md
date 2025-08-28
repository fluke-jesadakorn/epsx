# Analytics System Integration

## Overview
The My Data page integrates with the analytics system to provide users with personalized portfolio tracking capabilities. It leverages the same data structures, components, and API clients as the main analytics dashboard.

## Shared Components
- `AnalyticsClient`: Used for fetching analytics data
- `formatPercentage`: Utility for formatting growth percentages
- `AnalyticsNavigation`: Shared navigation between analytics pages
- Data types: `UnifiedRankingItem`, `QuarterlyData`, etc.

## Data Flow
1. User adds assets through the UI
2. Client component fetches analytics data using the same API as the main dashboard
3. Data is displayed using shared formatting utilities
4. UI follows the same design system and components

## Benefits
- Consistent user experience across analytics features
- Shared code reduces maintenance overhead
- Leveraging existing infrastructure for reliability
- Unified data models ensure accuracy