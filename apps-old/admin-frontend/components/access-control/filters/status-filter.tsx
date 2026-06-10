import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from '@/components/ui/select';

import { STATUS_OPTIONS } from './constants';

interface StatusFilterProps {
    value: string;
    onChange: (value: 'all' | 'active' | 'inactive') => void;
}

export function StatusFilter({ value, onChange }: StatusFilterProps) {
    return (
        <Select
            value={value}
            onValueChange={(v) => onChange(v as 'all' | 'active' | 'inactive')}
        >
            <SelectTrigger className="w-32 h-9">
                <SelectValue placeholder="Status" />
            </SelectTrigger>
            <SelectContent>
                {STATUS_OPTIONS.map((status) => (
                    <SelectItem key={status.value} value={status.value}>
                        {status.label}
                    </SelectItem>
                ))}
            </SelectContent>
        </Select>
    );
}
