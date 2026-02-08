'use client';

import { cn } from '@/lib/utils';
import { useDroppable } from '@dnd-kit/core';
import { Trash2 } from 'lucide-react';

interface TrashDropZoneProps {
    isDragging: boolean;
}

export function TrashDropZone({ isDragging }: TrashDropZoneProps) {
    const { setNodeRef, isOver } = useDroppable({
        id: 'trash',
        data: { type: 'trash' }
    });

    if (!isDragging) { return null; }

    return (
        <div
            ref={setNodeRef}
            className={cn(
                "fixed bottom-6 left-1/2 -translate-x-1/2 z-50 flex items-center justify-center gap-3 px-8 py-4 rounded-full shadow-2xl transition-all duration-200 border-2",
                isOver
                    ? "bg-red-600 border-red-400 text-white scale-110"
                    : "bg-white dark:bg-gray-800 border-red-200 dark:border-red-900 text-red-500 animate-in slide-in-from-bottom-10 fade-in"
            )}
        >
            <Trash2 className={cn("h-6 w-6", isOver && "animate-bounce")} />
            <span className={cn("font-bold text-lg", isOver ? "text-white" : "text-red-600 dark:text-red-400")}>
                {isOver ? 'Drop to Remove' : 'Drag here to Remove'}
            </span>
        </div>
    );
}
