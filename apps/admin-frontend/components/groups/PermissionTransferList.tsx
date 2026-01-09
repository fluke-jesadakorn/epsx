'use client'

import { ChevronLeft, ChevronRight, GripVertical, Loader2, Plus, Search, Shield, Trash2, X } from 'lucide-react'
import React, { useCallback, useMemo, useRef, useState } from 'react'
import { toast } from 'sonner'

import { cn } from '@/lib/utils'
import { Input } from '@/shared/components'

interface PermissionTransferListProps {
    available: string[]
    selected: string[]
    onChange: (selected: string[]) => void
    /** Optional async callback when a new custom permission is created (for DB persistence) */
    onCreatePermission?: (permission: string) => Promise<void>
    /** Optional async callback when a permission is deleted (for DB persistence) */
    onDeletePermission?: (permission: string) => Promise<void>
    /** Set of system permissions that cannot be deleted */
    systemPermissions?: Set<string>
    /** Loading state for external data */
    isLoading?: boolean
}

interface TouchDragState {
    item: string
    source: 'available' | 'selected'
    startY: number
    currentY: number
    element: HTMLElement | null
}

/**
 * Validates permission string format: platform:resource:action
 * Allows wildcards (*) and multi-segment resources (e.g., epsx:analytics:view)
 * @param permission
 */
const isValidPermissionFormat = (permission: string): boolean => {
    // Basic format check: at least 2 colons, non-empty segments
    const parts = permission.split(':')
    if (parts.length < 3) {return false}
    // Each part should be either a valid identifier, wildcard, or dash-separated words
    return parts.every(part =>
        part === '*' || /^[a-zA-Z0-9_-]+$/.test(part)
    )
}

/**
 *
 * @param root0
 * @param root0.available
 * @param root0.selected
 * @param root0.onChange
 * @param root0.onCreatePermission
 * @param root0.onDeletePermission
 * @param root0.systemPermissions
 * @param root0.isLoading
 */
export const PermissionTransferList: React.FC<PermissionTransferListProps> = ({
    available: allAvailable,
    selected,
    onChange,
    onCreatePermission,
    onDeletePermission,
    systemPermissions = new Set(),
    isLoading = false
}) => {
    const [leftSearch, setLeftSearch] = useState('')
    const [rightSearch, setRightSearch] = useState('')
    const [draggingItem, setDraggingItem] = useState<string | null>(null)
    const [customPermission, setCustomPermission] = useState('')
    const [isCreating, setIsCreating] = useState(false)
    const [deletingPermission, setDeletingPermission] = useState<string | null>(null)

    // Touch drag state
    const [touchDrag, setTouchDrag] = useState<TouchDragState | null>(null)
    const [dropTarget, setDropTarget] = useState<'available' | 'selected' | null>(null)
    const longPressTimer = useRef<ReturnType<typeof setTimeout> | null>(null)
    const availableListRef = useRef<HTMLDivElement>(null)
    const selectedListRef = useRef<HTMLDivElement>(null)

    // Filter out selected ones from available
    const availableItems = useMemo(() => {
        return allAvailable.filter(item => !selected.includes(item))
    }, [allAvailable, selected])

    // Handler to add a custom permission
    const handleAddCustomPermission = useCallback(async () => {
        const trimmed = customPermission.trim().toLowerCase()

        if (!trimmed) {
            toast.error('Please enter a permission string')
            return
        }

        if (!isValidPermissionFormat(trimmed)) {
            toast.error('Invalid format. Use: platform:resource:action (e.g., epsx:analytics:view)')
            return
        }

        // Check if already exists
        if (allAvailable.includes(trimmed) || selected.includes(trimmed)) {
            toast.error('This permission already exists')
            return
        }

        // If we have a create callback, use it for DB persistence
        if (onCreatePermission) {
            setIsCreating(true)
            try {
                await onCreatePermission(trimmed)
                setCustomPermission('')
                toast.success(`Permission "${trimmed}" created and saved`)
            } catch (error: any) {
                toast.error(error.message || 'Failed to create permission')
            } finally {
                setIsCreating(false)
            }
        } else {
            // Local only - no persistence
            setCustomPermission('')
            toast.success(`Permission "${trimmed}" added (not persisted)`)
        }
    }, [customPermission, allAvailable, selected, onCreatePermission])

    // Handler to delete a permission
    const handleDeletePermission = useCallback(async (permission: string, e: React.MouseEvent) => {
        e.stopPropagation()

        if (systemPermissions.has(permission)) {
            toast.error('System permissions cannot be deleted')
            return
        }

        if (!onDeletePermission) {
            toast.error('Deletion is not enabled')
            return
        }

        setDeletingPermission(permission)
        try {
            await onDeletePermission(permission)
            toast.success(`Permission "${permission}" deleted`)
        } catch (error: any) {
            toast.error(error.message || 'Failed to delete permission')
        } finally {
            setDeletingPermission(null)
        }
    }, [onDeletePermission, systemPermissions])

    // Handle enter key in custom permission input
    const handleCustomPermissionKeyDown = useCallback((e: React.KeyboardEvent<HTMLInputElement>) => {
        if (e.key === 'Enter') {
            e.preventDefault()
            handleAddCustomPermission()
        }
    }, [handleAddCustomPermission])

    // Filtered lists based on search
    const filteredAvailable = useMemo(() => {
        console.log('Filtering available items:', {
            total: availableItems.length,
            search: leftSearch,
            sample: availableItems.slice(0, 3)
        })
        return availableItems.filter(item => {
            if (!item || typeof item !== 'string') {
                console.warn('Invalid item found in availableItems:', item)
                return false
            }
            const match = item.toLowerCase().includes(leftSearch.toLowerCase())
            if (match) {console.log('Match found:', item)}
            return match
        })
    }, [availableItems, leftSearch])

    const filteredSelected = useMemo(() => {
        return selected.filter(item =>
            item && typeof item === 'string' && item.toLowerCase().includes(rightSearch.toLowerCase())
        )
    }, [selected, rightSearch])

    // Move item from available to selected
    const moveRight = useCallback((item: string) => {
        if (!selected.includes(item)) {
            onChange([...selected, item])
        }
    }, [selected, onChange])

    // Move item from selected to available
    const moveLeft = useCallback((item: string) => {
        onChange(selected.filter(i => i !== item))
    }, [selected, onChange])

    // Check if touch is over a drop zone
    const getDropZone = useCallback((touchX: number, touchY: number): 'available' | 'selected' | null => {
        const availableRect = availableListRef.current?.getBoundingClientRect()
        const selectedRect = selectedListRef.current?.getBoundingClientRect()

        if (availableRect &&
            touchX >= availableRect.left && touchX <= availableRect.right &&
            touchY >= availableRect.top && touchY <= availableRect.bottom) {
            return 'available'
        }

        if (selectedRect &&
            touchX >= selectedRect.left && touchX <= selectedRect.right &&
            touchY >= selectedRect.top && touchY <= selectedRect.bottom) {
            return 'selected'
        }

        return null
    }, [])

    // Touch handlers for mobile drag and drop
    const handleTouchStart = useCallback((e: React.TouchEvent, item: string, source: 'available' | 'selected') => {
        const touch = e.touches[0]
        if (!touch) {return}

        const element = e.currentTarget as HTMLElement

        // Start long press timer for drag initiation
        longPressTimer.current = setTimeout(() => {
            // Vibrate on supported devices for haptic feedback
            if (navigator.vibrate) {
                navigator.vibrate(50)
            }

            setTouchDrag({
                item,
                source,
                startY: touch.clientY,
                currentY: touch.clientY,
                element
            })
            setDraggingItem(item)
            element.classList.add('scale-105', 'shadow-lg', 'z-50')
        }, 200) // 200ms long press to start drag
    }, [])

    const handleTouchMove = useCallback((e: React.TouchEvent) => {
        const touch = e.touches[0]
        if (!touch) {return}

        if (!touchDrag) {
            // If not dragging yet, cancel the long press timer on significant movement
            if (longPressTimer.current) {
                const startTouch = e.currentTarget.getBoundingClientRect()
                const moveDistance = Math.abs(touch.clientY - startTouch.top)
                if (moveDistance > 10) {
                    clearTimeout(longPressTimer.current)
                    longPressTimer.current = null
                }
            }
            return
        }

        e.preventDefault() // Prevent scrolling while dragging

        setTouchDrag(prev => prev ? { ...prev, currentY: touch.clientY } : null)

        // Check which drop zone we're over
        const zone = getDropZone(touch.clientX, touch.clientY)
        setDropTarget(zone)
    }, [touchDrag, getDropZone])

    const handleTouchEnd = useCallback(() => {
        // Clear long press timer
        if (longPressTimer.current) {
            clearTimeout(longPressTimer.current)
            longPressTimer.current = null
        }

        if (touchDrag) {
            // Reset element styling
            if (touchDrag.element) {
                touchDrag.element.classList.remove('scale-105', 'shadow-lg', 'z-50')
            }

            // Perform the drop if over a different zone
            if (dropTarget && dropTarget !== touchDrag.source) {
                if (dropTarget === 'selected') {
                    moveRight(touchDrag.item)
                } else {
                    moveLeft(touchDrag.item)
                }
            }

            setTouchDrag(null)
            setDraggingItem(null)
            setDropTarget(null)
        }
    }, [touchDrag, dropTarget, moveRight, moveLeft])

    // DnD Handlers (Desktop)
    const handleDragStart = (e: React.DragEvent, item: string, source: 'available' | 'selected') => {
        setDraggingItem(item)
        e.dataTransfer.setData('item', item)
        e.dataTransfer.setData('source', source)
        e.dataTransfer.effectAllowed = 'move'

        // Add custom drag image styling if needed
        const target = e.target as HTMLElement
        target.classList.add('opacity-40')
    }

    const handleDragEnd = (e: React.DragEvent) => {
        setDraggingItem(null)
        const target = e.target as HTMLElement
        target.classList.remove('opacity-40')
    }

    const handleDragOver = (e: React.DragEvent) => {
        e.preventDefault()
        e.dataTransfer.dropEffect = 'move'
    }

    const handleDrop = (e: React.DragEvent, target: 'available' | 'selected') => {
        e.preventDefault()
        const item = e.dataTransfer.getData('item')
        const source = e.dataTransfer.getData('source')

        if (source !== target) {
            if (target === 'selected') {
                moveRight(item)
            } else {
                moveLeft(item)
            }
        }
    }

    // Handle click/tap (works for both desktop and mobile quick tap)
    const handleItemClick = useCallback((item: string, source: 'available' | 'selected') => {
        // Only trigger on tap (not after drag)
        if (!touchDrag && !draggingItem) {
            if (source === 'available') {
                moveRight(item)
            } else {
                moveLeft(item)
            }
        }
    }, [touchDrag, draggingItem, moveRight, moveLeft])

    return (
        <div className="space-y-4">
            {/* Create Custom Permission Section */}
            <div className="p-4 rounded-2xl border border-purple-500/20 bg-purple-500/5 backdrop-blur-sm">
                <div className="flex items-center gap-2 mb-3">
                    <div className="p-1.5 rounded-lg bg-purple-500/20 border border-purple-500/30">
                        <Plus className="w-3.5 h-3.5 text-purple-400" />
                    </div>
                    <span className="text-xs uppercase font-bold tracking-wider text-purple-400">
                        Create Custom Permission
                    </span>
                </div>
                <div className="flex gap-2">
                    <div className="relative flex-1 group">
                        <Shield className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-purple-600/50 group-focus-within:text-purple-400 transition-colors" />
                        <Input
                            value={customPermission}
                            onChange={(e) => setCustomPermission(e.target.value)}
                            onKeyDown={handleCustomPermissionKeyDown}
                            placeholder="platform:resource:action (e.g., epsx:analytics:view)"
                            className="pl-10 pr-4 bg-white/5 border-purple-500/20 hover:border-purple-500/30 focus:border-purple-500/50 h-11 rounded-xl text-sm font-mono placeholder:text-gray-600"
                        />
                    </div>
                    <button
                        type="button"
                        onClick={handleAddCustomPermission}
                        disabled={!customPermission.trim() || isCreating}
                        className={cn(
                            "flex items-center justify-center gap-2 px-4 h-11 rounded-xl font-semibold text-sm transition-all duration-200",
                            (customPermission.trim() && !isCreating)
                                ? "bg-gradient-to-r from-purple-600 to-indigo-600 hover:from-purple-500 hover:to-indigo-500 text-white shadow-lg shadow-purple-900/30"
                                : "bg-white/5 text-gray-600 cursor-not-allowed border border-white/10"
                        )}
                    >
                        {isCreating ? (
                            <Loader2 className="w-4 h-4 animate-spin" />
                        ) : (
                            <Plus className="w-4 h-4" />
                        )}
                        <span className="hidden sm:inline">{isCreating ? 'Saving...' : 'Add'}</span>
                    </button>
                </div>
                <p className="mt-2 text-xs text-gray-500">
                    Format: <code className="text-purple-400/80 bg-purple-500/10 px-1.5 py-0.5 rounded">platform:resource:action</code>
                    {' '}— Use <code className="text-purple-400/80 bg-purple-500/10 px-1.5 py-0.5 rounded">*</code> for wildcards
                </p>
            </div>

            {/* Transfer Lists */}
            <div
                className="grid grid-cols-1 sm:grid-cols-2 gap-4 lg:gap-8 p-1"
                onTouchMove={handleTouchMove}
                onTouchEnd={handleTouchEnd}
                onTouchCancel={handleTouchEnd}
            >
                {/* Available List */}
                <div className="flex flex-col space-y-3">
                    <div className="flex items-center justify-between px-1">
                        <h3 className="text-xs uppercase font-bold tracking-widest text-gray-500 flex items-center gap-2">
                            Available <span className="bg-white/5 px-2 py-0.5 rounded-full text-[10px]">{availableItems.length}</span>
                        </h3>
                    </div>

                    <div className="relative group">
                        <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-600 group-focus-within:text-blue-500 transition-colors" />
                        <Input
                            value={leftSearch}
                            onChange={(e) => setLeftSearch(e.target.value)}
                            placeholder="Search available..."
                            className="pl-10 bg-white/5 border-white/10 h-10 rounded-xl focus:border-blue-500/50"
                        />
                        {leftSearch && (
                            <button
                                onClick={() => setLeftSearch('')}
                                className="absolute right-3 top-1/2 -translate-y-1/2 text-gray-600 hover:text-white"
                            >
                                <X className="w-3.5 h-3.5" />
                            </button>
                        )}
                    </div>

                    <div
                        ref={availableListRef}
                        onDragOver={handleDragOver}
                        onDrop={(e) => handleDrop(e, 'available')}
                        className={cn(
                            "flex flex-col gap-2 p-2 rounded-2xl border border-white/5 bg-white/[0.02] h-[300px] overflow-y-auto custom-scrollbar transition-all",
                            "hover:border-white/10 group/list",
                            dropTarget === 'available' && touchDrag?.source === 'selected' && "border-green-500/50 bg-green-500/5 ring-2 ring-green-500/20"
                        )}
                    >
                        {filteredAvailable.length === 0 ? (
                            <div className="flex flex-col items-center justify-center h-full text-gray-600 space-y-2">
                                <Shield className="w-8 h-8 opacity-10" />
                                <p className="text-sm italic">{leftSearch ? 'No matches found' : 'All permissions assigned'}</p>
                            </div>
                        ) : (
                            filteredAvailable.map(item => (
                                <div
                                    key={item}
                                    draggable
                                    onDragStart={(e) => handleDragStart(e, item, 'available')}
                                    onDragEnd={handleDragEnd}
                                    onTouchStart={(e) => handleTouchStart(e, item, 'available')}
                                    onClick={() => handleItemClick(item, 'available')}
                                    className={cn(
                                        "flex items-center justify-between p-3 rounded-xl border border-white/5 bg-white/5 cursor-grab active:cursor-grabbing hover:bg-white/10 hover:border-blue-500/20 transition-all group/item select-none touch-manipulation",
                                        draggingItem === item && "border-blue-500/50 bg-blue-500/5 opacity-60"
                                    )}
                                >
                                    <div className="flex items-center gap-3 flex-1 min-w-0">
                                        <GripVertical className="w-4 h-4 text-gray-700 group-hover/item:text-blue-500/50 flex-shrink-0" />
                                        <span className="text-sm font-medium text-gray-300 group-hover/item:text-white truncate">{item}</span>
                                    </div>
                                    <div className="flex items-center gap-1 flex-shrink-0">
                                        {/* Delete button - only show for non-system permissions when onDeletePermission is available */}
                                        {onDeletePermission && !systemPermissions.has(item) && (
                                            <button
                                                type="button"
                                                onClick={(e) => handleDeletePermission(item, e)}
                                                disabled={deletingPermission === item}
                                                className="p-1.5 rounded-lg text-gray-600 hover:text-red-400 hover:bg-red-500/10 transition-all opacity-0 group-hover/item:opacity-100"
                                                title="Delete permission"
                                            >
                                                {deletingPermission === item ? (
                                                    <Loader2 className="w-3.5 h-3.5 animate-spin" />
                                                ) : (
                                                    <Trash2 className="w-3.5 h-3.5" />
                                                )}
                                            </button>
                                        )}
                                        <ChevronRight className="w-4 h-4 text-gray-700 group-hover/item:text-blue-500 transition-all" />
                                    </div>
                                </div>
                            ))
                        )}
                    </div>
                </div>

                {/* Selected List */}
                <div className="flex flex-col space-y-3">
                    <div className="flex items-center justify-between px-1">
                        <h3 className="text-xs uppercase font-bold tracking-widest text-blue-500/70 flex items-center gap-2">
                            Authorized <span className="bg-blue-500/10 px-2 py-0.5 rounded-full text-[10px] text-blue-400 border border-blue-500/20">{selected.length}</span>
                        </h3>
                    </div>

                    <div className="relative group">
                        <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-600 group-focus-within:text-blue-500 transition-colors" />
                        <Input
                            value={rightSearch}
                            onChange={(e) => setRightSearch(e.target.value)}
                            placeholder="Search authorized..."
                            className="pl-10 bg-white/5 border-white/10 h-10 rounded-xl focus:border-blue-500/50"
                        />
                        {rightSearch && (
                            <button
                                onClick={() => setRightSearch('')}
                                className="absolute right-3 top-1/2 -translate-y-1/2 text-gray-600 hover:text-white"
                            >
                                <X className="w-3.5 h-3.5" />
                            </button>
                        )}
                    </div>

                    <div
                        ref={selectedListRef}
                        onDragOver={handleDragOver}
                        onDrop={(e) => handleDrop(e, 'selected')}
                        className={cn(
                            "flex flex-col gap-2 p-2 rounded-2xl border border-blue-500/10 bg-blue-500/[0.02] h-[300px] overflow-y-auto custom-scrollbar transition-all",
                            "hover:border-blue-500/20 ring-1 ring-blue-500/0 hover:ring-blue-500/5",
                            dropTarget === 'selected' && touchDrag?.source === 'available' && "border-blue-500/50 bg-blue-500/10 ring-2 ring-blue-500/30"
                        )}
                    >
                        {filteredSelected.length === 0 ? (
                            <div className="flex flex-col items-center justify-center h-full text-blue-500/20 space-y-2">
                                <Shield className="w-8 h-8" />
                                <p className="text-sm italic">{rightSearch ? 'No matches found' : 'Drag or tap to authorize'}</p>
                            </div>
                        ) : (
                            filteredSelected.map(item => (
                                <div
                                    key={item}
                                    draggable
                                    onDragStart={(e) => handleDragStart(e, item, 'selected')}
                                    onDragEnd={handleDragEnd}
                                    onTouchStart={(e) => handleTouchStart(e, item, 'selected')}
                                    onClick={() => handleItemClick(item, 'selected')}
                                    className={cn(
                                        "flex items-center justify-between p-3 rounded-xl border border-blue-500/10 bg-blue-500/10 cursor-grab active:cursor-grabbing hover:bg-blue-500/20 hover:border-red-500/20 transition-all group/item select-none touch-manipulation",
                                        draggingItem === item && "border-red-500/50 bg-red-500/5 opacity-60"
                                    )}
                                >
                                    <div className="flex items-center gap-3">
                                        <GripVertical className="w-4 h-4 text-blue-500/30 group-hover/item:text-blue-400 flex-shrink-0" />
                                        <span className="text-sm font-medium text-blue-300 group-hover/item:text-white truncate max-w-[180px] sm:max-w-none">{item}</span>
                                    </div>
                                    <ChevronLeft className="w-4 h-4 text-gray-700 group-hover/item:text-red-500 transition-all flex-shrink-0" />
                                </div>
                            ))
                        )}
                    </div>
                </div>

                <style jsx global>{`
                .custom-scrollbar::-webkit-scrollbar {
                    width: 4px;
                }
                .custom-scrollbar::-webkit-scrollbar-track {
                    background: transparent;
                }
                .custom-scrollbar::-webkit-scrollbar-thumb {
                    background: rgba(255, 255, 255, 0.05);
                    border-radius: 10px;
                }
                .custom-scrollbar::-webkit-scrollbar-thumb:hover {
                    background: rgba(255, 255, 255, 0.1);
                }
                .touch-manipulation {
                    touch-action: manipulation;
                }
            `}</style>
            </div>
        </div>
    )
}
