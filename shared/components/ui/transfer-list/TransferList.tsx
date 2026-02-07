'use client'

import { CheckSquare, ChevronLeft, ChevronRight, Search, Shield, Square, X } from 'lucide-react'
import React, { useCallback, useMemo, useRef, useState } from 'react'

import { cn } from '@/shared/utils/cn'
import { Button } from '../button'
import { Input } from '../input'

export interface TransferListProps<T> {
    available: T[]
    selected: T[]
    onChange: (selected: T[]) => void
    renderItem: (item: T, type: 'available' | 'selected') => React.ReactNode
    keyExtractor: (item: T) => string
    filterItem?: (item: T, query: string) => boolean
    availableTitle?: string
    selectedTitle?: string
    availableSearchPlaceholder?: string
    selectedSearchPlaceholder?: string
    className?: string
    isLoading?: boolean
    emptyStateAvailable?: React.ReactNode
    emptyStateSelected?: React.ReactNode
    /** Enable selection mode with checkboxes and arrow buttons */
    showSelection?: boolean
}

interface TouchDragState<T> {
    item: T
    source: 'available' | 'selected'
    startY: number
    currentY: number
    element: HTMLElement | null
}

export function TransferList<T>({
    available: allAvailable,
    selected,
    onChange,
    renderItem,
    keyExtractor,
    filterItem,
    availableTitle = 'Available',
    selectedTitle = 'Selected',
    availableSearchPlaceholder = 'Search available...',
    selectedSearchPlaceholder = 'Search selected...',
    className,
    isLoading = false,
    emptyStateAvailable,
    emptyStateSelected,
    showSelection = false,
}: TransferListProps<T>) {
    const [leftSearch, setLeftSearch] = useState('')
    const [rightSearch, setRightSearch] = useState('')
    const [draggingItem, setDraggingItem] = useState<T | null>(null)

    // Selection state for bulk operations
    const [selectedAvailable, setSelectedAvailable] = useState<Set<string>>(new Set())
    const [selectedSelected, setSelectedSelected] = useState<Set<string>>(new Set())

    // Touch drag state
    const [touchDrag, setTouchDrag] = useState<TouchDragState<T> | null>(null)
    const [dropTarget, setDropTarget] = useState<'available' | 'selected' | null>(null)
    const longPressTimer = useRef<ReturnType<typeof setTimeout> | null>(null)
    const availableListRef = useRef<HTMLDivElement>(null)
    const selectedListRef = useRef<HTMLDivElement>(null)

    // Default filter function
    const defaultFilter = useCallback((item: T, query: string) => {
        return keyExtractor(item).toLowerCase().includes(query.toLowerCase())
    }, [keyExtractor])

    const activeFilter = filterItem || defaultFilter

    const filteredAvailable = useMemo(() => {
        const filtered = allAvailable.filter(item => activeFilter(item, leftSearch))
        const seen = new Set<string>()
        return filtered.filter(item => {
            const key = keyExtractor(item)
            if (seen.has(key)) {return false}
            seen.add(key)
            return true
        })
    }, [allAvailable, leftSearch, activeFilter, keyExtractor])

    const filteredSelected = useMemo(() => {
        const filtered = selected.filter(item => activeFilter(item, rightSearch))
        const seen = new Set<string>()
        return filtered.filter(item => {
            const key = keyExtractor(item)
            if (seen.has(key)) {return false}
            seen.add(key)
            return true
        })
    }, [selected, rightSearch, activeFilter, keyExtractor])

    // Move item from available to selected
    const moveRight = useCallback((item: T) => {
        const key = keyExtractor(item)
        if (!selected.some(i => keyExtractor(i) === key)) {
            onChange([...selected, item])
        }
        // Clear from selection
        setSelectedAvailable(prev => {
            const next = new Set(prev)
            next.delete(key)
            return next
        })
    }, [selected, onChange, keyExtractor])

    // Move item from selected to available
    const moveLeft = useCallback((item: T) => {
        const key = keyExtractor(item)
        onChange(selected.filter(i => keyExtractor(i) !== key))
        // Clear from selection
        setSelectedSelected(prev => {
            const next = new Set(prev)
            next.delete(key)
            return next
        })
    }, [selected, onChange, keyExtractor])

    // Bulk move selected items to the right
    const bulkMoveRight = useCallback(() => {
        if (selectedAvailable.size === 0) {return}
        const itemsToMove = allAvailable.filter(item => selectedAvailable.has(keyExtractor(item)))
        const existingKeys = new Set(selected.map(keyExtractor))
        const newItems = itemsToMove.filter(item => !existingKeys.has(keyExtractor(item)))
        onChange([...selected, ...newItems])
        setSelectedAvailable(new Set())
    }, [allAvailable, selected, selectedAvailable, keyExtractor, onChange])

    // Bulk move selected items to the left
    const bulkMoveLeft = useCallback(() => {
        if (selectedSelected.size === 0) {return}
        onChange(selected.filter(item => !selectedSelected.has(keyExtractor(item))))
        setSelectedSelected(new Set())
    }, [selected, selectedSelected, keyExtractor, onChange])

    // Selection toggle handlers
    const toggleAvailableSelection = useCallback((item: T) => {
        const key = keyExtractor(item)
        setSelectedAvailable(prev => {
            const next = new Set(prev)
            if (next.has(key)) {
                next.delete(key)
            } else {
                next.add(key)
            }
            return next
        })
    }, [keyExtractor])

    const toggleSelectedSelection = useCallback((item: T) => {
        const key = keyExtractor(item)
        setSelectedSelected(prev => {
            const next = new Set(prev)
            if (next.has(key)) {
                next.delete(key)
            } else {
                next.add(key)
            }
            return next
        })
    }, [keyExtractor])

    // Select all handlers
    const selectAllAvailable = useCallback((select: boolean) => {
        if (select) {
            setSelectedAvailable(new Set(filteredAvailable.map(keyExtractor)))
        } else {
            setSelectedAvailable(new Set())
        }
    }, [filteredAvailable, keyExtractor])

    const selectAllSelected = useCallback((select: boolean) => {
        if (select) {
            setSelectedSelected(new Set(filteredSelected.map(keyExtractor)))
        } else {
            setSelectedSelected(new Set())
        }
    }, [filteredSelected, keyExtractor])

    const allAvailableSelected = filteredAvailable.length > 0 && filteredAvailable.every(item => selectedAvailable.has(keyExtractor(item)))
    const allSelectedSelected = filteredSelected.length > 0 && filteredSelected.every(item => selectedSelected.has(keyExtractor(item)))

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

    // Touch handlers
    const handleTouchStart = useCallback((e: React.TouchEvent, item: T, source: 'available' | 'selected') => {
        const touch = e.touches[0]
        if (!touch) {return}

        const element = e.currentTarget as HTMLElement

        longPressTimer.current = setTimeout(() => {
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
        }, 200)
    }, [])

    const handleTouchMove = useCallback((e: React.TouchEvent) => {
        const touch = e.touches[0]
        if (!touch) {return}

        if (!touchDrag) {
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

        e.preventDefault()
        setTouchDrag(prev => prev ? { ...prev, currentY: touch.clientY } : null)

        const zone = getDropZone(touch.clientX, touch.clientY)
        setDropTarget(zone)
    }, [touchDrag, getDropZone])

    const handleTouchEnd = useCallback(() => {
        if (longPressTimer.current) {
            clearTimeout(longPressTimer.current)
            longPressTimer.current = null
        }

        if (touchDrag) {
            if (touchDrag.element) {
                touchDrag.element.classList.remove('scale-105', 'shadow-lg', 'z-50')
            }

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
    const handleDragStart = (e: React.DragEvent, item: T, source: 'available' | 'selected') => {
        setDraggingItem(item)
        e.dataTransfer.setData('key', keyExtractor(item))
        e.dataTransfer.setData('source', source)
        e.dataTransfer.effectAllowed = 'move'

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
        const source = e.dataTransfer.getData('source')

        if (!draggingItem) {return}

        if (source !== target) {
            if (target === 'selected') {
                moveRight(draggingItem)
            } else {
                moveLeft(draggingItem)
            }
        }
    }

    const handleItemClick = useCallback((item: T, source: 'available' | 'selected') => {
        if (!touchDrag && !draggingItem) {
            if (source === 'available') {
                moveRight(item)
            } else {
                moveLeft(item)
            }
        }
    }, [touchDrag, draggingItem, moveRight, moveLeft])

    // Render item wrapper with optional checkbox
    const renderItemWithSelection = useCallback((item: T, type: 'available' | 'selected') => {
        if (!showSelection) {
            return renderItem(item, type)
        }

        const key = keyExtractor(item)
        const isChecked = type === 'available'
            ? selectedAvailable.has(key)
            : selectedSelected.has(key)

        return (
            <div className="flex items-center gap-2">
                <button
                    onClick={(e) => {
                        e.stopPropagation()
                        if (type === 'available') {
                            toggleAvailableSelection(item)
                        } else {
                            toggleSelectedSelection(item)
                        }
                    }}
                    className="p-1.5 hover:bg-gray-100 dark:hover:bg-gray-700 rounded flex-shrink-0"
                >
                    {isChecked ? (
                        <CheckSquare className="h-4 w-4 text-blue-600" />
                    ) : (
                        <Square className="h-4 w-4 text-gray-400" />
                    )}
                </button>
                <div className="flex-1 min-w-0">
                    {renderItem(item, type)}
                </div>
            </div>
        )
    }, [showSelection, keyExtractor, selectedAvailable, selectedSelected, toggleAvailableSelection, toggleSelectedSelection, renderItem])

    return (
        <div
            className={cn(
                "grid gap-4 lg:gap-6",
                showSelection
                    ? "grid-cols-1 md:grid-cols-[1fr_auto_1fr]"
                    : "grid-cols-1 sm:grid-cols-2",
                className
            )}
            onTouchMove={handleTouchMove}
            onTouchEnd={handleTouchEnd}
            onTouchCancel={handleTouchEnd}
        >
            {/* Available List */}
            <div className="flex flex-col space-y-3 min-h-0">
                <div className="flex items-center justify-between px-1 flex-shrink-0">
                    <h3 className="text-xs uppercase font-bold tracking-widest text-gray-500 flex items-center gap-2">
                        {availableTitle} <span className="bg-white/5 px-2 py-0.5 rounded-full text-[10px]">{allAvailable.length}</span>
                    </h3>
                </div>

                <div className="relative group flex-shrink-0">
                    <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-600 group-focus-within:text-blue-500 transition-colors" />
                    <Input
                        value={leftSearch}
                        onChange={(e) => setLeftSearch(e.target.value)}
                        placeholder={availableSearchPlaceholder}
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

                {/* Select All (only when showSelection is true) */}
                {showSelection && filteredAvailable.length > 0 && (
                    <div className="flex items-center px-2 flex-shrink-0">
                        <button
                            onClick={() => selectAllAvailable(!allAvailableSelected)}
                            className="flex items-center gap-2 text-xs text-gray-500 hover:text-gray-900 dark:hover:text-white"
                        >
                            {allAvailableSelected ? (
                                <CheckSquare className="h-3.5 w-3.5 text-blue-600" />
                            ) : (
                                <Square className="h-3.5 w-3.5" />
                            )}
                            Select All
                        </button>
                    </div>
                )}

                <div
                    ref={availableListRef}
                    onDragOver={handleDragOver}
                    onDrop={(e) => handleDrop(e, 'available')}
                    className={cn(
                        "flex flex-col gap-2 p-2 rounded-2xl border border-white/5 bg-white/[0.02] flex-1 min-h-[200px] overflow-y-auto custom-scrollbar transition-all",
                        "hover:border-white/10 group/list",
                        dropTarget === 'available' && touchDrag?.source === 'selected' && "border-green-500/50 bg-green-500/5 ring-2 ring-green-500/20"
                    )}
                >
                    {filteredAvailable.length === 0 ? (
                        emptyStateAvailable || (
                            <div className="flex flex-col items-center justify-center h-full text-gray-600 space-y-2">
                                <Shield className="w-8 h-8 opacity-10" />
                                <p className="text-sm italic">{leftSearch ? 'No matches found' : 'No items available'}</p>
                            </div>
                        )
                    ) : (
                        filteredAvailable.map(item => (
                            <div
                                key={keyExtractor(item)}
                                draggable
                                onDragStart={(e) => handleDragStart(e, item, 'available')}
                                onDragEnd={handleDragEnd}
                                onTouchStart={(e) => handleTouchStart(e, item, 'available')}
                                onClick={() => handleItemClick(item, 'available')}
                                className={cn(
                                    "cursor-grab active:cursor-grabbing transition-all select-none touch-manipulation",
                                    draggingItem && keyExtractor(draggingItem) === keyExtractor(item) && "opacity-60"
                                )}
                            >
                                {renderItemWithSelection(item, 'available')}
                            </div>
                        ))
                    )}
                </div>
            </div>

            {/* Central Arrow Buttons (only when showSelection is true) */}
            {showSelection && (
                <div className="flex flex-row md:flex-col items-center justify-center gap-2 py-2 md:py-0">
                    <Button
                        variant="secondary"
                        size="icon"
                        disabled={selectedAvailable.size === 0}
                        onClick={bulkMoveRight}
                        className={cn(
                            "rounded-full transition-all shadow-sm w-10 h-10 md:w-12 md:h-12",
                            selectedAvailable.size > 0
                                ? "bg-white dark:bg-gray-700 text-green-600 hover:bg-green-50 hover:text-green-700 hover:shadow-md border-green-200"
                                : "opacity-50"
                        )}
                        title="Move Selected Right"
                    >
                        <ChevronRight className="h-5 w-5 md:h-6 md:w-6 transform rotate-90 md:rotate-0" />
                    </Button>

                    <Button
                        variant="secondary"
                        size="icon"
                        disabled={selectedSelected.size === 0}
                        onClick={bulkMoveLeft}
                        className={cn(
                            "rounded-full transition-all shadow-sm w-10 h-10 md:w-12 md:h-12",
                            selectedSelected.size > 0
                                ? "bg-white dark:bg-gray-700 text-red-600 hover:bg-red-50 hover:text-red-700 hover:shadow-md border-red-200"
                                : "opacity-50"
                        )}
                        title="Move Selected Left"
                    >
                        <ChevronLeft className="h-5 w-5 md:h-6 md:w-6 transform rotate-90 md:rotate-0" />
                    </Button>
                </div>
            )}

            {/* Selected List */}
            <div className="flex flex-col space-y-3 min-h-0">
                <div className="flex items-center justify-between px-1 flex-shrink-0">
                    <h3 className="text-xs uppercase font-bold tracking-widest text-blue-500/70 flex items-center gap-2">
                        {selectedTitle} <span className="bg-blue-500/10 px-2 py-0.5 rounded-full text-[10px] text-blue-400 border border-blue-500/20">{selected.length}</span>
                    </h3>
                </div>

                <div className="relative group flex-shrink-0">
                    <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-600 group-focus-within:text-blue-500 transition-colors" />
                    <Input
                        value={rightSearch}
                        onChange={(e) => setRightSearch(e.target.value)}
                        placeholder={selectedSearchPlaceholder}
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

                {/* Select All (only when showSelection is true) */}
                {showSelection && filteredSelected.length > 0 && (
                    <div className="flex items-center px-2 flex-shrink-0">
                        <button
                            onClick={() => selectAllSelected(!allSelectedSelected)}
                            className="flex items-center gap-2 text-xs text-gray-500 hover:text-gray-900 dark:hover:text-white"
                        >
                            {allSelectedSelected ? (
                                <CheckSquare className="h-3.5 w-3.5 text-blue-600" />
                            ) : (
                                <Square className="h-3.5 w-3.5" />
                            )}
                            Select All
                        </button>
                    </div>
                )}

                <div
                    ref={selectedListRef}
                    onDragOver={handleDragOver}
                    onDrop={(e) => handleDrop(e, 'selected')}
                    className={cn(
                        "flex flex-col gap-2 p-2 rounded-2xl border border-blue-500/10 bg-blue-500/[0.02] flex-1 min-h-[200px] overflow-y-auto custom-scrollbar transition-all",
                        "hover:border-blue-500/20 ring-1 ring-blue-500/0 hover:ring-blue-500/5",
                        dropTarget === 'selected' && touchDrag?.source === 'available' && "border-blue-500/50 bg-blue-500/10 ring-2 ring-blue-500/30"
                    )}
                >
                    {filteredSelected.length === 0 ? (
                        emptyStateSelected || (
                            <div className="flex flex-col items-center justify-center h-full text-blue-500/20 space-y-2">
                                <Shield className="w-8 h-8" />
                                <p className="text-sm italic">{rightSearch ? 'No matches found' : 'Drag items here'}</p>
                            </div>
                        )
                    ) : (
                        filteredSelected.map(item => (
                            <div
                                key={keyExtractor(item)}
                                draggable
                                onDragStart={(e) => handleDragStart(e, item, 'selected')}
                                onDragEnd={handleDragEnd}
                                onTouchStart={(e) => handleTouchStart(e, item, 'selected')}
                                onClick={() => handleItemClick(item, 'selected')}
                                className={cn(
                                    "cursor-grab active:cursor-grabbing transition-all select-none touch-manipulation",
                                    draggingItem && keyExtractor(draggingItem) === keyExtractor(item) && "opacity-60"
                                )}
                            >
                                {renderItemWithSelection(item, 'selected')}
                            </div>
                        ))
                    )}
                </div>
            </div>

            <style dangerouslySetInnerHTML={{
                __html: `
        .custom-scrollbar::-webkit-scrollbar {
            width: 8px;
            height: 8px;
        }
        .custom-scrollbar::-webkit-scrollbar-track {
            background: transparent;
        }
        .custom-scrollbar::-webkit-scrollbar-thumb {
            background: rgba(255, 255, 255, 0.1);
            border-radius: 20px;
            border: 2px solid transparent;
            background-clip: content-box;
        }
        .custom-scrollbar::-webkit-scrollbar-thumb:hover {
            background: rgba(255, 255, 255, 0.2);
            background-clip: content-box;
        }
        .dark .custom-scrollbar::-webkit-scrollbar-thumb {
            background: rgba(255, 255, 255, 0.15);
            background-clip: content-box;
        }
        .dark .custom-scrollbar::-webkit-scrollbar-thumb:hover {
            background: rgba(255, 255, 255, 0.25);
            background-clip: content-box;
        }
        .touch-manipulation {
            touch-action: manipulation;
        }
    ` }} />
        </div>
    )
}
