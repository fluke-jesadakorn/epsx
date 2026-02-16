import { Calendar, Clock, Copy, Hash, Loader2, Package, RotateCcw, Shield, Trash2, Users } from 'lucide-react';
import React from 'react';

import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardHeader, CardTitle } from '@/components/ui/card';
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
    categoryBadgeClass,
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
    hasChanges: boolean;
    isSaving: boolean;
    onSave: () => void;
    onDiscard: () => void;
    onDelete: () => void;
    onDuplicate?: (plan: PermissionPlan) => void;
    permissions: PermissionDefinition[];
    onPermissionsChanged?: () => void;
}

function SectionHeader({ children }: { children: React.ReactNode }) {
    return (
        <div className="flex items-center gap-3 pt-2">
            <Label className="text-[#1fc7d4] uppercase tracking-wider font-bold text-xs whitespace-nowrap">
                {children}
            </Label>
            <div className="h-px flex-1 bg-white/5" />
        </div>
    );
}

// eslint-disable-next-line max-lines-per-function -- plan editor form with sequential sections
export function PlanEditor({
    selectedPlan,
    form,
    setForm,
    setHasChanges,
    hasChanges,
    isSaving,
    onSave,
    onDiscard,
    onDelete,
    onDuplicate,
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
        <Card className="h-full border border-white/5 bg-slate-900/40 backdrop-blur-xl shadow-xl rounded-[32px] overflow-hidden flex flex-col">
            {/* Header */}
            <CardHeader className="py-5 px-8 border-b border-white/5 bg-white/5 flex flex-row items-center justify-between shrink-0">
                <div className="flex items-center gap-3 min-w-0">
                    <div className="h-10 w-10 rounded-xl bg-[#1fc7d4]/20 flex items-center justify-center shrink-0">
                        <Package className="w-5 h-5 text-[#1fc7d4]" />
                    </div>
                    <div className="min-w-0">
                        <div className="flex items-center gap-2">
                            <CardTitle className="text-lg font-bold truncate">
                                {selectedPlan.name}
                            </CardTitle>
                            <Badge variant="outline" className={`text-[10px] px-1.5 py-0 shrink-0 ${categoryBadgeClass(form.plan_category)}`}>
                                {form.plan_category}
                            </Badge>
                            {isSys && (
                                <Badge variant="outline" className="text-[10px] px-1.5 py-0 shrink-0 bg-purple-500/15 text-purple-400 border-purple-500/30">
                                    <Shield className="w-2.5 h-2.5 mr-1" />
                                    System
                                </Badge>
                            )}
                            {!form.is_active && (
                                <Badge variant="outline" className="text-[10px] px-1.5 py-0 shrink-0 bg-red-500/15 text-red-400 border-red-500/30">
                                    inactive
                                </Badge>
                            )}
                        </div>
                        <p className="text-[11px] text-muted-foreground font-mono truncate">
                            {selectedPlan.slug ?? selectedPlan.id}
                        </p>
                    </div>
                </div>
                <div className="flex gap-2 shrink-0">
                    {isSys ? (
                        <Button
                            variant="ghost"
                            className="text-purple-400 hover:text-purple-300 hover:bg-purple-500/10"
                            size="sm"
                            onClick={() => onDuplicate?.(selectedPlan)}
                        >
                            <Copy className="w-4 h-4 mr-2" />
                            Use as Template
                        </Button>
                    ) : (
                        <Button
                            variant="ghost"
                            className="text-red-400 hover:text-red-300 hover:bg-red-500/10"
                            size="sm"
                            onClick={onDelete}
                            disabled={isFree}
                        >
                            <Trash2 className="w-4 h-4 mr-2" />
                            Delete
                        </Button>
                    )}
                    <Button
                        variant="ghost"
                        size="sm"
                        onClick={onDiscard}
                        disabled={!hasChanges || isSaving}
                        className="text-muted-foreground hover:text-foreground"
                    >
                        <RotateCcw className="w-4 h-4 mr-2" />
                        Discard
                    </Button>
                    <Button
                        size="sm"
                        onClick={onSave}
                        disabled={!hasChanges || isSaving}
                        className="bg-[#1fc7d4] text-white hover:bg-[#1fc7d4]/90"
                    >
                        {isSaving && <Loader2 className="w-3 h-3 animate-spin mr-2" />}
                        Save
                    </Button>
                </div>
            </CardHeader>

            {/* Stats bar */}
            <div className="px-8 py-3 border-b border-white/5 flex items-center gap-6 text-xs text-muted-foreground shrink-0">
                <span className="flex items-center gap-1.5">
                    <Users className="w-3.5 h-3.5" />
                    {selectedPlan.member_count ?? 0} members
                </span>
                <span className="flex items-center gap-1.5">
                    <Hash className="w-3.5 h-3.5" />
                    Priority {form.priority}
                </span>
                {selectedPlan.created_at != null && (
                    <span className="flex items-center gap-1.5">
                        <Calendar className="w-3.5 h-3.5" />
                        {new Date(selectedPlan.created_at).toLocaleDateString()}
                    </span>
                )}
                {selectedPlan.updated_at != null && (
                    <span className="flex items-center gap-1.5">
                        <Clock className="w-3.5 h-3.5" />
                        {new Date(selectedPlan.updated_at).toLocaleDateString()}
                    </span>
                )}
            </div>

            {/* Scrollable content */}
            <div className="flex-1 overflow-y-auto p-8 space-y-6">

                {/* General */}
                <SectionHeader>General</SectionHeader>
                <div className="grid grid-cols-2 gap-6">
                    <div className="space-y-2">
                        <Label>Plan Name</Label>
                        <Input
                            value={form.name}
                            onChange={(e) => {
                                setForm((p) => ({ ...p, name: e.target.value }));
                                setHasChanges(true);
                            }}
                            className="bg-white/5 border-white/10"
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
                            className="bg-white/5 border-white/10 min-h-[38px] h-[38px] resize-y"
                            rows={1}
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
                            <SelectTrigger className="bg-white/5 border-white/10">
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
                            <SelectTrigger className="bg-white/5 border-white/10">
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
                <div className="grid grid-cols-2 gap-6">
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
                            className="bg-white/5 border-white/10"
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
                            className="bg-white/5 border-white/10"
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
                            className="bg-white/5 border-white/10"
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
                            className="bg-white/5 border-white/10"
                            placeholder="0"
                        />
                    </div>
                </div>

                {/* Status */}
                <SectionHeader>Status</SectionHeader>
                <div className="grid grid-cols-2 gap-6">
                    <div className="flex items-center justify-between p-4 rounded-lg bg-white/5 border border-white/10">
                        <div className="flex flex-col">
                            <span className="text-sm font-medium text-white/80">Public Visibility</span>
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
                                <TooltipContent side="left" className="bg-slate-900 border-white/10 text-white max-w-[200px]">
                                    <p className="text-xs">System plan visibility cannot be changed</p>
                                </TooltipContent>
                            )}
                        </Tooltip>
                    </div>
                    <div className="flex items-center justify-between p-4 rounded-lg bg-white/5 border border-white/10">
                        <div className="flex flex-col">
                            <span className="text-sm font-medium text-white/80">Active Status</span>
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
                                <TooltipContent side="left" className="bg-slate-900 border-white/10 text-white max-w-[200px]">
                                    <p className="text-xs">System plan status cannot be changed</p>
                                </TooltipContent>
                            )}
                        </Tooltip>
                    </div>
                </div>

                {/* Rate Limits & Quotas */}
                <SectionHeader>Rate Limits & Quotas</SectionHeader>
                <div className="grid grid-cols-2 gap-4">
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
                                    className="bg-white/5 border-white/10"
                                />
                            </div>
                        );
                    })}
                </div>

                {/* Feature Toggles */}
                <SectionHeader>Feature Toggles</SectionHeader>
                <div className="grid grid-cols-2 gap-4">
                    {booleanFeatures.map((fp) => {
                        const val = getFeatureValue(form.permissions, fp.prefix);
                        return (
                            <div key={fp.prefix} className="flex items-center justify-between p-3 rounded-lg bg-white/5 border border-white/10">
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
                        className="bg-white/5 border-white/10 min-h-[100px] font-mono text-sm"
                        placeholder={'Advanced analytics\nUnlimited stock analysis\nPriority support'}
                    />
                </div>

                {/* Permission Assignment */}
                <SectionHeader>Permission Assignment</SectionHeader>
                <div className="h-[500px]">
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
        </Card>
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
            <TooltipContent side="right" className="bg-slate-900 border-white/10 text-white max-w-[200px]">
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
        <div className="h-full flex flex-col items-center justify-center border border-dashed border-white/10 rounded-[32px] bg-slate-900/20 text-slate-500 p-8 text-center">
            <div className="h-20 w-20 rounded-full bg-white/5 flex items-center justify-center mb-6">
                <Icon className="h-10 w-10 opacity-30" />
            </div>
            <h3 className="text-xl font-bold text-slate-300 mb-2">{title}</h3>
            <p className="text-sm max-w-xs mx-auto">{description}</p>
        </div>
    );
}
