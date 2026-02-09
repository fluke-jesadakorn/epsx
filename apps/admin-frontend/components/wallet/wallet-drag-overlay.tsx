import { DragOverlay } from '@dnd-kit/core';
import { Key, Package } from 'lucide-react';

import type { AccessItem } from '@/hooks/use-wallet-access';
import { cn } from '@/lib/utils';

interface WalletDragOverlayProps {
    activeDragItem: AccessItem | null;
}

export function WalletDragOverlay({ activeDragItem }: WalletDragOverlayProps) {
    return (
        <DragOverlay>
            {activeDragItem !== null && (
                <div className={cn(
                    "flex items-center gap-2 px-3 py-2 bg-white dark:bg-gray-800 rounded-lg shadow-xl border opacity-90 scale-105 pointer-events-none",
                    activeDragItem.type === 'permission' ? "border-blue-500" : "border-purple-500"
                )}>
                    {activeDragItem.type === 'permission' ? (
                        <Key className="h-4 w-4 text-purple-500" />
                    ) : (
                        <Package className="h-4 w-4 text-purple-500" />
                    )}
                    <span className="font-medium text-sm">{activeDragItem.name}</span>
                </div>
            )}
        </DragOverlay>
    );
}
