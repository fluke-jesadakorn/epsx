// Frontend Components Tests - Presentation Layer (UI Components)
// Tests for stock data display components
// Clean Architecture: Presentation Layer - UI rendering and user interactions

import React from 'react'
import { render, screen, fireEvent } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { StockCard } from '@/components/StockCard'

const mockStockData = {
  symbol: 'AAPL',
  companyName: 'Apple Inc.',
  currentPrice: 150.25,
  changeAmount: 2.5,
  changePercent: 1.69,
  eps: 6.05,
  peRatio: 24.83,
  marketCap: '2.5T',
  volume: 45623000
}

describe('StockCard Component', () => {
  const user = userEvent.setup()

  describe('Data Display', () => {
    test('renders all stock information correctly', () => {
      render(<StockCard stockData={mockStockData} />)

      expect(screen.getByText('AAPL')).toBeInTheDocument()
      expect(screen.getByText('Apple Inc.')).toBeInTheDocument()
      expect(screen.getByText('$150.25')).toBeInTheDocument()
      expect(screen.getByText('+$2.50')).toBeInTheDocument()
      expect(screen.getByText('+1.69%')).toBeInTheDocument()
      expect(screen.getByText('EPS: $6.05')).toBeInTheDocument()
      expect(screen.getByText('P/E: 24.83')).toBeInTheDocument()
    })

    test('displays positive change with green styling', () => {
      render(<StockCard stockData={mockStockData} />)

      const changeElement = screen.getByText('+$2.50')
      expect(changeElement).toHaveClass('text-green-600')
    })

    test('displays negative change with red styling', () => {
      const negativeStockData = {
        ...mockStockData,
        changeAmount: -3.25,
        changePercent: -2.12
      }

      render(<StockCard stockData={negativeStockData} />)

      const changeElement = screen.getByText('-$3.25')
      expect(changeElement).toHaveClass('text-red-600')
      
      const percentElement = screen.getByText('-2.12%')
      expect(percentElement).toHaveClass('text-red-600')
    })

    test('displays zero change with neutral styling', () => {
      const neutralStockData = {
        ...mockStockData,
        changeAmount: 0,
        changePercent: 0
      }

      render(<StockCard stockData={neutralStockData} />)

      const changeElement = screen.getByText('$0.00')
      expect(changeElement).toHaveClass('text-gray-600')
    })
  })

  describe('Interactive Features', () => {
    test('calls onClick handler when card is clicked', async () => {
      const mockOnClick = jest.fn()
      
      render(<StockCard stockData={mockStockData} onClick={mockOnClick} />)

      const card = screen.getByRole('button')
      await user.click(card)

      expect(mockOnClick).toHaveBeenCalledWith(mockStockData)
    })

    test('shows hover effects when interactive', async () => {
      const mockOnClick = jest.fn()
      
      render(<StockCard stockData={mockStockData} onClick={mockOnClick} />)

      const card = screen.getByRole('button')
      
      await user.hover(card)
      expect(card).toHaveClass('hover:shadow-lg')
      
      await user.unhover(card)
      expect(card).not.toHaveClass('hover:shadow-lg')
    })

    test('is not clickable when onClick is not provided', () => {
      render(<StockCard stockData={mockStockData} />)

      const card = screen.getByTestId('stock-card')
      expect(card).not.toHaveAttribute('role', 'button')
      expect(card).not.toHaveClass('cursor-pointer')
    })
  })

  describe('Loading and Error States', () => {
    test('shows skeleton loader when loading', () => {
      render(<StockCard loading={true} />)

      expect(screen.getByTestId('stock-card-skeleton')).toBeInTheDocument()
      expect(screen.queryByText('AAPL')).not.toBeInTheDocument()
    })

    test('shows error state when data is missing', () => {
      render(<StockCard error="Failed to load stock data" />)

      expect(screen.getByText(/failed to load/i)).toBeInTheDocument()
      expect(screen.getByTestId('error-icon')).toBeInTheDocument()
    })

    test('shows retry button on error', async () => {
      const mockOnRetry = jest.fn()
      
      render(<StockCard error="Network error" onRetry={mockOnRetry} />)

      const retryButton = screen.getByRole('button', { name: /retry/i })
      await user.click(retryButton)

      expect(mockOnRetry).toHaveBeenCalled()
    })
  })

  describe('Accessibility', () => {
    test('has proper ARIA labels', () => {
      render(<StockCard stockData={mockStockData} />)

      expect(screen.getByLabelText(/stock information for apple inc/i)).toBeInTheDocument()
    })

    test('supports keyboard navigation when clickable', async () => {
      const mockOnClick = jest.fn()
      
      render(<StockCard stockData={mockStockData} onClick={mockOnClick} />)

      const card = screen.getByRole('button')
      
      // Tab to focus the card
      await user.tab()
      expect(card).toHaveFocus()
      
      // Press Enter to activate
      fireEvent.keyDown(card, { key: 'Enter', code: 'Enter' })
      expect(mockOnClick).toHaveBeenCalled()
    })

    test('supports Space key activation', () => {
      const mockOnClick = jest.fn()
      
      render(<StockCard stockData={mockStockData} onClick={mockOnClick} />)

      const card = screen.getByRole('button')
      card.focus()
      
      fireEvent.keyDown(card, { key: ' ', code: 'Space' })
      expect(mockOnClick).toHaveBeenCalled()
    })
  })

  describe('Data Formatting', () => {
    test('formats large numbers with appropriate suffixes', () => {
      const largeCapStock = {
        ...mockStockData,
        marketCap: '2500000000000', // 2.5 trillion
        volume: 45623000 // 45.6 million
      }

      render(<StockCard stockData={largeCapStock} />)

      expect(screen.getByText(/2\.5T/)).toBeInTheDocument()
      expect(screen.getByText(/45\.6M/)).toBeInTheDocument()
    })

    test('handles missing optional data gracefully', () => {
      const minimalStock = {
        symbol: 'TEST',
        companyName: 'Test Company',
        currentPrice: 100.0,
        changeAmount: 0,
        changePercent: 0
      }

      render(<StockCard stockData={minimalStock} />)

      expect(screen.getByText('TEST')).toBeInTheDocument()
      expect(screen.getByText('Test Company')).toBeInTheDocument()
      expect(screen.getByText('$100.00')).toBeInTheDocument()
      
      // Optional fields should show fallback or be hidden
      expect(screen.queryByText(/EPS:/)).not.toBeInTheDocument()
      expect(screen.queryByText(/P\/E:/)).not.toBeInTheDocument()
    })

    test('handles extreme values correctly', () => {
      const extremeStock = {
        ...mockStockData,
        currentPrice: 0.001,
        changeAmount: -0.0001,
        changePercent: -9.09,
        eps: -2.5,
        peRatio: null
      }

      render(<StockCard stockData={extremeStock} />)

      expect(screen.getByText('$0.001')).toBeInTheDocument()
      expect(screen.getByText('-$0.0001')).toBeInTheDocument()
      expect(screen.getByText('-9.09%')).toBeInTheDocument()
      expect(screen.getByText('EPS: -$2.50')).toBeInTheDocument()
      expect(screen.queryByText(/P\/E:/)).not.toBeInTheDocument() // null P/E should be hidden
    })
  })
})