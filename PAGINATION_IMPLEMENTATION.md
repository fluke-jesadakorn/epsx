# Pagination Implementation

This document describes the pagination feature implementation for the EPSX stock ranking system.

## Features Implemented

### 1. Pagination UI Component (`/components/ui/pagination.tsx`)
- **Smart page navigation** with ellipsis for large page counts
- **Previous/Next buttons** with proper disabled states
- **Clickable page numbers** with current page highlighting
- **Loading states** to prevent multiple rapid clicks
- **Responsive design** that works on mobile and desktop

### 2. Pagination Hook (`/hooks/usePagination.ts`)
- **State management** for current page, items per page, and loading state
- **Callback support** for page and limit changes
- **Reset functionality** to return to initial state
- **Reusable** across different components

### 3. Paginated Feature Access (`/hooks/usePaginatedFeatureAccess.ts`)
- **Role-based access control** integrated with existing payment system
- **Tier-based limits**: Basic (5), Silver (25), Gold (50), Platinum (100)
- **Page access validation** based on user subscription level
- **Dynamic page size options** based on user tier

### 4. Paginated Stock Data Service (`/app/actions/stockRankingPaginated.ts`)
- **Server-side pagination** with proper skip/limit calculation
- **Count queries** for accurate pagination metadata
- **Error handling** with fallback to empty states
- **Type-safe** pagination response structure

### 5. Enhanced Stock Service (`/lib/services/stock.service.ts`)
- **Count function** for total stock records
- **Caching** for both data and count queries
- **Performance optimization** with 5-minute TTL
- **Consistent error handling**

### 6. Paginated Stock Grid Component (`/components/shared/PaginatedStockGrid.tsx`)
- **Grid layout** with responsive design (1-3 columns based on screen size)
- **Items per page selector** with role-based options
- **Access control indicators** showing locked content for lower tiers
- **Loading states** during pagination
- **Upgrade prompts** for users hitting tier limits
- **Stock card display** with EPS growth, price, and ranking information

## Usage Examples

### Basic Usage
```tsx
import { PaginatedStockGrid } from '@/components/shared/PaginatedStockGrid';
import { fetchPaginatedStockData } from '@/app/actions/stockRankingPaginated';

export default async function StockPage() {
  const initialData = await fetchPaginatedStockData(1, 10);
  
  return <PaginatedStockGrid initialData={initialData} />;
}
```

### With Tabs Integration
```tsx
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';

<Tabs defaultValue="grid">
  <TabsList>
    <TabsTrigger value="grid">Grid View</TabsTrigger>
    <TabsTrigger value="table">Table View</TabsTrigger>
  </TabsList>
  <TabsContent value="grid">
    <PaginatedStockGrid initialData={initialData} />
  </TabsContent>
</Tabs>
```

## User Experience

### Tier-Based Access
- **Basic users**: See up to 5 items with upgrade prompts
- **Silver users**: Access to 25 items with larger page sizes
- **Gold users**: Access to 50 items with advanced features
- **Platinum users**: Full access to all 100+ items

### Visual Indicators
- **Locked content** shown with opacity and lock icons
- **Upgrade prompts** with clear call-to-action buttons
- **Loading states** during data fetching
- **Error handling** with user-friendly messages

### Performance
- **Server-side caching** reduces API calls
- **Optimistic updates** for better perceived performance
- **Proper loading states** prevent user confusion
- **Debounced navigation** prevents rapid-fire requests

## Files Modified/Created

### New Files
- `/components/ui/pagination.tsx` - Main pagination component
- `/hooks/usePagination.ts` - Pagination state management
- `/hooks/usePaginatedFeatureAccess.ts` - Role-based access control
- `/app/actions/stockRankingPaginated.ts` - Paginated data fetching
- `/components/shared/PaginatedStockGrid.tsx` - Main grid component
- `/app/rankings/page.tsx` - Standalone rankings page

### Modified Files
- `/lib/services/stock.service.ts` - Added count function
- `/app/analytics/page.tsx` - Added tabs with grid view

## API Response Format

```typescript
interface PaginatedStockData {
  data: StockFinancialData[];
  pagination: {
    page: number;
    limit: number;
    total: number;
    totalPages: number;
    hasNext: boolean;
    hasPrev: boolean;
  };
}
```

## Integration with Payment System

The pagination system is fully integrated with the existing payment/subscription system:
- Uses existing `useFeatureAccess` hook for tier detection
- Respects existing role-based access controls
- Integrates with upgrade flow via payment page
- Maintains consistent UX with other premium features

## Future Enhancements

1. **Search and Filters** - Add search functionality within paginated results
2. **Sorting Options** - Allow sorting by different criteria
3. **Bookmark Pages** - Save user's current page in URL or localStorage
4. **Infinite Scroll** - Alternative pagination pattern for mobile
5. **Bulk Operations** - Allow selections across multiple pages
6. **Export Options** - Export current page or all accessible data
