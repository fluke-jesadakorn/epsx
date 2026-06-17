import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from '@/components/ui/select';

import { type PolicyType } from '../types';
import { POLICY_TYPES } from './constants';

interface TypeFilterProps {
    value: PolicyType[] | 'all';
    onChange: (value: PolicyType[] | 'all') => void;
}

export function TypeFilter({ value, onChange }: TypeFilterProps) {
    // Get current type value for select
    const currentTypeValue =
        value === 'all'
            ? 'all'
            : Array.isArray(value) && value.length === 1
                ? value[0]
                : 'all';

    const handleTypeChange = (newValue: string) => {
        if (newValue === 'all') {
            onChange('all');
        } else {
            onChange([newValue as PolicyType]);
        }
    };

    return (
        <Select value={currentTypeValue} onValueChange={handleTypeChange}>
            <SelectTrigger className="w-40 h-9">
                <SelectValue placeholder="Type" />
            </SelectTrigger>
            <SelectContent>
                {POLICY_TYPES.map((type) => (
                    <SelectItem key={type.value} value={type.value}>
                        <span className="flex items-center gap-2">
                            <span>{type.icon}</span>
                            <span>{type.label}</span>
                        </span>
                    </SelectItem>
                ))}
            </SelectContent>
        </Select>
    );
}
