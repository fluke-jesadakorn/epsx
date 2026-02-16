'use client';

import { closestCenter, DndContext, DragOverlay } from '@dnd-kit/core';
import { rectSwappingStrategy, SortableContext } from '@dnd-kit/sortable';
import { useCardDashboardData } from '@/hooks/use-card-dashboard-data';
import { useSortableCards } from '@/hooks/use-sortable-cards';
import {
  DashboardHeader,
  EmptyState,
  ErrorView,
  FilterPanel,
  LoadingGrid,
  PaginationControls,
  SortableSymbolCard,
  StatusLegend,
  SymbolCard,
} from './card-dashboard-sections';

interface CardDashboardViewProps {
  className?: string;
}

export function CardDashboardView({ className = '' }: CardDashboardViewProps) {
  const {
    data,
    setData,
    loading,
    error,
    showFilters,
    setShowFilters,
    filters,
    filterOptions,
    updateFilters,
    resetFilters,
    handleExport,
    loadData,
  } = useCardDashboardData();

  const { sensors, activeId, handleDragStart, handleDragEnd } = useSortableCards({
    data,
    setData,
  });

  if (loading) {
    return (
      <div className={`space-y-6 ${className}`}>
        <LoadingGrid />
      </div>
    );
  }

  if (error !== null) {
    return <ErrorView error={error} onRetry={() => void loadData()} className={className} />;
  }

  return (
    <div className={`space-y-6 ${className}`}>
      {data && (
        <DashboardHeader
          dataLength={data.data.length}
          total={data.pagination.total}
          showFilters={showFilters}
          onToggleFilters={() => setShowFilters(!showFilters)}
          onRefresh={() => void loadData()}
          onExport={handleExport}
          loading={loading}
        />
      )}

      {showFilters && (
        <FilterPanel
          filters={filters}
          filterOptions={filterOptions}
          onUpdateFilters={updateFilters}
          onReset={resetFilters}
        />
      )}

      <StatusLegend />

      {data && (
        <div className="text-center text-sm text-gray-500 dark:text-gray-400">
          Processed in {data.processing_time_ms} ms
        </div>
      )}

      {data?.data && data.data.length > 0 ? (
        <DndContext
          sensors={sensors}
          collisionDetection={closestCenter}
          onDragStart={handleDragStart}
          onDragEnd={handleDragEnd}
        >
          <SortableContext
            items={data.data.map((d) => d.symbol)}
            strategy={rectSwappingStrategy}
          >
            <div className="grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
              {data.data.map((cardData) =>
                cardData.symbol !== undefined &&
                cardData.symbol !== null &&
                cardData.symbol !== '' ? (
                  <SortableSymbolCard key={cardData.symbol} cardData={cardData} />
                ) : null
              )}
            </div>
          </SortableContext>

          <DragOverlay>
            {activeId !== null && data.data.find((d) => d.symbol === activeId) ? (
              <SymbolCard
                cardData={data.data.find((d) => d.symbol === activeId)!}
                isOverlay
              />
            ) : null}
          </DragOverlay>
        </DndContext>
      ) : (
        <EmptyState onReset={resetFilters} />
      )}

      {data?.pagination && data.pagination.totalPages > 1 && (
        <PaginationControls
          page={filters.page}
          totalPages={data.pagination.totalPages}
          hasNext={data.pagination.hasNext}
          hasPrev={data.pagination.hasPrev}
          onPageChange={(page) => updateFilters({ page })}
        />
      )}
    </div>
  );
}
