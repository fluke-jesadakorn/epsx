import type { DragEndEvent, DragStartEvent } from '@dnd-kit/core';
import { KeyboardSensor, PointerSensor, useSensor, useSensors } from '@dnd-kit/core';
import { sortableKeyboardCoordinates } from '@dnd-kit/sortable';
import { useState } from 'react';

interface SymbolCardData {
  rank: number;
  symbol: string;
  latest_date: string;
  value: number;
  active_status: string;
  quarterly_performance: Array<{
    quarter: string;
    date: string;
    price: number;
    eps: number;
    eps_growth: number;
    price_growth: number;
    is_estimated?: boolean;
  }>;
  next_quarter_estimate?: {
    quarter: string;
    announcement_date: string;
    announcement_timestamp: number;
    days_until_announcement: number;
    estimated_eps: number;
    estimated_price_target?: number;
    confidence: string;
  };
  next_earnings_date?: number;
  last_earnings_date?: number;
  next_earnings_date_formatted?: string;
  days_until_next_earnings?: number;
  progress_percentage?: number;
}

interface CardDashboardResponse {
  success: boolean;
  data: SymbolCardData[];
  pagination: {
    page: number;
    limit: number;
    total: number;
    totalPages: number;
    hasNext: boolean;
    hasPrev: boolean;
  };
  metadata: {
    available_countries: string[];
    available_sectors: string[];
    request_timestamp: string;
    data_source: string;
  };
  message?: string;
  processing_time_ms: number;
}

interface UseSortableCardsContext {
  data: CardDashboardResponse | null;
  setData: React.Dispatch<React.SetStateAction<CardDashboardResponse | null>>;
}

export function useSortableCards(ctx: UseSortableCardsContext) {
  const [activeId, setActiveId] = useState<string | null>(null);

  const sensors = useSensors(
    useSensor(PointerSensor, {
      activationConstraint: {
        distance: 8,
      },
    }),
    useSensor(KeyboardSensor, {
      coordinateGetter: sortableKeyboardCoordinates,
    })
  );

  const handleDragStart = (event: DragStartEvent) => {
    const { active } = event;
    setActiveId(active.id as string);
  };

  const handleDragEnd = (event: DragEndEvent) => {
    const { active, over } = event;

    if (active.id !== over?.id && ctx.data) {
      ctx.setData((prev) => {
        if (!prev) {
          return null;
        }

        const oldIndex = prev.data.findIndex((item) => item.symbol === active.id);
        const newIndex = prev.data.findIndex((item) => item.symbol === over?.id);

        return {
          ...prev,
          data: (() => {
            const newArr = [...prev.data];
            [newArr[oldIndex], newArr[newIndex]] = [newArr[newIndex], newArr[oldIndex]];
            return newArr;
          })(),
        };
      });
    }

    setActiveId(null);
  };

  return {
    sensors,
    activeId,
    handleDragStart,
    handleDragEnd,
  };
}
