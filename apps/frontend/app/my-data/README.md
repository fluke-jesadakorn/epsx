# My Data Page

## Overview
The My Data page (`/my-data`) allows users to track and analyze their portfolio assets using the same analytics system that powers the main analytics dashboard. It provides a personalized view of asset performance with growth factor analysis, quarterly performance comparisons, and market data.

## Features
- Portfolio position tracking with action recommendations (KEEP, TRACK, STOP)
- Real-time analytics data for your portfolio
- Visual performance indicators with gradient cards
- Action phase tracking with countdown timers
- Refresh functionality to update data

## Implementation Details

### Architecture
The page follows the same server-component-first pattern as the main analytics page:
- **Server Component** (`app/my-data/page.tsx`): Sets up the page with metadata
- **Client Component** (`components/my-data/MyDataClientWrapper.tsx`): Handles user interactions and UI state
- **Shared Components**:
  - `PositionCard.tsx`: Displays individual portfolio positions
  - `PortfolioHeader.tsx`: Shows portfolio metadata and controls
  - `mockPortfolioData.ts`: Provides sample data for demonstration

### Data Flow
1. Page loads with mock portfolio data (in a real implementation, this would come from a database)
2. User can toggle action statuses for each position (KEEP, TRACK, STOP)
3. User can refresh data to simulate fetching updated analytics
4. Portfolio positions are displayed in visually distinct gradient cards

### Integration with Analytics System
The My Data page integrates with the analytics system by:
- Using the same data structures and types as the main analytics dashboard
- Sharing UI components and design patterns
- Leveraging the same API client for data fetching (when implemented)
- Following the same authentication and authorization patterns

## Components

### MyDataClientWrapper
Main client component that manages state and renders the portfolio view.

### PositionCard
Displays an individual portfolio position with:
- Symbol and ranking
- Action status toggle (KEEP/TRACK/STOP)
- Action phase timeline with countdown
- Performance indicator
- Quarterly data comparison
- Next announcement date

### PortfolioHeader
Shows portfolio metadata including:
- Processing time
- Refresh button
- Filter and settings controls

## Future Enhancements
- User authentication and persistent portfolio storage
- Real-time data updates with WebSocket integration
- Export functionality for portfolio data
- Advanced filtering and sorting options
- Performance benchmarks against market indices
- Integration with actual analytics API endpoints