'use client'

import { Lock, Shield } from 'lucide-react'

import { Badge } from '@/components/ui/badge'
import { TransferList } from '@/shared/components/ui/transfer-list/TransferList'

interface PermissionTransferListProps {
    available: string[]
    selected: string[]
    onChange: (selected: string[]) => void
    isLoading?: boolean
    systemPermissions?: Set<string>
    className?: string
}

export function PermissionTransferList({
    available,
    selected,
    onChange,
    isLoading,
    systemPermissions = new Set(),
    className
}: PermissionTransferListProps) {

    // Filter out selected items from available list
    const availableFiltered = available.filter(p => !selected.includes(p))

    const renderItem = (item: string, type: 'available' | 'selected') => {
        const isSystem = systemPermissions.has(item)
        // Extract parts: "platform:resource:action" 
        const parts = item.split(':')
        const action = parts.pop() || item
        const resource = parts.pop() || ''
        const platform = parts.join(':')

        return (
            <div className="flex items-center justify-between gap-2 p-2 rounded-lg bg-card border border-border/50 hover:bg-muted/50 transition-colors">
                <div className="flex items-center gap-3 overflow-hidden">
                    <div className={`p-1.5 rounded-md ${isSystem ? 'bg-destructive/10 text-destructive' : 'bg-primary/10 text-primary'}`}>
                        {isSystem ? <Shield className="w-4 h-4" /> : <Lock className="w-4 h-4" />}
                    </div>
                    <div className="flex flex-col min-w-0">
                        <span className="text-sm font-medium truncate" title={item}>
                            {action.replace(/_/g, ' ')}
                        </span>
                        <span className="text-xs text-muted-foreground truncate flex items-center gap-1">
                            {platform && <span className="opacity-70">{platform}:</span>}
                            {resource && <span>{resource}</span>}
                        </span>
                    </div>
                </div>
                {isSystem && (
                    <Badge variant="outline" className="text-[10px] px-1 h-5 border-destructive/20 text-destructive/80">
                        System
                    </Badge>
                )}
            </div>
        )
    }

    return (
        <div className={className}>
            <TransferList
                available={availableFiltered}
                selected={selected}
                onChange={onChange}
                renderItem={renderItem}
                keyExtractor={(item) => item}
                availableTitle="Available Permissions"
                selectedTitle="Granted Permissions"
                availableSearchPlaceholder="Search permissions..."
                selectedSearchPlaceholder="Search granted..."
                isLoading={isLoading}
                showSelection={true}
                emptyStateAvailable={
                    <div className="flex flex-col items-center justify-center p-8 text-muted-foreground/50">
                        <Shield className="w-12 h-12 mb-2 opacity-20" />
                        <p>No permissions found</p>
                    </div>
                }
            />
        </div>
    )
}
