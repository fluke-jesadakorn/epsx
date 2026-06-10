import { Button } from '@/components/ui/button';
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from '@/components/ui/select';

import { SORT_OPTIONS } from './constants';

interface SortControlProps {
    sortBy: string;
    sortOrder: 'asc' | 'desc';
    onSortByChange: (
        value: 'name' | 'members' | 'created_at' | 'revenue' | 'type'
    ) => void;
    onSortOrderChange: (value: 'asc' | 'desc') => void;
}

export function SortControl({
    sortBy,
    sortOrder,
    onSortByChange,
    onSortOrderChange,
}: SortControlProps) {
    return (
        <>
            <Select
                value={sortBy}
                onValueChange={(v) =>
                    onSortByChange(
                        v as 'name' | 'members' | 'created_at' | 'revenue' | 'type'
                    )
                }
            >
                <SelectTrigger className="w-36 h-9">
                    <SelectValue placeholder="Sort by" />
                </SelectTrigger>
                <SelectContent>
                    {SORT_OPTIONS.map((sort) => (
                        <SelectItem key={sort.value} value={sort.value}>
                            {sort.label}
                        </SelectItem>
                    ))}
                </SelectContent>
            </Select>

            <Button
                variant="outline"
                size="sm"
                onClick={() => onSortOrderChange(sortOrder === 'asc' ? 'desc' : 'asc')}
                className="h-9 px-3"
            >
                {sortOrder === 'asc' ? '↑ Asc' : '↓ Desc'}
            </Button>
        </>
    );
}
