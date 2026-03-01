'use client'

import { CheckSquare, ChevronLeft, ChevronRight, Search, Shield, Square, X } from 'lucide-react'
import React, { useCallback } from 'react'

import { cn } from '@/shared/utils/cn'
import { Button } from '../button'
import { Input } from '../input'

import type { TouchDragState } from './use-transfer-list-state'
import { useTransferListState } from './use-transfer-list-state'

export interface TransferListProps<T> {
    available: T[]
    selected: T[]
    onChange: (selected: T[]) => void
    renderItem: (item: T, type: 'available' | 'selected') => React.ReactNode
    /**
     * Unique key extractor for items
     * @param item The item to extract key from
     */
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

/**
 * Sub-components and Helpers for TransferList
 */

interface ColumnProps<T> {
    title: string
    count: number
    search: string
    onSearchChange: (val: string) => void
    placeholder: string
    listRef: React.RefObject<HTMLDivElement | null>
    onDragOver: (e: React.DragEvent) => void
    onDrop: (e: React.DragEvent) => void
    isDropTarget: boolean
    items: T[]
    keyExtractor: (item: T) => string
    onDragStart: (e: React.DragEvent, item: T) => void
    onDragEnd: (e: React.DragEvent) => void
    onTouchStart: (e: React.TouchEvent, item: T) => void
    onClick: (item: T) => void
    renderItem: (item: T) => React.ReactNode
    emptyState?: React.ReactNode
    emptyText: string
    draggingItem: T | null
    type: 'available' | 'selected'
}

function TransferListColumn<T>({
    title, count, search, onSearchChange, placeholder, listRef,
    onDragOver, onDrop, isDropTarget, items, keyExtractor,
    onDragStart, onDragEnd, onTouchStart, onClick, renderItem,
    emptyState, emptyText, draggingItem, type
}: ColumnProps<T>) {
    const isSelected = type === 'selected'
    return (
        <div className="flex flex-col space-y-3 min-h-0">
            <div className="flex items-center justify-between px-1 flex-shrink-0">
                <h3 className={cn(
                    "text-xs uppercase font-bold tracking-widest flex items-center gap-2",
                    isSelected ? "text-blue-500/70" : "text-gray-500"
                )}>
                    {title} <span className={cn(
                        "px-2 py-0.5 rounded-full text-[10px]",
                        isSelected ? "bg-blue-500/10 text-blue-400 border border-blue-500/20" : "bg-white dark:bg-white/5"
                    )}>{count}</span>
                </h3>
            </div>

            <div className="relative group flex-shrink-0">
                <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-600 group-focus-within:text-blue-500 transition-colors" />
                <Input
                    value={search}
                    onChange={(e) => onSearchChange(e.target.value)}
                    placeholder={placeholder}
                    className="pl-10 bg-white dark:bg-white/5 border-gray-200 dark:border-slate-700 h-10 rounded-xl focus:border-blue-500/50"
                />
                {Boolean(search) && (
                    <button onClick={() => onSearchChange('')} className="absolute right-3 top-1/2 -translate-y-1/2 text-gray-600 hover:text-white">
                        <X className="w-3.5 h-3.5" />
                    </button>
                )}
            </div>

            <div
                ref={listRef}
                onDragOver={onDragOver}
                onDrop={onDrop}
                className={cn(
                    "flex flex-col gap-2 p-2 rounded-2xl border flex-1 min-h-[200px] overflow-y-auto custom-scrollbar transition-all",
                    isSelected ? "border-blue-500/10 bg-blue-500/[0.02] hover:border-blue-500/20 ring-1 ring-blue-500/0 hover:ring-blue-500/5" : "border-gray-200 dark:border-slate-700 bg-white/[0.02] hover:border-gray-200 dark:border-slate-700 group/list",
                    isDropTarget && "border-blue-500/50 bg-blue-500/10 ring-2 ring-blue-500/30"
                )}
            >
                {items.length === 0 ? (
                    emptyState ?? (
                        <div className={cn("flex flex-col items-center justify-center h-full space-y-2", isSelected ? "text-blue-500/20" : "text-gray-600")}>
                            <Shield className="w-8 h-8 opacity-10" />
                            <p className="text-sm italic">{search ? 'No matches found' : emptyText}</p>
                        </div>
                    )
                ) : (
                    items.map(item => (
                        <div
                            key={keyExtractor(item)}
                            draggable
                            onDragStart={(e) => onDragStart(e, item)}
                            onDragEnd={onDragEnd}
                            onTouchStart={(e) => onTouchStart(e, item)}
                            onClick={() => onClick(item)}
                            className={cn(
                                "cursor-grab active:cursor-grabbing transition-all select-none touch-manipulation",
                                draggingItem !== null && keyExtractor(draggingItem) === keyExtractor(item) && "opacity-60"
                            )}
                        >
                            {renderItem(item)}
                        </div>
                    ))
                )}
            </div>
        </div>
    )
}

interface ControlsProps {
    onMoveRight: () => void;
    onMoveLeft: () => void;
    canMoveRight: boolean;
    canMoveLeft: boolean;
}

const TransferListControls: React.FC<ControlsProps> = ({
    onMoveRight, onMoveLeft, canMoveRight, canMoveLeft
}) => (
    <div className="flex flex-row md:flex-col items-center justify-center gap-2 py-2 md:py-0">
        <Button
            variant="secondary" size="icon" disabled={!canMoveRight}
            onClick={onMoveRight}
            className={cn(
                "rounded-full transition-all shadow-sm w-10 h-10 md:w-12 md:h-12",
                canMoveRight ? "bg-white dark:bg-gray-700 text-green-600 hover:bg-green-50 hover:text-green-700 border-green-200" : "opacity-50"
            )}
        >
            <ChevronRight className="h-5 w-5 md:h-6 md:w-6 transform rotate-90 md:rotate-0" />
        </Button>
        <Button
            variant="secondary" size="icon" disabled={!canMoveLeft}
            onClick={onMoveLeft}
            className={cn(
                "rounded-full transition-all shadow-sm w-10 h-10 md:w-12 md:h-12",
                canMoveLeft ? "bg-white dark:bg-gray-700 text-red-600 hover:bg-red-50 hover:text-red-700 border-red-200" : "opacity-50"
            )}
        >
            <ChevronLeft className="h-5 w-5 md:h-6 md:w-6 transform rotate-90 md:rotate-0" />
        </Button>
    </div>
);

export function TransferList<T>(props: TransferListProps<T>) {
    const {
        available: allAvailable, selected, onChange, renderItem, keyExtractor,
        filterItem, availableTitle = 'Available', selectedTitle = 'Selected',
        availableSearchPlaceholder = 'Search available...', selectedSearchPlaceholder = 'Search selected...',
        className, emptyStateAvailable, emptyStateSelected, showSelection = false
    } = props

    const state = useTransferListState({ available: allAvailable, selected, onChange, keyExtractor, filterItem })
    const {
        leftSearch, setLeftSearch, rightSearch, setRightSearch,
        draggingItem, setDraggingItem, selectedAvailable,
        selectedSelected, touchDrag,
        dropTarget,
        availableListRef, selectedListRef, filteredAvailable, filteredSelected,
        moveRight, moveLeft, bulkMoveRight, bulkMoveLeft
    } = state

    const handlers = useTransferListHandlers({
        state, keyExtractor, renderItem, showSelection
    })

    const {
        handleTouchStart, handleTouchMove, handleTouchEnd,
        handleDragStart, handleDrop, renderItemWithSelection
    } = handlers

    return (
        <div
            className={cn("grid gap-4 lg:gap-6", showSelection ? "grid-cols-1 md:grid-cols-[1fr_auto_1fr]" : "grid-cols-1 sm:grid-cols-2", className)}
            onTouchMove={handleTouchMove} onTouchEnd={handleTouchEnd} onTouchCancel={handleTouchEnd}
        >
            <TransferListColumn
                title={availableTitle} count={allAvailable.length}
                search={leftSearch} onSearchChange={setLeftSearch}
                placeholder={availableSearchPlaceholder} listRef={availableListRef}
                onDragOver={(e) => { e.preventDefault(); e.dataTransfer.dropEffect = 'move'; }}
                onDrop={(e) => handleDrop(e, 'available')}
                isDropTarget={dropTarget === 'available' && touchDrag?.source === 'selected'}
                items={filteredAvailable} keyExtractor={keyExtractor}
                onDragStart={(e, i) => handleDragStart(e, i, 'available')}
                onDragEnd={(e) => { (e.target as HTMLElement).classList.remove('opacity-40'); setDraggingItem(null); }}
                onTouchStart={(e, i) => handleTouchStart(e, i, 'available')}
                onClick={(item) => touchDrag === null && draggingItem === null && moveRight(item)}
                renderItem={(item) => renderItemWithSelection(item, 'available')}
                emptyState={emptyStateAvailable} emptyText="No items available"
                draggingItem={draggingItem} type="available"
            />

            {showSelection && (
                <TransferListControls
                    onMoveRight={bulkMoveRight}
                    onMoveLeft={bulkMoveLeft}
                    canMoveRight={selectedAvailable.size > 0}
                    canMoveLeft={selectedSelected.size > 0}
                />
            )}

            <TransferListColumn
                title={selectedTitle} count={selected.length}
                search={rightSearch} onSearchChange={setRightSearch}
                placeholder={selectedSearchPlaceholder} listRef={selectedListRef}
                onDragOver={(e) => { e.preventDefault(); e.dataTransfer.dropEffect = 'move'; }}
                onDrop={(e) => handleDrop(e, 'selected')}
                isDropTarget={dropTarget === 'selected' && touchDrag?.source === 'available'}
                items={filteredSelected} keyExtractor={keyExtractor}
                onDragStart={(e, i) => handleDragStart(e, i, 'selected')}
                onDragEnd={(e) => { (e.target as HTMLElement).classList.remove('opacity-40'); setDraggingItem(null); }}
                onTouchStart={(e, i) => handleTouchStart(e, i, 'selected')}
                onClick={(item) => touchDrag === null && draggingItem === null && moveLeft(item)}
                renderItem={(item) => renderItemWithSelection(item, 'selected')}
                emptyState={emptyStateSelected} emptyText="Drag items here"
                draggingItem={draggingItem} type="selected"
            />

            <style dangerouslySetInnerHTML={{
                __html: `
                .custom-scrollbar::-webkit-scrollbar { width: 8px; height: 8px; }
                .custom-scrollbar::-webkit-scrollbar-thumb { background: rgba(255, 255, 255, 0.1); border-radius: 20px; border: 2px solid transparent; background-clip: content-box; }
                .touch-manipulation { touch-action: manipulation; }
            ` }} />

            {/* Global touch overlay */}
            {/* Global touch overlay */}
            <TransferListOverlay touchDrag={touchDrag} onTouchMove={handleTouchMove} onTouchEnd={handleTouchEnd} />
        </div>
    )
}

function TransferListOverlay<T>({ touchDrag, onTouchMove, onTouchEnd }: {
    touchDrag: TouchDragState<T> | null;
    onTouchMove: (e: React.TouchEvent) => void;
    onTouchEnd: () => void;
}) {
    if (!touchDrag) { return null; }
    return (
        <div
            className="fixed inset-0 z-40 touch-none"
            onTouchMove={onTouchMove}
            onTouchEnd={onTouchEnd}
        />
    )
}

interface UseTransferListHandlersProps<T> {
    state: {
        draggingItem: T | null;
        setDraggingItem: React.Dispatch<React.SetStateAction<T | null>>;
        selectedAvailable: Set<string>;
        setSelectedAvailable: React.Dispatch<React.SetStateAction<Set<string>>>;
        selectedSelected: Set<string>;
        setSelectedSelected: React.Dispatch<React.SetStateAction<Set<string>>>;
        touchDrag: TouchDragState<T> | null;
        setTouchDrag: React.Dispatch<React.SetStateAction<TouchDragState<T> | null>>;
        dropTarget: 'available' | 'selected' | null;
        setDropTarget: React.Dispatch<React.SetStateAction<'available' | 'selected' | null>>;
        longPressTimer: React.MutableRefObject<ReturnType<typeof setTimeout> | null>;
        availableListRef: React.RefObject<HTMLDivElement | null>;
        selectedListRef: React.RefObject<HTMLDivElement | null>;
        moveRight: (item: T) => void;
        moveLeft: (item: T) => void;
    };
    keyExtractor: (item: T) => string;
    renderItem: (item: T, type: 'available' | 'selected') => React.ReactNode;
    showSelection: boolean;
}

function useTransferListHandlers<T>({
    state, keyExtractor, renderItem, showSelection
}: UseTransferListHandlersProps<T>) {
    const {
        draggingItem, setDraggingItem, selectedAvailable, setSelectedAvailable,
        selectedSelected, setSelectedSelected, touchDrag, setTouchDrag,
        dropTarget, setDropTarget, longPressTimer,
        availableListRef, selectedListRef,
        moveRight, moveLeft
    } = state

    // Selection toggle handlers
    const toggleSelection = useCallback((item: T, type: 'available' | 'selected') => {
        const key = keyExtractor(item)
        const setter = type === 'available' ? setSelectedAvailable : setSelectedSelected
        setter((prev: Set<string>) => {
            const next = new Set(prev)
            if (next.has(key)) { next.delete(key); } else { next.add(key); }
            return next
        })
    }, [keyExtractor, setSelectedAvailable, setSelectedSelected])

    // Touch handlers
    const handleTouchStart = useCallback((e: React.TouchEvent, item: T, source: 'available' | 'selected') => {
        if (e.touches.length === 0) { return; }
        const touch = e.touches[0];
        const element = e.currentTarget as HTMLElement
        longPressTimer.current = setTimeout(() => {
            if (typeof navigator.vibrate === 'function') { navigator.vibrate(50); }
            setTouchDrag({ item, source, startY: touch.clientY, currentY: touch.clientY, element })
            setDraggingItem(item)
            element.classList.add('scale-105', 'shadow-lg', 'z-50')
        }, 200)
    }, [longPressTimer, setTouchDrag, setDraggingItem])

    const handleTouchMove = useCallback((e: React.TouchEvent) => {
        if (!touchDrag || e.touches.length === 0) { return }
        const touch = e.touches[0]
        e.preventDefault()
        setTouchDrag((prev) => {
            if (!prev) { return null }
            return { ...prev, currentY: touch.clientY }
        })

        const x = touch.clientX
        const y = touch.clientY

        if (isPointInRect({ x, y }, availableListRef.current?.getBoundingClientRect())) {
            setDropTarget('available')
        } else if (isPointInRect({ x, y }, selectedListRef.current?.getBoundingClientRect())) {
            setDropTarget('selected')
        } else {
            setDropTarget(null)
        }
    }, [touchDrag, availableListRef, selectedListRef, setTouchDrag, setDropTarget])

    const handleTouchEnd = useCallback(() => {
        if (longPressTimer.current) { clearTimeout(longPressTimer.current); longPressTimer.current = null; }
        if (!touchDrag) { return }
        if (touchDrag.element) { touchDrag.element.classList.remove('scale-105', 'shadow-lg', 'z-50'); }
        if (dropTarget !== null && dropTarget !== touchDrag.source) {
            if (dropTarget === 'selected') { moveRight(touchDrag.item); } else { moveLeft(touchDrag.item); }
        }
        setTouchDrag(null); setDraggingItem(null); setDropTarget(null)
    }, [longPressTimer, touchDrag, dropTarget, moveRight, moveLeft, setTouchDrag, setDraggingItem, setDropTarget])

    // DnD Handlers
    const handleDragStart = (e: React.DragEvent, item: T, source: 'available' | 'selected') => {
        setDraggingItem(item)
        e.dataTransfer.setData('key', keyExtractor(item))
        e.dataTransfer.setData('source', source)
        e.dataTransfer.effectAllowed = 'move'
        const target = e.target as HTMLElement
        target.classList.add('opacity-40')
    }

    const handleDrop = (e: React.DragEvent, target: 'available' | 'selected') => {
        e.preventDefault()
        const source = e.dataTransfer.getData('source')
        if (draggingItem === null || source === target) { return }
        if (target === 'selected') { moveRight(draggingItem); } else { moveLeft(draggingItem); }
    }

    const renderItemWithSelection = useCallback((item: T, type: 'available' | 'selected') => {
        if (!showSelection) { return renderItem(item, type); }
        const key = keyExtractor(item)
        const isChecked = type === 'available' ? selectedAvailable.has(key) : selectedSelected.has(key)
        return (
            <div className="flex items-center gap-2">
                <button
                    onClick={(e) => { e.stopPropagation(); toggleSelection(item, type); }}
                    className="p-1.5 hover:bg-gray-100 dark:hover:bg-gray-700 rounded flex-shrink-0"
                >
                    {isChecked ? <CheckSquare className="h-4 w-4 text-blue-600" /> : <Square className="h-4 w-4 text-gray-400" />}
                </button>
                <div className="flex-1 min-w-0">{renderItem(item, type)}</div>
            </div>
        )
    }, [showSelection, renderItem, keyExtractor, selectedAvailable, selectedSelected, toggleSelection])

    return {
        handleTouchStart, handleTouchMove, handleTouchEnd,
        handleDragStart, handleDrop, toggleSelection, renderItemWithSelection
    }
}

function isPointInRect(point: { x: number, y: number }, rect?: DOMRect) {
    if (!rect) { return false }
    return point.x >= rect.left && point.x <= rect.right && point.y >= rect.top && point.y <= rect.bottom
}
