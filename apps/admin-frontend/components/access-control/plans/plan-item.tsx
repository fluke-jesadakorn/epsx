import { useSortable } from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';
import { GripVertical } from 'lucide-react';

import { Badge } from '@/components/ui/badge';
import { Switch } from '@/components/ui/switch';
import { type PermissionPlan } from '@/lib/api/plan-management-client';
import { cn } from '@/lib/utils';

export interface PlanItemProps {
    plan: PermissionPlan;
    index: number;
    selectedPlanId?: string;
    onSelect?: (plan: PermissionPlan) => void;
    isFreePlan: boolean;
    onQuickToggle?: (e: React.MouseEvent, plan: PermissionPlan) => void;
    isDragging?: boolean;
    disabled?: boolean;
    // Props passed from useSortable listeners/attributes
    dragHandleProps?: Record<string, unknown>;
    // Style override
    style?: React.CSSProperties;
    innerRef?: (node: HTMLElement | null) => void;
}

export function PlanItem({
    plan,
    index,
    selectedPlanId,
    onSelect,
    isFreePlan,
    onQuickToggle,
    isDragging,
    disabled,
    dragHandleProps, // { ...listeners, ...attributes }
    style,
    innerRef,
}: PlanItemProps) {
    const isPlanActive = plan.is_active !== false;

    return (
        <div
            ref={innerRef}
            style={style}
            onClick={() => onSelect?.(plan)}
            className={cn(
                'p-4 cursor-pointer hover:bg-white/5 transition-colors border-l-4 group relative bg-transparent', // Explicit bg-transparent for base
                selectedPlanId === plan.id
                    ? 'bg-cyan-500/10 border-l-[#1fc7d4]'
                    : 'border-l-transparent',
                isDragging === true ? 'opacity-40' : ''
            )}
        >
            {/* Drag Handle - Absolutely positioned */}
            <div
                {...(dragHandleProps ?? {})}
                className={cn(
                    'absolute left-1 top-1/2 -translate-y-1/2 p-1 text-slate-500 hover:text-white cursor-grab active:cursor-grabbing opacity-0 group-hover:opacity-100 transition-opacity',
                    disabled === true ? 'hidden' : ''
                )}
                onClick={(e) => e.stopPropagation()}
            >
                <GripVertical className="h-4 w-4" />
            </div>

            {/* Content with padding for handle */}
            <div className={cn('pl-4 transition-all duration-200')}>
                <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2">
                        {/* ORDER INDICATOR */}
                        <div className="h-5 w-5 rounded bg-white/5 flex items-center justify-center text-[10px] font-mono text-muted-foreground">
                            {index + 1}
                        </div>
                        <div>
                            <h4 className="font-bold text-sm text-foreground">{plan.name}</h4>
                            <p className="text-xs text-muted-foreground line-clamp-1">
                                {plan.description}
                            </p>
                        </div>
                    </div>
                    <Badge variant="secondary" className="text-[10px] h-5 bg-white/5">
                        {plan.permissions?.length ?? 0}
                    </Badge>
                </div>
                <div className="flex items-center justify-between mt-3 pt-3 border-t border-white/5">
                    <span
                        className={cn(
                            'text-[10px] font-medium uppercase tracking-wider',
                            isPlanActive ? 'text-emerald-500' : 'text-slate-500'
                        )}
                    >
                        {isPlanActive ? 'Active' : 'Inactive'}
                    </span>
                    <Switch
                        checked={isPlanActive}
                        onCheckedChange={() => { }}
                        onClick={(e) => onQuickToggle?.(e, plan)}
                        disabled={isFreePlan}
                        className="scale-75 origin-right"
                    />
                </div>
            </div>
        </div>
    );
}

export interface SortablePlanItemProps {
    plan: PermissionPlan;
    index: number;
    selectedPlanId?: string;
    onSelect: (plan: PermissionPlan) => void;
    isFreePlan: boolean;
    onQuickToggle: (e: React.MouseEvent, plan: PermissionPlan) => void;
    disabled?: boolean;
}

export function SortablePlanItem({
    plan,
    index,
    selectedPlanId,
    onSelect,
    isFreePlan,
    onQuickToggle,
    disabled,
}: SortablePlanItemProps) {
    const {
        attributes,
        listeners,
        setNodeRef,
        transform,
        transition,
        isDragging,
    } = useSortable({ id: plan.id, disabled });

    const style = {
         
        transform: transform ? CSS.Transform.toString(transform) : undefined,
        transition,
        touchAction: 'none' as const,
    };

    return (
        <PlanItem
            plan={plan}
            index={index}
            selectedPlanId={selectedPlanId}
            onSelect={onSelect}
            isFreePlan={isFreePlan}
            onQuickToggle={onQuickToggle}
            isDragging={isDragging}
            disabled={disabled}
            dragHandleProps={{ ...attributes, ...listeners } as Record<string, unknown>}
            style={style}
            innerRef={setNodeRef}
        />
    );
}
