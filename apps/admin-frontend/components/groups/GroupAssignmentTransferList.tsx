'use client'

import { Star, Users, X } from 'lucide-react'
import React, { useCallback, useMemo } from 'react'

import { Badge } from '@/components/ui/badge'
import { PermissionGroup } from '@/lib/api/group-management-client'
import { cn } from '@/lib/utils'
import { TransferList } from '@/shared/components/ui/transfer-list'

interface GroupAssignmentTransferListProps {
    available: PermissionGroup[]
    selected: PermissionGroup[]
    onChange: (selected: PermissionGroup[]) => void
    isLoading?: boolean
}

export const GroupAssignmentTransferList: React.FC<GroupAssignmentTransferListProps> = ({
    available: allAvailable,
    selected,
    onChange,
    isLoading = false
}) => {
    // Filter available items to exclude already selected ones
    // Note: TransferList expects 'available' to represent the left side list
    // efficiently, if the parent passes all groups, we filter here.
    // If parent passes filtered, this is redundant but harmless.
    const availableItems = useMemo(() => {
        return allAvailable.filter(item => !selected.some(s => s.id === item.id))
    }, [allAvailable, selected])

    const renderGroupItem = useCallback((group: PermissionGroup, type: 'available' | 'selected') => {
        const isSystem = group.group_type === 'system' || group.is_system_group

        return (
            <div className={cn(
                "flex items-center justify-between p-3 rounded-xl border border-white/5 bg-white/5 hover:bg-white/10 transition-all group/item min-h-[60px]",
                type === 'selected' && "border-blue-500/10 bg-blue-500/10 hover:bg-blue-500/20"
            )}>
                <div className="flex items-center gap-3 min-w-0">
                    <div className={cn(
                        "p-2 rounded-lg flex-shrink-0",
                        type === 'selected'
                            ? "bg-blue-500/20 text-blue-400"
                            : (isSystem ? "bg-purple-500/10 text-purple-400" : "bg-white/5 text-gray-400")
                    )}>
                        {isSystem ? <Star className="w-4 h-4" /> : <Users className="w-4 h-4" />}
                    </div>

                    <div className="flex flex-col min-w-0">
                        <span className={cn(
                            "text-sm font-medium truncate",
                            type === 'selected' ? "text-blue-100" : "text-gray-300"
                        )}>
                            {group.name}
                        </span>
                        <div className="flex items-center gap-2">
                            <span className="text-xs text-gray-500">
                                {group.permissions.length} permissions
                            </span>
                            {isSystem && (
                                <Badge variant="secondary" className="text-[10px] py-0 h-4 bg-purple-500/10 text-purple-400 border-none">
                                    System
                                </Badge>
                            )}
                        </div>
                    </div>
                </div>

                {type === 'selected' && (
                    <div className="text-gray-500 group-hover/item:text-red-400 transition-colors">
                        <X className="w-4 h-4" />
                    </div>
                )}
            </div>
        )
    }, [])

    return (
        <TransferList
            available={availableItems}
            selected={selected}
            onChange={onChange}
            renderItem={renderGroupItem}
            keyExtractor={(group) => group.id}
            filterItem={(group, query) =>
                group.name.toLowerCase().includes(query.toLowerCase()) ||
                (group.description && group.description.toLowerCase().includes(query.toLowerCase()))
            }
            availableTitle="Available Groups"
            selectedTitle="Authorized Access"
            availableSearchPlaceholder="Search groups..."
            selectedSearchPlaceholder="Search authorized..."
            isLoading={isLoading}
            emptyStateAvailable={
                <div className="flex flex-col items-center justify-center h-full text-gray-600 space-y-2">
                    <Users className="w-8 h-8 opacity-10" />
                    <p className="text-sm italic">All groups assigned</p>
                </div>
            }
        />
    )
}
