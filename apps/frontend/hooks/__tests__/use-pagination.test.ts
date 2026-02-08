// Frontend Hooks Tests - Application Layer (Business Workflows)
// Tests for custom React hooks and state management
// Clean Architecture: Application Layer - Uses mocked dependencies

import { renderHook, act } from '@testing-library/react'
import { usePagination } from '../usepagination'

describe('usePagination Hook', () => {
  describe('Initialization', () => {
    test('initializes with correct default values', () => {
      const { result } = renderHook(() => usePagination({ totalItems: 100 }))

      expect(result.current.currentPage).toBe(1)
      expect(result.current.pageSize).toBe(10)
      expect(result.current.totalPages).toBe(10)
      expect(result.current.hasNextPage).toBe(true)
      expect(result.current.hasPreviousPage).toBe(false)
      expect(result.current.startIndex).toBe(0)
      expect(result.current.endIndex).toBe(9)
    })

    test('initializes with custom initial page', () => {
      const { result } = renderHook(() => 
        usePagination({ totalItems: 100, initialPage: 3, pageSize: 20 })
      )

      expect(result.current.currentPage).toBe(3)
      expect(result.current.pageSize).toBe(20)
      expect(result.current.totalPages).toBe(5)
      expect(result.current.startIndex).toBe(40)
      expect(result.current.endIndex).toBe(59)
    })
  })

  describe('Page Calculations', () => {
    test('calculates total pages correctly with exact division', () => {
      const { result } = renderHook(() => 
        usePagination({ totalItems: 50, pageSize: 10 })
      )
      expect(result.current.totalPages).toBe(5)
    })

    test('calculates total pages correctly with remainder', () => {
      const { result } = renderHook(() => 
        usePagination({ totalItems: 25, pageSize: 10 })
      )
      expect(result.current.totalPages).toBe(3)
    })

    test('handles zero items', () => {
      const { result } = renderHook(() => 
        usePagination({ totalItems: 0 })
      )
      expect(result.current.totalPages).toBe(1)
      expect(result.current.hasNextPage).toBe(false)
      expect(result.current.hasPreviousPage).toBe(false)
    })
  })

  describe('Navigation', () => {
    test('navigates to next page correctly', () => {
      const { result } = renderHook(() => usePagination({ totalItems: 100 }))

      act(() => {
        result.current.goToNextPage()
      })

      expect(result.current.currentPage).toBe(2)
      expect(result.current.hasPreviousPage).toBe(true)
      expect(result.current.startIndex).toBe(10)
      expect(result.current.endIndex).toBe(19)
    })

    test('navigates to previous page correctly', () => {
      const { result } = renderHook(() => 
        usePagination({ totalItems: 100, initialPage: 3 })
      )

      act(() => {
        result.current.goToPreviousPage()
      })

      expect(result.current.currentPage).toBe(2)
      expect(result.current.hasPreviousPage).toBe(true)
      expect(result.current.hasNextPage).toBe(true)
    })

    test('navigates to specific page correctly', () => {
      const { result } = renderHook(() => usePagination({ totalItems: 100 }))

      act(() => {
        result.current.goToPage(5)
      })

      expect(result.current.currentPage).toBe(5)
      expect(result.current.startIndex).toBe(40)
      expect(result.current.endIndex).toBe(49)
    })
  })

  describe('Boundary Conditions', () => {
    test('does not go beyond first page', () => {
      const { result } = renderHook(() => usePagination({ totalItems: 100 }))

      // Try to go to previous page when already at first page
      act(() => {
        result.current.goToPreviousPage()
      })

      expect(result.current.currentPage).toBe(1)
      expect(result.current.hasPreviousPage).toBe(false)
    })

    test('does not go beyond last page', () => {
      const { result } = renderHook(() => 
        usePagination({ totalItems: 100, pageSize: 10 })
      )

      // Go to last page
      act(() => {
        result.current.goToPage(10)
      })
      expect(result.current.currentPage).toBe(10)

      // Try to go beyond last page
      act(() => {
        result.current.goToNextPage()
      })
      expect(result.current.currentPage).toBe(10)
      expect(result.current.hasNextPage).toBe(false)
    })

    test('handles invalid page numbers', () => {
      const { result } = renderHook(() => usePagination({ totalItems: 100 }))

      // Try negative page
      act(() => {
        result.current.goToPage(-1)
      })
      expect(result.current.currentPage).toBe(1)

      // Try page beyond maximum
      act(() => {
        result.current.goToPage(999)
      })
      expect(result.current.currentPage).toBe(10) // Should be at max page
    })
  })

  describe('Callbacks', () => {
    test('calls onChange callback when page changes', () => {
      const onChange = jest.fn()
      const { result } = renderHook(() => 
        usePagination({ totalItems: 100, onChange })
      )

      act(() => {
        result.current.goToNextPage()
      })

      expect(onChange).toHaveBeenCalledWith({
        page: 2,
        pageSize: 10,
        startIndex: 10,
        endIndex: 19,
        totalPages: 10
      })
    })

    test('does not call onChange on initialization', () => {
      const onChange = jest.fn()
      renderHook(() => 
        usePagination({ totalItems: 100, onChange })
      )

      expect(onChange).not.toHaveBeenCalled()
    })

    test('calls onChange only when page actually changes', () => {
      const onChange = jest.fn()
      const { result } = renderHook(() => 
        usePagination({ totalItems: 100, onChange })
      )

      // Try to go to previous page when already at first page
      act(() => {
        result.current.goToPreviousPage()
      })

      expect(onChange).not.toHaveBeenCalled()
    })
  })

  describe('Dynamic Updates', () => {
    test('handles totalItems changes', () => {
      const { result, rerender } = renderHook(
        ({ totalItems }) => usePagination({ totalItems }),
        { initialProps: { totalItems: 100 } }
      )

      expect(result.current.totalPages).toBe(10)

      // Change totalItems
      rerender({ totalItems: 50 })
      expect(result.current.totalPages).toBe(5)
    })

    test('adjusts current page when totalItems decreases', () => {
      const { result, rerender } = renderHook(
        ({ totalItems }) => usePagination({ totalItems }),
        { initialProps: { totalItems: 100 } }
      )

      // Go to last page
      act(() => {
        result.current.goToPage(10)
      })
      expect(result.current.currentPage).toBe(10)

      // Reduce total items so current page becomes invalid
      rerender({ totalItems: 25 }) // Now only 3 pages
      expect(result.current.currentPage).toBe(3) // Should adjust to last valid page
      expect(result.current.totalPages).toBe(3)
    })
  })
})