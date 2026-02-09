'use client';

import { useCallback, useMemo, useState } from 'react';
import { usePlanTransferDrag } from './hooks/use-plan-transfer-drag';
import { usePlanTransferHandlers } from './hooks/use-plan-transfer-handlers';
import { PermissionList } from './ui/permission-list';

interface PermissionItem {
  id: string;
  name: string;
  code: string;
}

interface PlanTransferListProps {
  available: string[];
  selected: string[];
  onChange: (selected: string[]) => void;
}

function formatPermissionName(permission: string): string {
  const parts = permission.split(':');
  const relevantParts = parts.slice(1);
  return relevantParts
    .map(part => part.charAt(0).toUpperCase() + part.slice(1).replace(/_/g, ' '))
    .join(' ');
}

function toPermissionItem(permission: string): PermissionItem {
  return {
    id: permission,
    name: formatPermissionName(permission),
    code: permission,
  };
}

export function PlanTransferList({
  available: allAvailable,
  selected,
  onChange,
}: PlanTransferListProps) {
  const [leftSearch, setLeftSearch] = useState('');
  const [rightSearch, setRightSearch] = useState('');

  const availableItems = useMemo(() => {
    return allAvailable
      .filter((perm) => !selected.includes(perm))
      .map(toPermissionItem);
  }, [allAvailable, selected]);

  const selectedItems = useMemo(() => {
    return selected.map(toPermissionItem);
  }, [selected]);

  const filteredAvailable = useMemo(() => {
    const search = leftSearch.toLowerCase();
    return availableItems.filter(
      (item) =>
        item.name.toLowerCase().includes(search) ||
        item.code.toLowerCase().includes(search)
    );
  }, [availableItems, leftSearch]);

  const filteredSelected = useMemo(() => {
    const search = rightSearch.toLowerCase();
    return selectedItems.filter(
      (item) =>
        item.name.toLowerCase().includes(search) ||
        item.code.toLowerCase().includes(search)
    );
  }, [selectedItems, rightSearch]);

  const moveRight = useCallback(
    (item: PermissionItem) => {
      if (!selected.includes(item.id)) {
        onChange([...selected, item.id]);
      }
    },
    [selected, onChange]
  );

  const moveLeft = useCallback(
    (item: PermissionItem) => {
      onChange(selected.filter((id) => id !== item.id));
    },
    [selected, onChange]
  );

  const {
    draggingItem,
    setDraggingItem,
    touchDrag,
    dropTarget,
    availableListRef,
    selectedListRef,
    handleTouchStart,
    handleTouchMove,
    handleTouchEnd,
  } = usePlanTransferDrag({ moveRight, moveLeft });

  const handleDragStart = (
    e: React.DragEvent,
    item: PermissionItem,
    source: 'available' | 'selected'
  ) => {
    setDraggingItem(item.id);
    e.dataTransfer.setData('itemId', item.id);
    e.dataTransfer.setData('source', source);
    e.dataTransfer.effectAllowed = 'move';

    const target = e.target as HTMLElement;
    target.classList.add('opacity-40');
  };

  const handleDragEnd = (e: React.DragEvent) => {
    setDraggingItem(null);
    const target = e.target as HTMLElement;
    target.classList.remove('opacity-40');
  };

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    e.dataTransfer.dropEffect = 'move';
  };

  const handleDrop = (e: React.DragEvent, target: 'available' | 'selected') => {
    e.preventDefault();
    const itemId = e.dataTransfer.getData('itemId');
    const source = e.dataTransfer.getData('source');

    if (source !== target) {
      const item = toPermissionItem(itemId);
      if (target === 'selected') {
        moveRight(item);
      } else {
        moveLeft(item);
      }
    }
  };

  const handleItemClick = useCallback(
    (item: PermissionItem, source: 'available' | 'selected') => {
      if (touchDrag === null && draggingItem === null) {
        if (source === 'available') {
          moveRight(item);
        } else {
          moveLeft(item);
        }
      }
    },
    [touchDrag, draggingItem, moveRight, moveLeft]
  );

  return (
    <div className="space-y-4">
      <div className="flex items-center gap-2">
        <div className="p-1.5 rounded-lg bg-amber-500/20 border border-amber-500/30">
          <svg
            className="w-4 h-4 text-amber-500"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z"
            />
          </svg>
        </div>
        <span className="text-sm font-semibold text-gray-700 dark:text-gray-300">
          Select Permissions
        </span>
      </div>

      <div
        className="grid grid-cols-1 sm:grid-cols-2 gap-4 lg:gap-6"
        onTouchMove={handleTouchMove}
        onTouchEnd={handleTouchEnd}
        onTouchCancel={handleTouchEnd}
      >
        <PermissionList
          title="Available"
          items={filteredAvailable}
          count={availableItems.length}
          searchValue={leftSearch}
          onSearchChange={setLeftSearch}
          listRef={availableListRef}
          onDragOver={handleDragOver}
          onDrop={(e) => handleDrop(e, 'available')}
          dropTarget={dropTarget}
          touchSource={touchDrag?.source ?? null}
          source="available"
          draggingItem={draggingItem}
          onDragStart={(e, item) => handleDragStart(e, item, 'available')}
          onDragEnd={handleDragEnd}
          onTouchStart={(e, item) => handleTouchStart(e, item, 'available')}
          onItemClick={(item) => handleItemClick(item, 'available')}
          emptyMessage="All permissions selected"
          variant="available"
        />

        <PermissionList
          title="Authorized"
          items={filteredSelected}
          count={selectedItems.length}
          searchValue={rightSearch}
          onSearchChange={setRightSearch}
          listRef={selectedListRef}
          onDragOver={handleDragOver}
          onDrop={(e) => handleDrop(e, 'selected')}
          dropTarget={dropTarget}
          touchSource={touchDrag?.source ?? null}
          source="selected"
          draggingItem={draggingItem}
          onDragStart={(e, item) => handleDragStart(e, item, 'selected')}
          onDragEnd={handleDragEnd}
          onTouchStart={(e, item) => handleTouchStart(e, item, 'selected')}
          onItemClick={(item) => handleItemClick(item, 'selected')}
          emptyMessage="Drag or tap to authorize"
          variant="selected"
        />
      </div>
    </div>
  );
}
