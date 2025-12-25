'use client';

import { Input } from '@/components/ui/input';
import { MouseEvent as ReactMouseEvent, TouchEvent, useCallback, useRef, useState } from 'react';

interface PermissionTransferListProps {
    available: string[];
    selected: string[];
    onChange: (selected: string[]) => void;
}

/**
 * Permission Transfer List Component
 * Allows drag-and-drop selection of individual permission strings
 */
export function PermissionTransferList({
    available,
    selected,
    onChange,
}: PermissionTransferListProps) {
    const [searchAvailable, setSearchAvailable] = useState('');
    const [searchAuthorized, setSearchAuthorized] = useState('');
    const [draggedPermission, setDraggedPermission] = useState<string | null>(null);
    const [dragOverSide, setDragOverSide] = useState<'available' | 'authorized' | null>(null);

    // Touch handling state
    const [touchStartTime, setTouchStartTime] = useState<number>(0);
    const [touchStartPos, setTouchStartPos] = useState<{ x: number; y: number } | null>(null);
    const [longPressActive, setLongPressActive] = useState(false);
    const longPressTimeout = useRef<ReturnType<typeof setTimeout> | null>(null);
    const LONG_PRESS_DURATION = 500; // ms

    // Get available permissions (not in selected)
    const availablePermissions = available.filter((p) => !selected.includes(p));

    // Search filtering
    const filteredAvailable = availablePermissions.filter(
        (p) => p.toLowerCase().includes(searchAvailable.toLowerCase())
    );

    const filteredSelected = selected.filter(
        (p) => p.toLowerCase().includes(searchAuthorized.toLowerCase())
    );

    // Move permission to authorized
    const moveToAuthorized = useCallback(
        (permission: string) => {
            if (!selected.includes(permission)) {
                onChange([...selected, permission]);
            }
        },
        [selected, onChange]
    );

    // Move permission to available
    const moveToAvailable = useCallback(
        (permission: string) => {
            onChange(selected.filter((p) => p !== permission));
        },
        [selected, onChange]
    );

    // Drag handlers for desktop
    const handleDragStart = (e: ReactMouseEvent | React.DragEvent, permission: string) => {
        e.stopPropagation();
        setDraggedPermission(permission);
        if ('dataTransfer' in e) {
            e.dataTransfer.effectAllowed = 'move';
        }
    };

    const handleDragEnd = () => {
        setDraggedPermission(null);
        setDragOverSide(null);
        setLongPressActive(false);
    };

    const handleDragOver = (e: React.DragEvent, side: 'available' | 'authorized') => {
        e.preventDefault();
        e.stopPropagation();
        setDragOverSide(side);
    };

    const handleDrop = (e: React.DragEvent, side: 'available' | 'authorized') => {
        e.preventDefault();
        e.stopPropagation();

        if (draggedPermission) {
            if (side === 'authorized' && !selected.includes(draggedPermission)) {
                moveToAuthorized(draggedPermission);
            } else if (side === 'available' && selected.includes(draggedPermission)) {
                moveToAvailable(draggedPermission);
            }
        }

        setDraggedPermission(null);
        setDragOverSide(null);
    };

    // Touch handlers for mobile
    const handleTouchStart = (e: TouchEvent, permission: string) => {
        const touch = e.touches[0];
        setTouchStartPos({ x: touch.clientX, y: touch.clientY });
        setTouchStartTime(Date.now());

        longPressTimeout.current = setTimeout(() => {
            setLongPressActive(true);
            setDraggedPermission(permission);
        }, LONG_PRESS_DURATION);
    };

    const handleTouchMove = (e: TouchEvent) => {
        if (longPressTimeout.current && !longPressActive) {
            const touch = e.touches[0];
            if (touchStartPos) {
                const dx = Math.abs(touch.clientX - touchStartPos.x);
                const dy = Math.abs(touch.clientY - touchStartPos.y);
                if (dx > 10 || dy > 10) {
                    clearTimeout(longPressTimeout.current);
                    longPressTimeout.current = null;
                }
            }
        }

        if (longPressActive && draggedPermission) {
            e.preventDefault();
            const touch = e.touches[0];
            const element = document.elementFromPoint(touch.clientX, touch.clientY);

            if (element?.closest('[data-drop-zone="authorized"]')) {
                setDragOverSide('authorized');
            } else if (element?.closest('[data-drop-zone="available"]')) {
                setDragOverSide('available');
            } else {
                setDragOverSide(null);
            }
        }
    };

    const handleTouchEnd = () => {
        if (longPressTimeout.current) {
            clearTimeout(longPressTimeout.current);
            longPressTimeout.current = null;
        }

        if (longPressActive && draggedPermission && dragOverSide) {
            if (dragOverSide === 'authorized' && !selected.includes(draggedPermission)) {
                moveToAuthorized(draggedPermission);
            } else if (dragOverSide === 'available' && selected.includes(draggedPermission)) {
                moveToAvailable(draggedPermission);
            }
        }

        setLongPressActive(false);
        setDraggedPermission(null);
        setDragOverSide(null);
        setTouchStartPos(null);
    };

    // Parse permission string to extract parts
    const parsePermission = (permission: string) => {
        const parts = permission.split(':');
        return {
            platform: parts[0] || '',
            resource: parts[1] || '',
            action: parts[2] || '',
        };
    };

    // Get category color based on platform
    const getCategoryColor = (platform: string) => {
        const colors: Record<string, string> = {
            epsx: 'bg-amber-100 text-amber-800 dark:bg-amber-900/30 dark:text-amber-400',
            admin: 'bg-purple-100 text-purple-800 dark:bg-purple-900/30 dark:text-purple-400',
            api: 'bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-400',
            default: 'bg-gray-100 text-gray-800 dark:bg-gray-900/30 dark:text-gray-400',
        };
        return colors[platform.toLowerCase()] || colors.default;
    };

    const PermissionItem = ({
        permission,
        isSelected,
        isDragging,
    }: {
        permission: string;
        isSelected: boolean;
        isDragging: boolean;
    }) => {
        const parsed = parsePermission(permission);

        return (
            <div
                draggable
                onDragStart={(e) => handleDragStart(e, permission)}
                onDragEnd={handleDragEnd}
                onTouchStart={(e) => handleTouchStart(e, permission)}
                onTouchMove={handleTouchMove}
                onTouchEnd={handleTouchEnd}
                onClick={() => (isSelected ? moveToAvailable(permission) : moveToAuthorized(permission))}
                className={`
          group flex items-center justify-between p-2.5 rounded-lg border cursor-move
          transition-all duration-150 select-none
          ${isDragging ? 'opacity-50 scale-95' : 'opacity-100'}
          ${isSelected
                        ? 'bg-gradient-to-r from-amber-50 to-orange-50 dark:from-amber-900/20 dark:to-orange-900/20 border-amber-200 dark:border-amber-700'
                        : 'bg-white dark:bg-gray-800 border-gray-200 dark:border-gray-700'
                    }
          hover:shadow-md hover:border-amber-300 dark:hover:border-amber-600
        `}
            >
                <div className="flex items-center gap-2 min-w-0 flex-1">
                    {/* Drag handle icon */}
                    <div className="flex-shrink-0 text-gray-400 group-hover:text-amber-500 transition-colors">
                        <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 8h16M4 16h16" />
                        </svg>
                    </div>

                    {/* Platform badge */}
                    <span className={`flex-shrink-0 text-xs px-1.5 py-0.5 rounded-md font-medium ${getCategoryColor(parsed.platform)}`}>
                        {parsed.platform}
                    </span>

                    {/* Permission string */}
                    <span className="text-sm font-mono text-gray-700 dark:text-gray-300 truncate">
                        {parsed.resource}{parsed.action ? `:${parsed.action}` : ''}
                    </span>
                </div>

                {/* Action arrow */}
                <div className="flex-shrink-0 ml-2 text-gray-400 group-hover:text-amber-500 transition-colors">
                    <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path
                            strokeLinecap="round"
                            strokeLinejoin="round"
                            strokeWidth={2}
                            d={isSelected ? 'M15 19l-7-7 7-7' : 'M9 5l7 7-7 7'}
                        />
                    </svg>
                </div>
            </div>
        );
    };

    return (
        <div className="space-y-3">
            {/* Header */}
            <div className="flex items-center gap-2">
                <svg className="w-5 h-5 text-amber-500" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
                </svg>
                <span className="text-sm font-semibold text-gray-700 dark:text-gray-200">
                    Select Permissions
                </span>
                <span className="text-xs text-gray-500 dark:text-gray-400">
                    (Drag or click to move)
                </span>
            </div>

            {/* Two Column Layout */}
            <div className="grid grid-cols-2 gap-4">
                {/* Available (Left) */}
                <div
                    data-drop-zone="available"
                    onDragOver={(e) => handleDragOver(e, 'available')}
                    onDrop={(e) => handleDrop(e, 'available')}
                    className={`
            rounded-xl border-2 transition-colors
            ${dragOverSide === 'available'
                            ? 'border-amber-400 bg-amber-50/50 dark:bg-amber-900/10'
                            : 'border-gray-200 dark:border-gray-700'
                        }
          `}
                >
                    <div className="p-3 border-b border-gray-200 dark:border-gray-700">
                        <div className="flex items-center justify-between mb-2">
                            <h4 className="text-sm font-medium text-gray-600 dark:text-gray-400">Available</h4>
                            <span className="text-xs px-2 py-0.5 rounded-full bg-gray-100 dark:bg-gray-800 text-gray-600 dark:text-gray-400">
                                {filteredAvailable.length}
                            </span>
                        </div>
                        <Input
                            placeholder="Search permissions..."
                            value={searchAvailable}
                            onChange={(e) => setSearchAvailable(e.target.value)}
                            className="h-8 text-sm"
                        />
                    </div>
                    <div className="p-2 h-64 overflow-y-auto space-y-1.5">
                        {filteredAvailable.length === 0 ? (
                            <div className="flex items-center justify-center h-full text-sm text-gray-400">
                                {availablePermissions.length === 0 ? 'All permissions selected' : 'No matches'}
                            </div>
                        ) : (
                            filteredAvailable.map((permission) => (
                                <PermissionItem
                                    key={permission}
                                    permission={permission}
                                    isSelected={false}
                                    isDragging={draggedPermission === permission}
                                />
                            ))
                        )}
                    </div>
                </div>

                {/* Authorized (Right) */}
                <div
                    data-drop-zone="authorized"
                    onDragOver={(e) => handleDragOver(e, 'authorized')}
                    onDrop={(e) => handleDrop(e, 'authorized')}
                    className={`
            rounded-xl border-2 transition-colors
            ${dragOverSide === 'authorized'
                            ? 'border-amber-400 bg-amber-50/50 dark:bg-amber-900/10'
                            : 'border-amber-200 dark:border-amber-700 bg-gradient-to-b from-amber-50/30 to-orange-50/30 dark:from-amber-900/10 dark:to-orange-900/10'
                        }
          `}
                >
                    <div className="p-3 border-b border-amber-200 dark:border-amber-700">
                        <div className="flex items-center justify-between mb-2">
                            <h4 className="text-sm font-medium text-amber-700 dark:text-amber-400">Authorized</h4>
                            <span className="text-xs px-2 py-0.5 rounded-full bg-amber-100 dark:bg-amber-900/30 text-amber-700 dark:text-amber-400">
                                {selected.length}
                            </span>
                        </div>
                        <Input
                            placeholder="Search selected..."
                            value={searchAuthorized}
                            onChange={(e) => setSearchAuthorized(e.target.value)}
                            className="h-8 text-sm"
                        />
                    </div>
                    <div className="p-2 h-64 overflow-y-auto space-y-1.5">
                        {filteredSelected.length === 0 ? (
                            <div className="flex items-center justify-center h-full text-sm text-gray-400">
                                {selected.length === 0 ? 'Drag permissions here' : 'No matches'}
                            </div>
                        ) : (
                            filteredSelected.map((permission) => (
                                <PermissionItem
                                    key={permission}
                                    permission={permission}
                                    isSelected={true}
                                    isDragging={draggedPermission === permission}
                                />
                            ))
                        )}
                    </div>
                </div>
            </div>
        </div>
    );
}
