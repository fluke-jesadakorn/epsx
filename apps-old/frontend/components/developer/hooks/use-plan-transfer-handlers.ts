'use client';

import { useCallback } from 'react';

interface PermissionItem {
  id: string;
  name: string;
  code: string;
}

function toPermissionItem(permission: string): PermissionItem {
  const parts = permission.split(':');
  const relevantParts = parts.slice(1);
  const name = relevantParts
    .map(part => part.charAt(0).toUpperCase() + part.slice(1).replace(/_/g, ' '))
    .join(' ');

  return {
    id: permission,
    name,
    code: permission,
  };
}

interface UsePlanTransferHandlersContext {
  setDraggingItem: (id: string | null) => void;
  moveRight: (item: PermissionItem) => void;
  moveLeft: (item: PermissionItem) => void;
  touchDrag: unknown | null;
  draggingItem: string | null;
}

export function usePlanTransferHandlers(ctx: UsePlanTransferHandlersContext) {
  const handleDragStart = useCallback((
    e: React.DragEvent,
    item: PermissionItem,
    source: 'available' | 'selected'
  ) => {
    ctx.setDraggingItem(item.id);
    e.dataTransfer.setData('itemId', item.id);
    e.dataTransfer.setData('source', source);
    e.dataTransfer.effectAllowed = 'move';

    const target = e.target as HTMLElement;
    target.classList.add('opacity-40');
  }, [ctx]);

  const handleDragEnd = useCallback((e: React.DragEvent) => {
    ctx.setDraggingItem(null);
    const target = e.target as HTMLElement;
    target.classList.remove('opacity-40');
  }, [ctx]);

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    e.dataTransfer.dropEffect = 'move';
  }, []);

  const handleDrop = useCallback((e: React.DragEvent, target: 'available' | 'selected') => {
    e.preventDefault();
    const itemId = e.dataTransfer.getData('itemId');
    const source = e.dataTransfer.getData('source');

    if (source !== target) {
      const item = toPermissionItem(itemId);
      if (target === 'selected') {
        ctx.moveRight(item);
      } else {
        ctx.moveLeft(item);
      }
    }
  }, [ctx]);

  const handleItemClick = useCallback(
    (item: PermissionItem, source: 'available' | 'selected') => {
      if (ctx.touchDrag === null && ctx.draggingItem === null) {
        if (source === 'available') {
          ctx.moveRight(item);
        } else {
          ctx.moveLeft(item);
        }
      }
    },
    [ctx]
  );

  return {
    handleDragStart,
    handleDragEnd,
    handleDragOver,
    handleDrop,
    handleItemClick,
  };
}
