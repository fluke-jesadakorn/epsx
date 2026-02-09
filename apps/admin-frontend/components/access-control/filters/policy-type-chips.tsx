import { cn } from '@/lib/utils';

import { type PolicyType, POLICY_TYPE_CONFIG } from '../types';

interface PolicyTypeChipsProps {
    activeTypes: PolicyType[] | 'all';
    onChange: (types: PolicyType[] | 'all') => void;
    className?: string;
}

export function PolicyTypeChips({
    activeTypes,
    onChange,
    className,
}: PolicyTypeChipsProps) {
    const isAllActive = activeTypes === 'all';
    const activeTypesArray = isAllActive ? [] : activeTypes;

    const toggleType = (type: PolicyType) => {
        if (isAllActive) {
            // Switch from all to just this type
            onChange([type]);
        } else if (activeTypesArray.includes(type)) {
            // Remove this type
            const newTypes = activeTypesArray.filter((t) => t !== type);
            onChange(newTypes.length === 0 ? 'all' : newTypes);
        } else {
            // Add this type
            onChange([...activeTypesArray, type]);
        }
    };

    return (
        <div className={cn('flex flex-wrap gap-2', className)}>
            <button
                onClick={() => onChange('all')}
                className={cn(
                    'px-3 py-1.5 rounded-full text-sm font-medium transition-all duration-200',
                    isAllActive
                        ? 'bg-primary text-primary-foreground shadow-md'
                        : 'bg-muted text-muted-foreground hover:bg-muted/80'
                )}
            >
                All
            </button>

            {Object.entries(POLICY_TYPE_CONFIG).map(([type, config]) => {
                const isActive =
                    !isAllActive && activeTypesArray.includes(type as PolicyType);
                return (
                    <button
                        key={type}
                        onClick={() => toggleType(type as PolicyType)}
                        className={cn(
                            'px-3 py-1.5 rounded-full text-sm font-medium transition-all duration-200 flex items-center gap-1.5',
                            isActive
                                ? 'bg-primary text-primary-foreground shadow-md'
                                : 'bg-muted text-muted-foreground hover:bg-muted/80'
                        )}
                    >
                        <span>{config.icon}</span>
                        <span>{config.label}</span>
                    </button>
                );
            })}
        </div>
    );
}
