import { Calendar, Clock, Hash, Package, Users } from 'lucide-react';
import React from 'react';

import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import {
    Select,
    SelectContent,
    SelectItem,
    SelectTrigger,
    SelectValue,
} from '@/components/ui/select';
import { Switch } from '@/components/ui/switch';
import { Textarea } from '@/components/ui/textarea';
import { type PermissionDefinition } from '@/lib/api/permissions-client';
import { type PermissionPlan } from '@/lib/api/plan-management-client';
import {
    Tooltip,
    TooltipContent,
    TooltipTrigger,
} from '@/shared/components/ui/tooltip';

import { DualPanePermissionSelector } from '../dual-pane-permission-selector';
import {
    FEATURE_PERMISSIONS,
    FREE_PLAN_ID,
    getFeatureValue,
    isSystemPlan,
    setFeatureValue,
    type PlanEditFormState,
} from './types';

export interface PlanEditorProps {
    selectedPlan: PermissionPlan | null;
    form: PlanEditFormState;
    setForm: (
        f: (prev: PlanEditFormState) => PlanEditFormState
    ) => void;
    setHasChanges: (hasChanges: boolean) => void;
    permissions: PermissionDefinition[];
    onPermissionsChanged?: () => void;
}

function SectionHeader({ children }: { children: React.ReactNode }) {
    return (
        <div className="flex items-center gap-3 pt-2">
            <Label className="text-[#1fc7d4] uppercase tracking-wider font-bold text-xs whitespace-nowrap">
                {children}
            </Label>
            <div className="h-px flex-1 bg-white dark:bg-white/[0.04]" />
        </div>
    );
}

// eslint-disable-next-line max-lines-per-function -- plan editor form with sequential sections
export function PlanEditor({
    selectedPlan,
    form,
    setForm,
    setHasChanges,
    permissions,
    onPermissionsChanged,
}: PlanEditorProps) {
    if (!selectedPlan) {
        return (
            <EmptyState
                icon={Package}
                title="Select a Plan"
                description="Select to edit plan details."
            />
        );
    }

    const isFree = selectedPlan.id === FREE_PLAN_ID;
    const isSys = isSystemPlan(selectedPlan);
    const isLocked = isFree || isSys;
    const numericFeatures = FEATURE_PERMISSIONS.filter(fp => fp.type === 'numeric');
    const booleanFeatures = FEATURE_PERMISSIONS.filter(fp => fp.type === 'boolean');

    return (
        <div className="h-full flex flex-col overflow-hidden">
            {/* Stats bar */}
            <div className="px-4 sm:px-8 py-4 border-b border-gray-200 dark:border-slate-700 shrink-0">
                <div className="flex flex-wrap gap-3">
                    <div className="flex items-center gap-3 p-3 rounded-xl bg-white/[0.03] border border-gray-200 dark:border-slate-700">
                        <div className="h-9 w-9 rounded-lg bg-[#1fc7d4]/10 flex items-center justify-center shrink-0">
                            <Users className="w-4 h-4 text-[#1fc7d4]" />
                        </div>
                        <div className="min-w-0">
                            <p className="text-base font-semibold text-white leading-tight">{selectedPlan.member_count ?? 0}</p>
                            <p className="text-[11px] text-muted-foreground">Members</p>
                        </div>
                    </div>
                    <div className="flex items-center gap-3 p-3 rounded-xl bg-white/[0.03] border border-gray-200 dark:border-slate-700">
                        <div className="h-9 w-9 rounded-lg bg-amber-500/10 flex items-center justify-center shrink-0">
                            <Hash className="w-4 h-4 text-amber-400" />
                        </div>
                        <div className="min-w-0">
                            <p className="text-base font-semibold text-white leading-tight">{form.priority}</p>
                            <p className="text-[11px] text-muted-foreground">Priority</p>
                        </div>
                    </div>
                    {selectedPlan.created_at != null && (
                        <div className="flex items-center gap-3 p-3 rounded-xl bg-white/[0.03] border border-gray-200 dark:border-slate-700">
                            <div className="h-9 w-9 rounded-lg bg-blue-500/10 flex items-center justify-center shrink-0">
                                <Calendar className="w-4 h-4 text-blue-400" />
                            </div>
                            <div className="min-w-0">
                                <p className="text-sm font-semibold text-white leading-tight">{new Date(selectedPlan.created_at).toLocaleDateString()}</p>
                                <p className="text-[11px] text-muted-foreground">Created</p>
                            </div>
                        </div>
                    )}
                    {selectedPlan.updated_at != null && (
                        <div className="flex items-center gap-3 p-3 rounded-xl bg-white/[0.03] border border-gray-200 dark:border-slate-700">
                            <div className="h-9 w-9 rounded-lg bg-purple-500/10 flex items-center justify-center shrink-0">
                                <Clock className="w-4 h-4 text-purple-400" />
                            </div>
                            <div className="min-w-0">
                                <p className="text-sm font-semibold text-white leading-tight">{new Date(selectedPlan.updated_at).toLocaleDateString()}</p>
                                <p className="text-[11px] text-muted-foreground">Updated</p>
                            </div>
                        </div>
                    )}
                </div>
            </div>

            {/* Scrollable content */}
            <div className="flex-1 overflow-y-auto p-4 sm:p-8 space-y-6">

                {/* General */}
                <SectionHeader>General</SectionHeader>
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 sm:gap-6">
                    <div className="space-y-2">
                        <Label>Plan Name</Label>
                        <Input
                            value={form.name}
                            onChange={(e) => {
                                setForm((p) => ({ ...p, name: e.target.value }));
                                setHasChanges(true);
                            }}
                            className="bg-white dark:bg-white/[0.04] border-gray-200 dark:border-slate-700"
                            disabled={isSys}
                        />
                    </div>
                    <div className="space-y-2">
                        <Label>Description</Label>
                        <Textarea
                            value={form.description}
                            onChange={(e) => {
                                setForm((p) => ({ ...p, description: e.target.value }));
                                setHasChanges(true);
                            }}
                            className="bg-white dark:bg-white/[0.04] border-gray-200 dark:border-slate-700 min-h-[80px] resize-y"
                            rows={3}
                        />
                    </div>
                    <div className="space-y-2">
                        <Label>Category</Label>
                        <Select
                            value={form.plan_category}
                            onValueChange={(val) => {
                                setForm((p) => ({ ...p, plan_category: val as 'base' | 'addon' | 'system' | 'exclusive' }));
                                setHasChanges(true);
                            }}
                            disabled={isSys}
                        >
                            <SelectTrigger className="bg-white dark:bg-white/[0.04] border-gray-200 dark:border-slate-700">
                                <SelectValue />
                            </SelectTrigger>
                            <SelectContent>
                                <SelectItem value="base">Base</SelectItem>
                                <SelectItem value="addon">Addon</SelectItem>
                                <SelectItem value="system">System</SelectItem>
                                <SelectItem value="exclusive">Exclusive</SelectItem>
                            </SelectContent>
                        </Select>
                        <p className="text-xs text-muted-foreground">
                            Base: 1 per wallet. Addon/System: stackable. Exclusive: max 3.
                        </p>
                    </div>
                    <div className="space-y-2">
                        <Label>Display Group</Label>
                        <Select
                            value={form.plan_group}
                            onValueChange={(val) => {
                                setForm((p) => ({ ...p, plan_group: val as 'personal' | 'enterprise' | 'api' | 'custom' }));
                                setHasChanges(true);
                            }}
                            disabled={isSys}
                        >
                            <SelectTrigger className="bg-white dark:bg-white/[0.04] border-gray-200 dark:border-slate-700">
                                <SelectValue />
                            </SelectTrigger>
                            <SelectContent>
                                <SelectItem value="personal">Personal</SelectItem>
                                <SelectItem value="enterprise">Enterprise</SelectItem>
                                <SelectItem value="api">API</SelectItem>
                                <SelectItem value="custom">Custom</SelectItem>
                            </SelectContent>
                        </Select>
                        <p className="text-xs text-muted-foreground">
                            Pricing page section for this plan.
                        </p>
                    </div>
                </div>

                {/* Pricing & Timing */}
                <SectionHeader>Pricing & Timing</SectionHeader>
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 sm:gap-6">
                    <div className="space-y-2">
                        <Label>Priority</Label>
                        <Input
                            type="text"
                            inputMode="numeric"
                            value={form.priority}
                            onChange={(e) => {
                                const val = e.target.value;
                                if (val === '-' || val === '') {
                                    setForm((p) => ({
                                        ...p,
                                        priority: val as unknown as number,
                                    }));
                                } else {
                                    const parsed = parseInt(val);
                                    if (!isNaN(parsed)) {
                                        setForm((p) => ({ ...p, priority: parsed }));
                                    }
                                }
                                setHasChanges(true);
                            }}
                            className="bg-white dark:bg-white/[0.04] border-gray-200 dark:border-slate-700"
                        />
                    </div>
                    <div className="space-y-2">
                        <Label className="flex items-center gap-2">
                            Price (USD)
                            {isLocked && (
                                <TooltipIcon text="Pricing for system plans cannot be modified." />
                            )}
                        </Label>
                        <Input
                            type="text"
                            inputMode="decimal"
                            value={form.price}
                            onChange={(e) => {
                                const val = e.target.value;
                                if (val === '-' || val === '' || val === '.') {
                                    setForm((p) => ({ ...p, price: val as unknown as number }));
                                } else {
                                    const parsed = parseFloat(val);
                                    if (!isNaN(parsed)) {
                                        setForm((p) => ({ ...p, price: parsed }));
                                    }
                                }
                                setHasChanges(true);
                            }}
                            className="bg-white dark:bg-white/[0.04] border-gray-200 dark:border-slate-700"
                            disabled={isLocked}
                        />
                    </div>
                    <div className="space-y-2">
                        <Label className="flex items-center gap-2">
                            Expiry (Days)
                            <TooltipIcon text="Set to -1 for permanent expiry (never expires)." />
                        </Label>
                        <Input
                            type="text"
                            inputMode="numeric"
                            value={form.expiryDays}
                            onChange={(e) => {
                                const val = e.target.value;
                                if (val === '-' || val === '') {
                                    setForm((p) => ({
                                        ...p,
                                        expiryDays: val as unknown as number,
                                    }));
                                } else {
                                    const parsed = parseInt(val);
                                    if (!isNaN(parsed)) {
                                        setForm((p) => ({ ...p, expiryDays: parsed }));
                                    }
                                }
                                setHasChanges(true);
                            }}
                            className="bg-white dark:bg-white/[0.04] border-gray-200 dark:border-slate-700"
                            placeholder="-1 for permanent"
                        />
                    </div>
                    <div className="space-y-2">
                        <Label className="flex items-center gap-2">
                            Grace Period (Hours)
                            <TooltipIcon text="Hours after expiry where access is maintained. 0 = no grace period. Max 168 (1 week)." />
                        </Label>
                        <Input
                            type="text"
                            inputMode="numeric"
                            value={form.gracePeriodHours}
                            onChange={(e) => {
                                const val = e.target.value;
                                if (val === '' || val === '0') {
                                    setForm((p) => ({
                                        ...p,
                                        gracePeriodHours: val === '' ? (0 as number) : 0,
                                    }));
                                } else {
                                    const parsed = parseInt(val);
                                    if (!isNaN(parsed) && parsed >= 0 && parsed <= 168) {
                                        setForm((p) => ({ ...p, gracePeriodHours: parsed }));
                                    }
                                }
                                setHasChanges(true);
                            }}
                            className="bg-white dark:bg-white/[0.04] border-gray-200 dark:border-slate-700"
                            placeholder="0"
                        />
                    </div>
                </div>

                {/* Status */}
                <SectionHeader>Status</SectionHeader>
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 sm:gap-6">
                    <div className="flex items-center justify-between p-4 rounded-lg bg-white dark:bg-white/[0.04] border border-gray-200 dark:border-slate-700">
                        <div className="flex flex-col">
                            <span className="text-sm font-medium text-foreground/80 dark:text-white/80">Public Visibility</span>
                            <span className="text-xs text-white/40">Show on pricing page</span>
                        </div>
                        <Tooltip>
                            <TooltipTrigger asChild>
                                <div className={isLocked ? 'cursor-not-allowed opacity-80' : ''}>
                                    <Switch
                                        checked={form.is_public}
                                        disabled={isLocked}
                                        onCheckedChange={(checked) => {
                                            setForm((p) => ({ ...p, is_public: checked }));
                                            setHasChanges(true);
                                        }}
                                    />
                                </div>
                            </TooltipTrigger>
                            {isLocked && (
                                <TooltipContent side="left" className="bg-white dark:bg-slate-900 border-gray-200 dark:border-slate-700 text-white max-w-[200px]">
                                    <p className="text-xs">System plan visibility cannot be changed</p>
                                </TooltipContent>
                            )}
                        </Tooltip>
                    </div>
                    <div className="flex items-center justify-between p-4 rounded-lg bg-white dark:bg-white/[0.04] border border-gray-200 dark:border-slate-700">
                        <div className="flex flex-col">
                            <span className="text-sm font-medium text-foreground/80 dark:text-white/80">Active Status</span>
                            <span className="text-xs text-white/40">Plan assignments allowed</span>
                        </div>
                        <Tooltip>
                            <TooltipTrigger asChild>
                                <div className={isLocked ? 'cursor-not-allowed opacity-80' : ''}>
                                    <Switch
                                        checked={form.is_active}
                                        disabled={isLocked}
                                        onCheckedChange={(checked) => {
                                            setForm((p) => ({ ...p, is_active: checked }));
                                            setHasChanges(true);
                                        }}
                                    />
                                </div>
                            </TooltipTrigger>
                            {isLocked && (
                                <TooltipContent side="left" className="bg-white dark:bg-slate-900 border-gray-200 dark:border-slate-700 text-white max-w-[200px]">
                                    <p className="text-xs">System plan status cannot be changed</p>
                                </TooltipContent>
                            )}
                        </Tooltip>
                    </div>
                </div>

                {/* Rate Limits & Quotas */}
                <SectionHeader>Rate Limits & Quotas</SectionHeader>
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                    {numericFeatures.map((fp) => {
                        const val = getFeatureValue(form.permissions, fp.prefix);
                        return (
                            <div key={fp.prefix} className="space-y-1">
                                <Label className="flex items-center gap-2 text-sm">
                                    {fp.label}
                                    {fp.tooltip != null && <TooltipIcon text={fp.tooltip} />}
                                </Label>
                                <Input
                                    type="text"
                                    inputMode="numeric"
                                    value={val ?? ''}
                                    placeholder={fp.placeholder}
                                    onChange={(e) => {
                                        const v = e.target.value;
                                        setForm((p) => ({
                                            ...p,
                                            permissions: setFeatureValue(p.permissions, fp.prefix, v === '' ? null : v),
                                        }));
                                        setHasChanges(true);
                                    }}
                                    className="bg-white dark:bg-white/[0.04] border-gray-200 dark:border-slate-700"
                                />
                            </div>
                        );
                    })}
                </div>

                {/* Feature Toggles */}
                <SectionHeader>Feature Toggles</SectionHeader>
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                    {booleanFeatures.map((fp) => {
                        const val = getFeatureValue(form.permissions, fp.prefix);
                        return (
                            <div key={fp.prefix} className="flex items-center justify-between p-3 rounded-lg bg-white dark:bg-white/[0.04] border border-gray-200 dark:border-slate-700">
                                <Label className="flex items-center gap-2 text-sm">
                                    {fp.label}
                                    {fp.tooltip != null && <TooltipIcon text={fp.tooltip} />}
                                </Label>
                                <Switch
                                    checked={val === 'true'}
                                    onCheckedChange={(checked) => {
                                        setForm((p) => ({
                                            ...p,
                                            permissions: setFeatureValue(p.permissions, fp.prefix, checked ? 'true' : null),
                                        }));
                                        setHasChanges(true);
                                    }}
                                />
                            </div>
                        );
                    })}
                </div>

                {/* Pricing Page Features */}
                <SectionHeader>Pricing Page Features</SectionHeader>
                <div className="space-y-2">
                    <p className="text-xs text-muted-foreground">
                        Displayed on the public pricing page. One per line.
                    </p>
                    <Textarea
                        value={form.features.join('\n')}
                        onChange={(e) => {
                            const list = e.target.value.split('\n');
                            setForm((p) => ({ ...p, features: list }));
                            setHasChanges(true);
                        }}
                        className="bg-white dark:bg-white/[0.04] border-gray-200 dark:border-slate-700 min-h-[100px] font-mono text-sm"
                        placeholder={'Advanced analytics\nUnlimited stock analysis\nPriority support'}
                    />
                </div>

                {/* Permission Assignment */}
                <SectionHeader>Permission Assignment</SectionHeader>
                <div className="h-[400px] sm:h-[500px]">
                    <DualPanePermissionSelector
                        availablePermissions={permissions}
                        assignedPermissionStrings={form.permissions}
                        onChange={(newPermissions) => {
                            setForm((prev) => ({
                                ...prev,
                                permissions: newPermissions,
                            }));
                            setHasChanges(true);
                        }}
                        onPermissionsChanged={onPermissionsChanged}
                    />
                </div>
            </div>
        </div>
    );
}

function TooltipIcon({ text }: { text: string }) {
    return (
        <Tooltip>
            <TooltipTrigger asChild>
                <div className="h-3.5 w-3.5 rounded-full bg-[#1fc7d4]/20 flex items-center justify-center cursor-help">
                    <span className="text-[10px] font-bold text-[#1fc7d4]">?</span>
                </div>
            </TooltipTrigger>
            <TooltipContent side="right" className="bg-white dark:bg-slate-900 border-gray-200 dark:border-slate-700 text-white max-w-[200px]">
                <p className="text-xs">{text}</p>
            </TooltipContent>
        </Tooltip>
    );
}

function EmptyState({
    icon: Icon,
    title,
    description,
}: {
    icon: React.ElementType;
    title: string;
    description: string;
}) {
    return (
        <div className="h-full flex flex-col items-center justify-center border border-dashed border-gray-200 dark:border-slate-700 rounded-[32px] bg-white/60 dark:bg-slate-900/60 text-slate-500 p-8 text-center">
            <div className="h-20 w-20 rounded-full bg-white dark:bg-white/[0.04] flex items-center justify-center mb-6">
                <Icon className="h-10 w-10 opacity-30" />
            </div>
            <h3 className="text-xl font-bold text-slate-300 mb-2">{title}</h3>
            <p className="text-sm max-w-xs mx-auto">{description}</p>
        </div>
    );
}
