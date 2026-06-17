import { useSortable } from '@dnd-kit/sortable';
import { CSS } from '@dnd-kit/utilities';
import { Clock, Copy, Eye, EyeOff, GripVertical, Hash, Shield, Users } from 'lucide-react';

import { Badge } from '@/components/ui/badge';
import { type PermissionPlan, type PlanGroup } from '@/lib/api/plan-management-client';
import { cn } from '@/lib/utils';
import { formatRelativeTime } from '@/shared/utils/formatting/date';

import { categoryBadgeClass, isSystemPlan } from './types';

const GROUP_BORDER: Record<PlanGroup, string> = {
    personal: 'border-l-blue-500/60',
    enterprise: 'border-l-amber-500/60',
    api: 'border-l-emerald-500/60',
    custom: 'border-l-purple-500/50',
};

export interface PlanItemProps {
    plan: PermissionPlan;
    index: number;
    group?: PlanGroup;
    selectedPlanId?: string;
    onSelect?: (plan: PermissionPlan) => void;
    onDuplicate?: (plan: PermissionPlan) => void;
    isDragging?: boolean;
    disabled?: boolean;
    dragHandleProps?: Record<string, unknown>;
    style?: React.CSSProperties;
    innerRef?: (node: HTMLElement | null) => void;
}

function PlanItemHeader({ plan, index, isSys, active, permCount, onDuplicate }: {
    plan: PermissionPlan; index: number; isSys: boolean; active: boolean; permCount: number;
    onDuplicate?: (plan: PermissionPlan) => void;
}) {
    return (
        <div className="flex items-center justify-between gap-2">
            <div className="flex items-center gap-2 min-w-0">
                {isSys ? (
                    <div className="h-5 w-5 rounded bg-purple-500/20 flex items-center justify-center shrink-0">
                        <Shield className="h-3 w-3 text-purple-400" />
                    </div>
                ) : (
                    <div className="h-5 w-5 rounded bg-muted/30 flex items-center justify-center text-[10px] font-mono text-muted-foreground shrink-0">
                        {index + 1}
                    </div>
                )}
                <h4 className="font-bold text-sm text-foreground truncate">{plan.name}</h4>
                <Badge variant="outline" className={cn('text-[9px] px-1 py-0 shrink-0', categoryBadgeClass(plan.plan_category))}>
                    {plan.plan_category}
                </Badge>
                <span className={cn('flex items-center gap-1 text-[9px] font-medium uppercase shrink-0', active ? 'text-emerald-400' : 'text-muted-foreground')}>
                    <span className={cn('h-1.5 w-1.5 rounded-full', active ? 'bg-emerald-400' : 'bg-slate-500')} />
                    {active ? 'On' : 'Off'}
                </span>
            </div>
            <div className="flex items-center gap-1.5 shrink-0">
                <button type="button" onClick={(e) => { e.stopPropagation(); onDuplicate?.(plan); }}
                    className="opacity-0 group-hover:opacity-100 transition-opacity p-1 rounded hover:bg-muted/50 text-muted-foreground hover:text-[#1fc7d4]"
                    title="Duplicate plan">
                    <Copy className="h-3.5 w-3.5" />
                </button>
                <Badge variant="secondary" className="text-[10px] h-5 bg-muted/30">{permCount}</Badge>
            </div>
        </div>
    );
}

function PlanItemMeta({ plan, members, isPublic }: { plan: PermissionPlan; members: number; isPublic: boolean }) {
    const maxMembers = plan.max_members !== null && plan.max_members !== undefined ? `/${plan.max_members}` : '';
    return (
        <div className="flex items-center gap-2 mt-2 pt-2 border-t border-border/20 text-[10px] text-muted-foreground">
            <span className="flex items-center gap-0.5" title="Tier level"><Hash className="h-2.5 w-2.5" />{plan.tier_level}</span>
            <span className="text-white/10">|</span>
            <span className="flex items-center gap-0.5" title="Members assigned"><Users className="h-2.5 w-2.5" />{members}{maxMembers}</span>
            <span className="text-white/10">|</span>
            <span title={isPublic ? 'Public' : 'Private'}>
                {isPublic ? <Eye className="h-2.5 w-2.5 text-emerald-400/70" /> : <EyeOff className="h-2.5 w-2.5" />}
            </span>
            <span className="flex-1" />
            <span className="flex items-center gap-0.5 text-muted-foreground/60" title={plan.updated_at}>
                <Clock className="h-2.5 w-2.5" />{formatRelativeTime(plan.updated_at)}
            </span>
        </div>
    );
}

export function PlanItem({
    plan,
    index,
    group,
    selectedPlanId,
    onSelect,
    onDuplicate,
    isDragging,
    disabled,
    dragHandleProps,
    style,
    innerRef,
}: PlanItemProps) {
    const active = plan.is_active !== false;
    const isSys = isSystemPlan(plan);
    const g = group ?? plan.plan_group ?? 'personal';
    const members = plan.member_count ?? 0;
    const isPublic = plan.is_public === true;

    return (
        <div
            ref={innerRef}
            style={style}
            onClick={() => onSelect?.(plan)}
            className={cn(
                'p-3 cursor-pointer hover:bg-muted/30 transition-colors border-l-4 group relative bg-transparent',
                selectedPlanId === plan.id ? 'bg-cyan-500/10 border-l-[#1fc7d4]' : GROUP_BORDER[g],
                isDragging === true ? 'opacity-40' : ''
            )}
        >
            <div
                {...(dragHandleProps ?? {})}
                className={cn(
                    'absolute left-1 top-1/2 -translate-y-1/2 p-1 text-muted-foreground hover:text-white cursor-grab active:cursor-grabbing opacity-0 group-hover:opacity-100 transition-opacity',
                    (disabled === true || isSys) ? 'hidden' : ''
                )}
                onClick={(e) => e.stopPropagation()}
            >
                <GripVertical className="h-4 w-4" />
            </div>
            <div className="pl-4">
                <PlanItemHeader plan={plan} index={index} isSys={isSys} active={active} permCount={plan.permissions.length} onDuplicate={onDuplicate} />
                <p className="text-xs text-muted-foreground line-clamp-1 mt-1 ml-7">{plan.description}</p>
                <PlanItemMeta plan={plan} members={members} isPublic={isPublic} />
            </div>
        </div>
    );
}

export interface SortablePlanItemProps {
    plan: PermissionPlan;
    index: number;
    group?: PlanGroup;
    selectedPlanId?: string;
    onSelect: (plan: PermissionPlan) => void;
    onDuplicate?: (plan: PermissionPlan) => void;
    disabled?: boolean;
}

export function SortablePlanItem({
    plan,
    index,
    group,
    selectedPlanId,
    onSelect,
    onDuplicate,
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
            group={group}
            selectedPlanId={selectedPlanId}
            onSelect={onSelect}
            onDuplicate={onDuplicate}
            isDragging={isDragging}
            disabled={disabled}
            dragHandleProps={{ ...attributes, ...listeners } as Record<string, unknown>}
            style={style}
            innerRef={setNodeRef}
        />
    );
}
