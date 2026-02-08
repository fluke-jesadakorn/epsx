'use client'

import { useCallback, useMemo, useRef, useState } from 'react'
import type { TransferListProps } from './transfer-list'

interface TouchDragState<T> {
    item: T
    source: 'available' | 'selected'
    startY: number
    currentY: number
    element: HTMLElement | null
}

export function useTransferListState<T>({
    available: allAvailable,
    selected,
    onChange,
    keyExtractor,
    filterItem,
}: Pick<TransferListProps<T>, 'available' | 'selected' | 'onChange' | 'keyExtractor' | 'filterItem'>) {
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

    const activeFilter = filterItem ?? defaultFilter

    const filteredAvailable = useMemo(() => {
        const filtered = allAvailable.filter(item => activeFilter(item, leftSearch))
        const seen = new Set<string>()
        return filtered.filter(item => {
            const key = keyExtractor(item)
            if (seen.has(key)) { return false }
            seen.add(key)
            return true
        })
    }, [allAvailable, leftSearch, activeFilter, keyExtractor])

    const filteredSelected = useMemo(() => {
        const filtered = selected.filter(item => activeFilter(item, rightSearch))
        const seen = new Set<string>()
        return filtered.filter(item => {
            const key = keyExtractor(item)
            if (seen.has(key)) { return false }
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
        setSelectedSelected(prev => {
            const next = new Set(prev)
            next.delete(key)
            return next
        })
    }, [selected, onChange, keyExtractor])

    // Bulk move selected items to the right
    const bulkMoveRight = useCallback(() => {
        if (selectedAvailable.size === 0) { return }
        const itemsToMove = allAvailable.filter(item => selectedAvailable.has(keyExtractor(item)))
        const existingKeys = new Set(selected.map(keyExtractor))
        const newItems = itemsToMove.filter(item => !existingKeys.has(keyExtractor(item)))
        onChange([...selected, ...newItems])
        setSelectedAvailable(new Set())
    }, [allAvailable, selected, selectedAvailable, keyExtractor, onChange])

    // Bulk move selected items to the left
    const bulkMoveLeft = useCallback(() => {
        if (selectedSelected.size === 0) { return }
        onChange(selected.filter(item => !selectedSelected.has(keyExtractor(item))))
        setSelectedSelected(new Set())
    }, [selected, selectedSelected, keyExtractor, onChange])

    return {
        leftSearch, setLeftSearch,
        rightSearch, setRightSearch,
        draggingItem, setDraggingItem,
        selectedAvailable, setSelectedAvailable,
        selectedSelected, setSelectedSelected,
        touchDrag, setTouchDrag,
        dropTarget, setDropTarget,
        longPressTimer,
        availableListRef, selectedListRef,
        filteredAvailable, filteredSelected,
        moveRight, moveLeft,
        bulkMoveRight, bulkMoveLeft
    }
}
