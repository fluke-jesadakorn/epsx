'use client';

import { useCallback, useRef, useState } from 'react';

interface PermissionItem {
  id: string;
  name: string;
  code: string;
}

interface TouchDragState {
  item: PermissionItem;
  source: 'available' | 'selected';
  startY: number;
  currentY: number;
  element: HTMLElement | null;
}

interface UsePlanTransferDragContext {
  moveRight: (item: PermissionItem) => void;
  moveLeft: (item: PermissionItem) => void;
}

export function usePlanTransferDrag(ctx: UsePlanTransferDragContext) {
  const [draggingItem, setDraggingItem] = useState<string | null>(null);
  const [touchDrag, setTouchDrag] = useState<TouchDragState | null>(null);
  const [dropTarget, setDropTarget] = useState<'available' | 'selected' | null>(null);
  const longPressTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
  const availableListRef = useRef<HTMLDivElement>(null);
  const selectedListRef = useRef<HTMLDivElement>(null);

  const getDropZone = useCallback(
    (touchX: number, touchY: number): 'available' | 'selected' | null => {
      const availableRect = availableListRef.current?.getBoundingClientRect();
      const selectedRect = selectedListRef.current?.getBoundingClientRect();

      const inAvailable =
        availableRect !== undefined &&
        touchX >= availableRect.left &&
        touchX <= availableRect.right &&
        touchY >= availableRect.top &&
        touchY <= availableRect.bottom;

      const inSelected =
        selectedRect !== undefined &&
        touchX >= selectedRect.left &&
        touchX <= selectedRect.right &&
        touchY >= selectedRect.top &&
        touchY <= selectedRect.bottom;

      if (inAvailable) {return 'available';}
      if (inSelected) {return 'selected';}
      return null;
    },
    []
  );

  const handleTouchStart = useCallback(
    (e: React.TouchEvent, item: PermissionItem, source: 'available' | 'selected') => {
      const touch = e.touches[0];
      if (touch === undefined) {return;}

      const element = e.currentTarget as HTMLElement;

      longPressTimer.current = setTimeout(() => {
        if (navigator.vibrate !== undefined) {
          navigator.vibrate(50);
        }

        setTouchDrag({
          item,
          source,
          startY: touch.clientY,
          currentY: touch.clientY,
          element,
        });
        setDraggingItem(item.id);
        element.classList.add('scale-105', 'shadow-lg', 'z-50');
      }, 200);
    },
    []
  );

  const handleTouchMove = useCallback(
    (e: React.TouchEvent) => {
      const touch = e.touches[0];
      if (touch === undefined) {return;}

      if (touchDrag === null) {
        if (longPressTimer.current !== null) {
          const startTouch = e.currentTarget.getBoundingClientRect();
          const moveDistance = Math.abs(touch.clientY - startTouch.top);
          if (moveDistance > 10) {
            clearTimeout(longPressTimer.current);
            longPressTimer.current = null;
          }
        }
        return;
      }

      e.preventDefault();
      setTouchDrag((prev) => (prev !== null ? { ...prev, currentY: touch.clientY } : null));

      const zone = getDropZone(touch.clientX, touch.clientY);
      setDropTarget(zone);
    },
    [touchDrag, getDropZone]
  );

  const handleTouchEnd = useCallback(() => {
    if (longPressTimer.current !== null) {
      clearTimeout(longPressTimer.current);
      longPressTimer.current = null;
    }

    if (touchDrag !== null) {
      if (touchDrag.element !== null) {
        touchDrag.element.classList.remove('scale-105', 'shadow-lg', 'z-50');
      }

      if (dropTarget !== null && dropTarget !== touchDrag.source) {
        if (dropTarget === 'selected') {
          ctx.moveRight(touchDrag.item);
        } else {
          ctx.moveLeft(touchDrag.item);
        }
      }

      setTouchDrag(null);
      setDraggingItem(null);
      setDropTarget(null);
    }
  }, [touchDrag, dropTarget, ctx]);

  return {
    draggingItem,
    setDraggingItem,
    touchDrag,
    dropTarget,
    availableListRef,
    selectedListRef,
    handleTouchStart,
    handleTouchMove,
    handleTouchEnd,
  };
}
