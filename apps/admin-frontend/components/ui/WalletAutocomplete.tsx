'use client'

import { groupMgmt } from '@/lib/api/group-management-client'
import { useQuery } from '@tanstack/react-query'
import { Loader2, Wallet, X } from 'lucide-react'
import { useCallback, useEffect, useRef, useState } from 'react'
import { Input } from './input'

interface WalletSuggestion {
    wallet_address: string
    user_id?: string
    tier?: string
    permissions?: string[]
    groups?: string[]
}

interface WalletAutocompleteProps {
    value: string
    onChange: (value: string) => void
    onSelect?: (wallet: WalletSuggestion) => void
    placeholder?: string
    className?: string
    disabled?: boolean
    excludeGroupId?: string
}

// Debounce hook
function useDebounce<T>(value: T, delay: number): T {
    const [debouncedValue, setDebouncedValue] = useState<T>(value)

    useEffect(() => {
        const handler = setTimeout(() => {
            setDebouncedValue(value)
        }, delay)

        return () => {
            clearTimeout(handler)
        }
    }, [value, delay])

    return debouncedValue
}

export function WalletAutocomplete({
    value,
    onChange,
    onSelect,
    placeholder = 'Enter wallet address (0x...)',
    className = '',
    disabled = false,
    excludeGroupId,
}: WalletAutocompleteProps) {
    const [isOpen, setIsOpen] = useState(false)
    const [inputValue, setInputValue] = useState(value)
    const containerRef = useRef<HTMLDivElement>(null)
    const inputRef = useRef<HTMLInputElement>(null)

    // Debounce search query
    const debouncedQuery = useDebounce(inputValue, 300)
    const shouldSearch = debouncedQuery.length >= 2

    // Fetch wallet suggestions
    const { data: suggestions = [], isLoading } = useQuery({
        queryKey: ['wallet-search', debouncedQuery, excludeGroupId],
        queryFn: async () => {
            if (!shouldSearch) return []

            console.log('[WalletAutocomplete] Searching with:', { query: debouncedQuery, excludeGroupId })

            try {
                // Use the group management client to search users
                const results = await groupMgmt.searchUsers(debouncedQuery, 10, excludeGroupId)

                console.log('[WalletAutocomplete] Search results:', results)

                // Transform results to wallet suggestions
                if (Array.isArray(results)) {
                    return results.map((user: any) => ({
                        wallet_address: user.wallet_address || user.user_id || user.id,
                        user_id: user.user_id || user.id,
                        tier: user.tier,
                        permissions: user.permissions,
                        groups: user.groups,
                    })).filter((w: WalletSuggestion) => w.wallet_address)
                }

                return []
            } catch (error) {
                // Silently fail - search is optional feature
                console.warn('Wallet search not available:', error)
                return []
            }
        },
        enabled: shouldSearch,
        staleTime: 30000, // Cache for 30 seconds
        retry: false, // Don't retry on 404
    })

    // Sync external value changes
    useEffect(() => {
        setInputValue(value)
    }, [value])

    // Handle click outside to close dropdown
    useEffect(() => {
        function handleClickOutside(event: MouseEvent) {
            if (containerRef.current && !containerRef.current.contains(event.target as Node)) {
                setIsOpen(false)
            }
        }

        document.addEventListener('mousedown', handleClickOutside)
        return () => document.removeEventListener('mousedown', handleClickOutside)
    }, [])

    const handleInputChange = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
        const newValue = e.target.value
        setInputValue(newValue)
        onChange(newValue)
        setIsOpen(true)
    }, [onChange])

    const handleSelectWallet = useCallback((wallet: WalletSuggestion) => {
        setInputValue(wallet.wallet_address)
        onChange(wallet.wallet_address)
        onSelect?.(wallet)
        setIsOpen(false)
    }, [onChange, onSelect])

    const handleClear = useCallback(() => {
        setInputValue('')
        onChange('')
        inputRef.current?.focus()
    }, [onChange])

    const handleFocus = useCallback(() => {
        if (inputValue.length >= 2) {
            setIsOpen(true)
        }
    }, [inputValue])

    // Format wallet address for display
    const formatAddress = (address: string) => {
        if (address.length > 16) {
            return `${address.slice(0, 8)}...${address.slice(-6)}`
        }
        return address
    }

    return (
        <div ref={containerRef} className={`relative ${className}`}>
            <div className="relative">
                <Input
                    ref={inputRef}
                    value={inputValue}
                    onChange={handleInputChange}
                    onFocus={handleFocus}
                    placeholder={placeholder}
                    disabled={disabled}
                    className="pr-10"
                />

                {/* Loading or Clear button */}
                <div className="absolute right-3 top-1/2 -translate-y-1/2 flex items-center gap-1">
                    {isLoading && shouldSearch && (
                        <Loader2 className="h-4 w-4 animate-spin text-gray-400" />
                    )}
                    {inputValue && !disabled && (
                        <button
                            type="button"
                            onClick={handleClear}
                            className="p-0.5 rounded hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors"
                        >
                            <X className="h-3.5 w-3.5 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300" />
                        </button>
                    )}
                </div>
            </div>

            {/* Dropdown suggestions */}
            {isOpen && shouldSearch && (
                <div className="absolute z-50 w-full mt-1 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg shadow-lg max-h-60 overflow-auto">
                    {isLoading ? (
                        <div className="p-3 flex items-center justify-center gap-2 text-sm text-gray-500">
                            <Loader2 className="h-4 w-4 animate-spin" />
                            Searching...
                        </div>
                    ) : suggestions.length > 0 ? (
                        <ul className="py-1">
                            {suggestions.map((wallet, index) => (
                                <li key={wallet.wallet_address || index}>
                                    <button
                                        type="button"
                                        onClick={() => handleSelectWallet(wallet)}
                                        className="w-full px-3 py-2 text-left hover:bg-gray-100 dark:hover:bg-gray-700 transition-colors flex items-center gap-3"
                                    >
                                        <div className="flex-shrink-0 w-8 h-8 bg-gradient-to-br from-blue-500 to-purple-600 rounded-full flex items-center justify-center">
                                            <Wallet className="h-4 w-4 text-white" />
                                        </div>
                                        <div className="flex-1 min-w-0">
                                            <div className="font-mono text-sm font-medium truncate text-gray-900 dark:text-gray-100">
                                                {formatAddress(wallet.wallet_address)}
                                            </div>
                                            <div className="flex items-center gap-2 text-xs text-gray-500 dark:text-gray-400">
                                                {wallet.tier && (
                                                    <span className="px-1.5 py-0.5 bg-blue-100 dark:bg-blue-900/30 text-blue-700 dark:text-blue-300 rounded">
                                                        {wallet.tier}
                                                    </span>
                                                )}
                                                {wallet.groups && wallet.groups.length > 0 && (
                                                    <span className="truncate">
                                                        {wallet.groups.length} group{wallet.groups.length !== 1 ? 's' : ''}
                                                    </span>
                                                )}
                                            </div>
                                        </div>
                                    </button>
                                </li>
                            ))}
                        </ul>
                    ) : (
                        <div className="p-3 text-sm text-gray-500 dark:text-gray-400 text-center">
                            No wallets found matching "{debouncedQuery}"
                        </div>
                    )}
                </div>
            )}
        </div>
    )
}
