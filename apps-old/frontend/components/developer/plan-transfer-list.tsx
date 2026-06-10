'use client';

import { usePlanTransferDrag } from './hooks/use-plan-transfer-drag';
import { usePlanTransferHandlers } from './hooks/use-plan-transfer-handlers';
import { usePlanTransferState } from './hooks/use-plan-transfer-state';
import { PermissionList } from './ui/permission-list';

interface PlanTransferListProps {
  available: string[];
  selected: string[];
  onChange: (selected: string[]) => void;
}

export function PlanTransferList({
  available,
  selected,
  onChange,
}: PlanTransferListProps) {
  const {
    leftSearch,
    setLeftSearch,
    rightSearch,
    setRightSearch,
    availableItems,
    selectedItems,
    filteredAvailable,
    filteredSelected,
    moveRight,
    moveLeft,
  } = usePlanTransferState({ available, selected, onChange });

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

  const {
    handleDragStart,
    handleDragEnd,
    handleDragOver,
    handleDrop,
    handleItemClick,
  } = usePlanTransferHandlers({
    setDraggingItem,
    moveRight,
    moveLeft,
    touchDrag,
    draggingItem,
  });

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
