'use client';

import { Input } from '@/components/ui/input';
import { useCallback, useMemo, useRef, useState } from 'react';

interface PermissionItem {
    id: string;      // The permission string itself (e.g., "epsx:read:platform")
    name: string;    // Human-readable name (e.g., "Read Platform")
    code: string;    // Same as id, the permission code
}

interface PlanTransferListProps {
    available: string[]; // Array of permission strings
    selected: string[];  // Array of selected permission strings
    onChange: (selected: string[]) => void;
}

interface TouchDragState {
    item: PermissionItem;
    source: 'available' | 'selected';
    startY: number;
    currentY: number;
    element: HTMLElement | null;
}

// Convert permission string to human-readable name
// e.g., "epsx:read:platform" -> "Read Platform"
function formatPermissionName(permission: string): string {
    const parts = permission.split(':');
    // Take the last 2 parts (action:resource or just resource)
    const relevantParts = parts.slice(1); // Remove platform prefix (epsx, admin, etc.)
    return relevantParts
        .map(part => part.charAt(0).toUpperCase() + part.slice(1).replace(/_/g, ' '))
        .join(' ');
}

// Convert permission string to PermissionItem
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
    const [draggingItem, setDraggingItem] = useState<string | null>(null);

    // Touch drag state
    const [touchDrag, setTouchDrag] = useState<TouchDragState | null>(null);
    const [dropTarget, setDropTarget] = useState<'available' | 'selected' | null>(null);
    const longPressTimer = useRef<ReturnType<typeof setTimeout> | null>(null);
    const availableListRef = useRef<HTMLDivElement>(null);
    const selectedListRef = useRef<HTMLDivElement>(null);

    // Convert to PermissionItem objects and filter out selected
    const availableItems = useMemo(() => {
        return allAvailable
            .filter((perm) => !selected.includes(perm))
            .map(toPermissionItem);
    }, [allAvailable, selected]);

    // Get selected permission items
    const selectedItems = useMemo(() => {
        return selected.map(toPermissionItem);
    }, [selected]);

    // Filtered lists based on search
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

    // Move item from available to selected
    const moveRight = useCallback(
        (item: PermissionItem) => {
            if (!selected.includes(item.id)) {
                onChange([...selected, item.id]);
            }
        },
        [selected, onChange]
    );

    // Move item from selected to available
    const moveLeft = useCallback(
        (item: PermissionItem) => {
            onChange(selected.filter((id) => id !== item.id));
        },
        [selected, onChange]
    );

    // Check if touch is over a drop zone
    const getDropZone = useCallback(
        (touchX: number, touchY: number): 'available' | 'selected' | null => {
            const availableRect = availableListRef.current?.getBoundingClientRect();
            const selectedRect = selectedListRef.current?.getBoundingClientRect();

            if (
                availableRect &&
                touchX >= availableRect.left &&
                touchX <= availableRect.right &&
                touchY >= availableRect.top &&
                touchY <= availableRect.bottom
            ) {
                return 'available';
            }

            if (
                selectedRect &&
                touchX >= selectedRect.left &&
                touchX <= selectedRect.right &&
                touchY >= selectedRect.top &&
                touchY <= selectedRect.bottom
            ) {
                return 'selected';
            }

            return null;
        },
        []
    );

    // Touch handlers for mobile drag and drop
    const handleTouchStart = useCallback(
        (e: React.TouchEvent, item: PermissionItem, source: 'available' | 'selected') => {
            const touch = e.touches[0];
            if (!touch) return;

            const element = e.currentTarget as HTMLElement;

            // Start long press timer for drag initiation
            longPressTimer.current = setTimeout(() => {
                // Vibrate on supported devices for haptic feedback
                if (navigator.vibrate) {
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
            if (!touch) return;

            if (!touchDrag) {
                // If not dragging yet, cancel the long press timer on significant movement
                if (longPressTimer.current) {
                    const startTouch = e.currentTarget.getBoundingClientRect();
                    const moveDistance = Math.abs(touch.clientY - startTouch.top);
                    if (moveDistance > 10) {
                        clearTimeout(longPressTimer.current);
                        longPressTimer.current = null;
                    }
                }
                return;
            }

            e.preventDefault(); // Prevent scrolling while dragging

            setTouchDrag((prev) => (prev ? { ...prev, currentY: touch.clientY } : null));

            // Check which drop zone we're over
            const zone = getDropZone(touch.clientX, touch.clientY);
            setDropTarget(zone);
        },
        [touchDrag, getDropZone]
    );

    const handleTouchEnd = useCallback(() => {
        // Clear long press timer
        if (longPressTimer.current) {
            clearTimeout(longPressTimer.current);
            longPressTimer.current = null;
        }

        if (touchDrag) {
            // Reset element styling
            if (touchDrag.element) {
                touchDrag.element.classList.remove('scale-105', 'shadow-lg', 'z-50');
            }

            // Perform the drop if over a different zone
            if (dropTarget && dropTarget !== touchDrag.source) {
                if (dropTarget === 'selected') {
                    moveRight(touchDrag.item);
                } else {
                    moveLeft(touchDrag.item);
                }
            }

            setTouchDrag(null);
            setDraggingItem(null);
            setDropTarget(null);
        }
    }, [touchDrag, dropTarget, moveRight, moveLeft]);

    // DnD Handlers (Desktop)
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

    // Handle click/tap (works for both desktop and mobile quick tap)
    const handleItemClick = useCallback(
        (item: PermissionItem, source: 'available' | 'selected') => {
            // Only trigger on tap (not after drag)
            if (!touchDrag && !draggingItem) {
                if (source === 'available') {
                    moveRight(item);
                } else {
                    moveLeft(item);
                }
            }
        },
        [touchDrag, draggingItem, moveRight, moveLeft]
    );

    const cn = (...classes: (string | boolean | undefined)[]) =>
        classes.filter(Boolean).join(' ');

    return (
        <div className="space-y-4">
            {/* Section Title */}
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

            {/* Transfer Lists */}
            <div
                className="grid grid-cols-1 sm:grid-cols-2 gap-4 lg:gap-6"
                onTouchMove={handleTouchMove}
                onTouchEnd={handleTouchEnd}
                onTouchCancel={handleTouchEnd}
            >
                {/* Available List */}
                <div className="flex flex-col space-y-2">
                    <div className="flex items-center justify-between px-1">
                        <h3 className="text-xs uppercase font-bold tracking-widest text-gray-500 flex items-center gap-2">
                            Available{' '}
                            <span className="bg-gray-100 dark:bg-gray-800 px-2 py-0.5 rounded-full text-[10px]">
                                {availableItems.length}
                            </span>
                        </h3>
                    </div>

                    <div className="relative group">
                        <svg
                            className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400 group-focus-within:text-amber-500 transition-colors"
                            fill="none"
                            stroke="currentColor"
                            viewBox="0 0 24 24"
                        >
                            <path
                                strokeLinecap="round"
                                strokeLinejoin="round"
                                strokeWidth={2}
                                d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
                            />
                        </svg>
                        <Input
                            value={leftSearch}
                            onChange={(e) => setLeftSearch(e.target.value)}
                            placeholder="Search available..."
                            className="pl-10 bg-gray-50 dark:bg-gray-800/50 border-gray-200 dark:border-gray-700 h-10 rounded-xl focus:border-amber-500/50"
                        />
                        {leftSearch && (
                            <button
                                onClick={() => setLeftSearch('')}
                                className="absolute right-3 top-1/2 -translate-y-1/2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
                            >
                                <svg className="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                                </svg>
                            </button>
                        )}
                    </div>

                    <div
                        ref={availableListRef}
                        onDragOver={handleDragOver}
                        onDrop={(e) => handleDrop(e, 'available')}
                        className={cn(
                            'flex flex-col gap-2 p-2 rounded-2xl border border-gray-200 dark:border-gray-700 bg-gray-50 dark:bg-gray-900/50 h-[250px] overflow-y-auto transition-all',
                            'hover:border-gray-300 dark:hover:border-gray-600',
                            dropTarget === 'available' &&
                            touchDrag?.source === 'selected' &&
                            'border-green-500/50 bg-green-500/5 ring-2 ring-green-500/20'
                        )}
                        style={{ scrollbarWidth: 'thin' }}
                    >
                        {filteredAvailable.length === 0 ? (
                            <div className="flex flex-col items-center justify-center h-full text-gray-400 space-y-2">
                                <svg className="w-8 h-8 opacity-30" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
                                </svg>
                                <p className="text-sm italic">
                                    {leftSearch ? 'No matches found' : 'All permissions selected'}
                                </p>
                            </div>
                        ) : (
                            filteredAvailable.map((item) => (
                                <div
                                    key={item.id}
                                    draggable
                                    onDragStart={(e) => handleDragStart(e, item, 'available')}
                                    onDragEnd={handleDragEnd}
                                    onTouchStart={(e) => handleTouchStart(e, item, 'available')}
                                    onClick={() => handleItemClick(item, 'available')}
                                    className={cn(
                                        'flex items-center justify-between p-3 rounded-xl border border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800/50 cursor-grab active:cursor-grabbing hover:bg-amber-50 dark:hover:bg-amber-900/10 hover:border-amber-300 dark:hover:border-amber-700 transition-all group/item select-none',
                                        draggingItem === item.id && 'border-amber-500/50 bg-amber-500/5 opacity-60'
                                    )}
                                    style={{ touchAction: 'manipulation' }}
                                >
                                    <div className="flex items-center gap-3 flex-1 min-w-0">
                                        <svg
                                            className="w-4 h-4 text-gray-400 group-hover/item:text-amber-500/50 flex-shrink-0"
                                            fill="none"
                                            stroke="currentColor"
                                            viewBox="0 0 24 24"
                                        >
                                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 8h16M4 16h16" />
                                        </svg>
                                        <div className="flex flex-col min-w-0">
                                            <span className="text-sm font-medium text-gray-700 dark:text-gray-300 group-hover/item:text-amber-700 dark:group-hover/item:text-amber-400 truncate">
                                                {item.name}
                                            </span>
                                            <span className="text-xs text-gray-400 font-mono truncate">{item.code}</span>
                                        </div>
                                    </div>
                                    <svg
                                        className="w-4 h-4 text-gray-400 group-hover/item:text-amber-500 transition-all flex-shrink-0"
                                        fill="none"
                                        stroke="currentColor"
                                        viewBox="0 0 24 24"
                                    >
                                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
                                    </svg>
                                </div>
                            ))
                        )}
                    </div>
                </div>

                {/* Selected List */}
                <div className="flex flex-col space-y-2">
                    <div className="flex items-center justify-between px-1">
                        <h3 className="text-xs uppercase font-bold tracking-widest text-amber-600 dark:text-amber-500 flex items-center gap-2">
                            Authorized{' '}
                            <span className="bg-amber-100 dark:bg-amber-900/30 text-amber-600 dark:text-amber-400 px-2 py-0.5 rounded-full text-[10px] border border-amber-200 dark:border-amber-800">
                                {selectedItems.length}
                            </span>
                        </h3>
                    </div>

                    <div className="relative group">
                        <svg
                            className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400 group-focus-within:text-amber-500 transition-colors"
                            fill="none"
                            stroke="currentColor"
                            viewBox="0 0 24 24"
                        >
                            <path
                                strokeLinecap="round"
                                strokeLinejoin="round"
                                strokeWidth={2}
                                d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z"
                            />
                        </svg>
                        <Input
                            value={rightSearch}
                            onChange={(e) => setRightSearch(e.target.value)}
                            placeholder="Search authorized..."
                            className="pl-10 bg-gray-50 dark:bg-gray-800/50 border-gray-200 dark:border-gray-700 h-10 rounded-xl focus:border-amber-500/50"
                        />
                        {rightSearch && (
                            <button
                                onClick={() => setRightSearch('')}
                                className="absolute right-3 top-1/2 -translate-y-1/2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
                            >
                                <svg className="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                                </svg>
                            </button>
                        )}
                    </div>

                    <div
                        ref={selectedListRef}
                        onDragOver={handleDragOver}
                        onDrop={(e) => handleDrop(e, 'selected')}
                        className={cn(
                            'flex flex-col gap-2 p-2 rounded-2xl border border-amber-200 dark:border-amber-800/30 bg-amber-50 dark:bg-amber-900/10 h-[250px] overflow-y-auto transition-all',
                            'hover:border-amber-300 dark:hover:border-amber-700 ring-1 ring-amber-500/0 hover:ring-amber-500/5',
                            dropTarget === 'selected' &&
                            touchDrag?.source === 'available' &&
                            'border-amber-500/50 bg-amber-500/10 ring-2 ring-amber-500/30'
                        )}
                        style={{ scrollbarWidth: 'thin' }}
                    >
                        {filteredSelected.length === 0 ? (
                            <div className="flex flex-col items-center justify-center h-full text-amber-500/30 space-y-2">
                                <svg className="w-8 h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={1.5} d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
                                </svg>
                                <p className="text-sm italic text-amber-500/50">
                                    {rightSearch ? 'No matches found' : 'Drag or tap to authorize'}
                                </p>
                            </div>
                        ) : (
                            filteredSelected.map((item) => (
                                <div
                                    key={item.id}
                                    draggable
                                    onDragStart={(e) => handleDragStart(e, item, 'selected')}
                                    onDragEnd={handleDragEnd}
                                    onTouchStart={(e) => handleTouchStart(e, item, 'selected')}
                                    onClick={() => handleItemClick(item, 'selected')}
                                    className={cn(
                                        'flex items-center justify-between p-3 rounded-xl border border-amber-300 dark:border-amber-700/50 bg-amber-100 dark:bg-amber-900/20 cursor-grab active:cursor-grabbing hover:bg-amber-200 dark:hover:bg-amber-800/30 hover:border-red-300 dark:hover:border-red-700 transition-all group/item select-none',
                                        draggingItem === item.id && 'border-red-500/50 bg-red-500/5 opacity-60'
                                    )}
                                    style={{ touchAction: 'manipulation' }}
                                >
                                    <div className="flex items-center gap-3 min-w-0">
                                        <svg
                                            className="w-4 h-4 text-amber-500/50 group-hover/item:text-amber-500 flex-shrink-0"
                                            fill="none"
                                            stroke="currentColor"
                                            viewBox="0 0 24 24"
                                        >
                                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 8h16M4 16h16" />
                                        </svg>
                                        <div className="flex flex-col min-w-0">
                                            <span className="text-sm font-medium text-amber-700 dark:text-amber-400 group-hover/item:text-gray-700 dark:group-hover/item:text-white truncate">
                                                {item.name}
                                            </span>
                                            <span className="text-xs text-amber-600/60 dark:text-amber-500/60 font-mono truncate">{item.code}</span>
                                        </div>
                                    </div>
                                    <svg
                                        className="w-4 h-4 text-gray-400 group-hover/item:text-red-500 transition-all flex-shrink-0"
                                        fill="none"
                                        stroke="currentColor"
                                        viewBox="0 0 24 24"
                                    >
                                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
                                    </svg>
                                </div>
                            ))
                        )}
                    </div>
                </div>
            </div>
        </div>
    );
}
