'use client'

import { Loader2, Plus, Shield, Trash2 } from 'lucide-react'
import React, { useCallback, useMemo, useState } from 'react'
import { toast } from 'sonner'

import { cn } from '@/lib/utils'
import { Input } from '@/shared/components'
import { TransferList } from '@/shared/components/ui/transfer-list'

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

/**
 * Validates permission string format: platform:resource:action
 * Allows wildcards (*) and multi-segment resources (e.g., epsx:analytics:view)
 * @param permission
 */
const isValidPermissionFormat = (permission: string): boolean => {
    // Basic format check: at least 2 colons, non-empty segments
    const parts = permission.split(':')
    if (parts.length < 3) { return false }
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
    const [customPermission, setCustomPermission] = useState('')
    const [isCreating, setIsCreating] = useState(false)
    const [deletingPermission, setDeletingPermission] = useState<string | null>(null)

    // Filter out selected ones from available for the left list
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

    const renderPermissionItem = useCallback((item: string, type: 'available' | 'selected') => {
        return (
            <div className={cn(
                "flex items-center justify-between p-3 rounded-xl border border-white/5 bg-white/5 hover:bg-white/10 transition-all group/item min-h-[50px]",
                type === 'selected' && "border-blue-500/10 bg-blue-500/10 hover:bg-blue-500/20"
            )}>
                <div className="flex items-center gap-3 min-w-0">
                    <Shield className={cn(
                        "w-4 h-4 flex-shrink-0",
                        type === 'selected' ? "text-blue-400" : "text-gray-500"
                    )} />
                    <span className={cn(
                        "text-sm font-medium truncate",
                        type === 'selected' ? "text-blue-100" : "text-gray-300"
                    )}>{item}</span>
                </div>

                {type === 'available' && onDeletePermission && !systemPermissions.has(item) && (
                    <button
                        type="button"
                        onClick={(e) => handleDeletePermission(item, e)}
                        disabled={deletingPermission === item}
                        className="p-1.5 rounded-lg text-gray-600 hover:text-red-400 hover:bg-red-500/10 transition-all opacity-0 group-hover/item:opacity-100 ml-2"
                        title="Delete permission"
                    >
                        {deletingPermission === item ? (
                            <Loader2 className="w-3.5 h-3.5 animate-spin" />
                        ) : (
                            <Trash2 className="w-3.5 h-3.5" />
                        )}
                    </button>
                )}
            </div>
        )
    }, [onDeletePermission, systemPermissions, deletingPermission, handleDeletePermission])

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

            {/* Transfer List */}
            <TransferList
                available={availableItems}
                selected={selected}
                onChange={onChange}
                renderItem={renderPermissionItem}
                keyExtractor={(item) => item}
                availableTitle="Available Permissions"
                selectedTitle="Authorized Permissions"
                availableSearchPlaceholder="Search available..."
                selectedSearchPlaceholder="Search authorized..."
                isLoading={isLoading}
            />
        </div>
    )
}
