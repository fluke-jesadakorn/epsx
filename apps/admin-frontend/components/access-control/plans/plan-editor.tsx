import {
    Activity,
    BarChart2,
    Building2,
    Calendar,
    CalendarRange,
    CheckCircle2,
    Clock,
    Code2,
    Crown,
    DollarSign,
    Globe,
    Hash,
    Headphones,
    Layers,
    ListOrdered,
    Loader2,
    Package,
    Percent,
    PlusCircle,
    Settings2,
    SlidersHorizontal,
    Tag,
    Timer,
    Trash2,
    User,
    Users,
    Zap,
} from 'lucide-react';
import React from 'react';

import { Input, type InputProps } from '@/components/ui/input';
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
import { Button } from '@/shared/components/ui/button';
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

const FEATURE_ICONS: Record<string, React.ElementType> = {
    'epsx:rankings:offset': ListOrdered,
    'epsx:rankings:limit': Layers,
    'epsx:api:calls_limit': Activity,
    'epsx:api:ratelimit_min': Timer,
    'epsx:api:ratelimit_hour': Clock,
    'epsx:api:ratelimit_day': Calendar,
    'epsx:api:burst': Zap,
    'epsx:analytics:enabled': BarChart2,
    'epsx:support:premium': Headphones,
};

interface InputIconProps extends InputProps {
    icon: React.ElementType;
}

function InputWithIcon({ icon: Icon, className, ...props }: InputIconProps) {
    return (
        <div className="relative">
            <Icon className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground pointer-events-none z-10" />
            <Input className={`pl-9 bg-white dark:bg-white/[0.04] border-gray-200 dark:border-slate-700 ${className ?? ''}`} {...props} />
        </div>
    );
}

export interface PlanEditorProps {
    selectedPlan: PermissionPlan | null;
    form: PlanEditFormState;
    setForm: (f: (prev: PlanEditFormState) => PlanEditFormState) => void;
    setHasChanges: (hasChanges: boolean) => void;
    hasChanges?: boolean;
    isSaving?: boolean;
    onSave?: () => void;
    onDiscard?: () => void;
    onDelete?: () => void;
    permissions: PermissionDefinition[];
    onPermissionsChanged?: () => void;
}

function SectionHeader({ icon: Icon, children }: { icon?: React.ElementType; children: React.ReactNode }) {
    return (
        <div className="flex items-center gap-3 pt-2">
            {Icon != null && <Icon className="w-3.5 h-3.5 text-[#1fc7d4] shrink-0" />}
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
    hasChanges,
    isSaving,
    onSave,
    onDiscard,
    onDelete,
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
    const DiscountIcon = form.promoType === 'percentage' ? Percent : DollarSign;

    return (
        <div className="h-full flex flex-col overflow-hidden">
            {/* Stats bar */}
            <div className="px-4 sm:px-8 py-4 border-b border-gray-200 dark:border-slate-700 shrink-0">
                <div className="flex flex-wrap gap-3">
                    <StatCard icon={Users} iconClass="text-[#1fc7d4]" iconBg="bg-[#1fc7d4]/10">
                        <p className="text-base font-semibold text-white leading-tight">{selectedPlan.member_count ?? 0}</p>
                        <p className="text-[11px] text-muted-foreground">Members</p>
                    </StatCard>
                    <StatCard icon={Hash} iconClass="text-amber-400" iconBg="bg-amber-500/10">
                        <p className="text-base font-semibold text-white leading-tight">{form.priority}</p>
                        <p className="text-[11px] text-muted-foreground">Priority</p>
                    </StatCard>
                    {selectedPlan.created_at != null && (
                        <StatCard icon={Calendar} iconClass="text-blue-400" iconBg="bg-blue-500/10">
                            <p className="text-sm font-semibold text-white leading-tight">{new Date(selectedPlan.created_at).toLocaleDateString()}</p>
                            <p className="text-[11px] text-muted-foreground">Created</p>
                        </StatCard>
                    )}
                    {selectedPlan.updated_at != null && (
                        <StatCard icon={Clock} iconClass="text-purple-400" iconBg="bg-purple-500/10">
                            <p className="text-sm font-semibold text-white leading-tight">{new Date(selectedPlan.updated_at).toLocaleDateString()}</p>
                            <p className="text-[11px] text-muted-foreground">Updated</p>
                        </StatCard>
                    )}
                </div>
            </div>

            {/* Action bar (full-page editor only) */}
            {onSave != null && (
                <div className="px-4 sm:px-8 py-3 border-b border-gray-200 dark:border-slate-700 shrink-0 flex items-center justify-between gap-3">
                    <Button
                        variant="destructive"
                        size="sm"
                        onClick={onDelete}
                        disabled={isSaving}
                        className="gap-2"
                    >
                        <Trash2 className="w-4 h-4" />
                        Delete
                    </Button>
                    <div className="flex items-center gap-2">
                        {hasChanges === true && (
                            <Button
                                variant="outline"
                                size="sm"
                                onClick={onDiscard}
                                disabled={isSaving}
                            >
                                Discard
                            </Button>
                        )}
                        <Button
                            size="sm"
                            onClick={onSave}
                            disabled={!hasChanges || isSaving}
                            className="gap-2"
                        >
                            {isSaving === true && <Loader2 className="w-4 h-4 animate-spin" />}
                            Save Changes
                        </Button>
                    </div>
                </div>
            )}

            {/* Scrollable content */}
            <div className="flex-1 overflow-y-auto p-4 sm:p-8 space-y-6">

                {/* General */}
                <SectionHeader icon={Package}>General</SectionHeader>
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 sm:gap-6">
                    <div className="space-y-2">
                        <Label className="flex items-center gap-1.5">
                            <Tag className="w-3.5 h-3.5 text-muted-foreground" />
                            Plan Name
                        </Label>
                        <InputWithIcon
                            icon={Tag}
                            value={form.name}
                            onChange={(e) => {
                                setForm((p) => ({ ...p, name: e.target.value }));
                                setHasChanges(true);
                            }}
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
                                <SelectItem value="base">
                                    <span className="flex items-center gap-2"><User className="h-3.5 w-3.5 text-blue-400" /> Base</span>
                                </SelectItem>
                                <SelectItem value="addon">
                                    <span className="flex items-center gap-2"><PlusCircle className="h-3.5 w-3.5 text-amber-400" /> Addon</span>
                                </SelectItem>
                                <SelectItem value="system">
                                    <span className="flex items-center gap-2"><Settings2 className="h-3.5 w-3.5 text-purple-400" /> System</span>
                                </SelectItem>
                                <SelectItem value="exclusive">
                                    <span className="flex items-center gap-2"><Crown className="h-3.5 w-3.5 text-emerald-400" /> Exclusive</span>
                                </SelectItem>
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
                                <SelectItem value="personal">
                                    <span className="flex items-center gap-2"><User className="h-3.5 w-3.5 text-blue-400" /> Personal</span>
                                </SelectItem>
                                <SelectItem value="enterprise">
                                    <span className="flex items-center gap-2"><Building2 className="h-3.5 w-3.5 text-indigo-400" /> Enterprise</span>
                                </SelectItem>
                                <SelectItem value="api">
                                    <span className="flex items-center gap-2"><Code2 className="h-3.5 w-3.5 text-[#1fc7d4]" /> API</span>
                                </SelectItem>
                                <SelectItem value="custom">
                                    <span className="flex items-center gap-2"><SlidersHorizontal className="h-3.5 w-3.5 text-amber-400" /> Custom</span>
                                </SelectItem>
                            </SelectContent>
                        </Select>
                        <p className="text-xs text-muted-foreground">
                            Pricing page section for this plan.
                        </p>
                    </div>
                </div>

                {/* Pricing & Timing */}
                <SectionHeader icon={DollarSign}>Pricing & Timing</SectionHeader>
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 sm:gap-6">
                    <div className="space-y-2">
                        <Label>Priority</Label>
                        <InputWithIcon
                            icon={Hash}
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
                        />
                    </div>
                    <div className="space-y-2">
                        <Label className="flex items-center gap-2">
                            Price (USD)
                            {isLocked && (
                                <TooltipIcon text="Pricing for system plans cannot be modified." />
                            )}
                        </Label>
                        <InputWithIcon
                            icon={DollarSign}
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
                            disabled={isLocked}
                        />
                    </div>
                    <div className="space-y-2">
                        <Label className="flex items-center gap-2">
                            Expiry (Days)
                            <TooltipIcon text="Set to -1 for permanent expiry (never expires)." />
                        </Label>
                        <InputWithIcon
                            icon={Calendar}
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
                            placeholder="-1 for permanent"
                        />
                    </div>
                    <div className="space-y-2">
                        <Label className="flex items-center gap-2">
                            Grace Period (Hours)
                            <TooltipIcon text="Hours after expiry where access is maintained. 0 = no grace period. Max 168 (1 week)." />
                        </Label>
                        <InputWithIcon
                            icon={Clock}
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
                            placeholder="0"
                        />
                    </div>

                    {/* Promotion Card */}
                    <div className={`sm:col-span-2 rounded-xl border p-4 transition-all ${
                        form.promoEnabled
                            ? 'border-amber-500/40 bg-amber-500/[0.03]'
                            : 'border-gray-200 dark:border-slate-700'
                    }`}>
                        <div className="flex items-center justify-between">
                            <div className="flex items-center gap-3">
                                <div className={`h-8 w-8 rounded-lg flex items-center justify-center transition-colors ${
                                    form.promoEnabled ? 'bg-amber-500/15' : 'bg-white/[0.04]'
                                }`}>
                                    <Tag className={`w-4 h-4 transition-colors ${
                                        form.promoEnabled ? 'text-amber-400' : 'text-muted-foreground'
                                    }`} />
                                </div>
                                <div>
                                    <p className="text-sm font-medium text-foreground/80 dark:text-white/80">Promotion</p>
                                    <p className="text-xs text-muted-foreground">Apply a discount to this plan</p>
                                </div>
                            </div>
                            <Switch
                                checked={form.promoEnabled}
                                onCheckedChange={(checked) => {
                                    setForm((p) => ({ ...p, promoEnabled: checked }));
                                    setHasChanges(true);
                                }}
                            />
                        </div>

                        {form.promoEnabled && (
                            <div className="mt-4 pt-4 border-t border-amber-500/20 grid grid-cols-1 sm:grid-cols-2 gap-4">
                                <div className="space-y-2">
                                    <Label>Discount Type</Label>
                                    <Select
                                        value={form.promoType}
                                        onValueChange={(val) => {
                                            setForm((p) => ({ ...p, promoType: val as 'percentage' | 'fixed' }));
                                            setHasChanges(true);
                                        }}
                                    >
                                        <SelectTrigger className="bg-white dark:bg-white/[0.04] border-gray-200 dark:border-slate-700">
                                            <SelectValue />
                                        </SelectTrigger>
                                        <SelectContent>
                                            <SelectItem value="percentage">
                                                <span className="flex items-center gap-2"><Percent className="h-3.5 w-3.5 text-amber-400" /> Percentage</span>
                                            </SelectItem>
                                            <SelectItem value="fixed">
                                                <span className="flex items-center gap-2"><DollarSign className="h-3.5 w-3.5 text-green-400" /> Fixed Amount</span>
                                            </SelectItem>
                                        </SelectContent>
                                    </Select>
                                </div>
                                <div className="space-y-2">
                                    <Label>{form.promoType === 'percentage' ? 'Discount (%)' : 'Discount ($)'}</Label>
                                    <InputWithIcon
                                        icon={DiscountIcon}
                                        type="text"
                                        inputMode="decimal"
                                        value={form.promoValue}
                                        onChange={(e) => {
                                            const v = parseFloat(e.target.value);
                                            setForm((p) => ({ ...p, promoValue: isNaN(v) ? 0 : v }));
                                            setHasChanges(true);
                                        }}
                                        placeholder={form.promoType === 'percentage' ? '20' : '5.00'}
                                    />
                                </div>
                                <div className="space-y-2">
                                    <Label className="flex items-center gap-2">
                                        Final Price ($)
                                        <TooltipIcon text="Override calculated price. Leave 0 to auto-calculate." />
                                    </Label>
                                    <InputWithIcon
                                        icon={DollarSign}
                                        type="text"
                                        inputMode="decimal"
                                        value={form.promoPrice}
                                        onChange={(e) => {
                                            const v = parseFloat(e.target.value);
                                            setForm((p) => ({ ...p, promoPrice: isNaN(v) ? 0 : v }));
                                            setHasChanges(true);
                                        }}
                                        placeholder="0 = auto"
                                    />
                                    {form.promoValue > 0 && form.promoPrice === 0 && (
                                        <p className="text-xs text-amber-400/80">
                                            Auto: ${form.promoType === 'percentage'
                                                ? Math.max(0, form.price * (1 - form.promoValue / 100)).toFixed(2)
                                                : Math.max(0, form.price - form.promoValue).toFixed(2)}
                                        </p>
                                    )}
                                </div>
                                <div className="space-y-2">
                                    <Label className="flex items-center gap-2">
                                        <CalendarRange className="w-3.5 h-3.5 text-muted-foreground" />
                                        Start Date
                                    </Label>
                                    <Input
                                        type="datetime-local"
                                        value={form.promoStart}
                                        onChange={(e) => {
                                            setForm((p) => ({ ...p, promoStart: e.target.value }));
                                            setHasChanges(true);
                                        }}
                                        className="bg-white dark:bg-white/[0.04] border-gray-200 dark:border-slate-700"
                                    />
                                </div>
                                <div className="space-y-2">
                                    <Label className="flex items-center gap-2">
                                        <CalendarRange className="w-3.5 h-3.5 text-muted-foreground" />
                                        End Date
                                    </Label>
                                    <Input
                                        type="datetime-local"
                                        value={form.promoEnd}
                                        onChange={(e) => {
                                            setForm((p) => ({ ...p, promoEnd: e.target.value }));
                                            setHasChanges(true);
                                        }}
                                        className="bg-white dark:bg-white/[0.04] border-gray-200 dark:border-slate-700"
                                    />
                                </div>
                            </div>
                        )}
                    </div>
                </div>

                {/* Status */}
                <SectionHeader icon={CheckCircle2}>Status</SectionHeader>
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-4 sm:gap-6">
                    <div className="flex items-center justify-between p-4 rounded-lg bg-white dark:bg-white/[0.04] border border-gray-200 dark:border-slate-700">
                        <div className="flex items-center gap-3">
                            <div className="h-8 w-8 rounded-lg bg-blue-500/10 flex items-center justify-center shrink-0">
                                <Globe className="w-4 h-4 text-blue-400" />
                            </div>
                            <div className="flex flex-col">
                                <span className="text-sm font-medium text-foreground/80 dark:text-white/80">Public Visibility</span>
                                <span className="text-xs text-muted-foreground">Show on pricing page</span>
                            </div>
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
                        <div className="flex items-center gap-3">
                            <div className="h-8 w-8 rounded-lg bg-emerald-500/10 flex items-center justify-center shrink-0">
                                <CheckCircle2 className="w-4 h-4 text-emerald-400" />
                            </div>
                            <div className="flex flex-col">
                                <span className="text-sm font-medium text-foreground/80 dark:text-white/80">Active Status</span>
                                <span className="text-xs text-muted-foreground">Plan assignments allowed</span>
                            </div>
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
                <SectionHeader icon={Activity}>Rate Limits & Quotas</SectionHeader>
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                    {numericFeatures.map((fp) => {
                        const val = getFeatureValue(form.permissions, fp.prefix);
                        const FeatureIcon = FEATURE_ICONS[fp.prefix] ?? Hash;
                        return (
                            <div key={fp.prefix} className="space-y-1">
                                <Label className="flex items-center gap-2 text-sm">
                                    <FeatureIcon className="w-3.5 h-3.5 text-muted-foreground shrink-0" />
                                    {fp.label}
                                    {fp.tooltip != null && <TooltipIcon text={fp.tooltip} />}
                                </Label>
                                <InputWithIcon
                                    icon={FeatureIcon}
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
                                />
                            </div>
                        );
                    })}
                </div>

                {/* Feature Toggles */}
                <SectionHeader icon={Zap}>Feature Toggles</SectionHeader>
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                    {booleanFeatures.map((fp) => {
                        const val = getFeatureValue(form.permissions, fp.prefix);
                        const FeatureIcon = FEATURE_ICONS[fp.prefix] ?? Zap;
                        const isOn = val === 'true';
                        return (
                            <div key={fp.prefix} className={`flex items-center justify-between p-3 rounded-lg border transition-colors ${
                                isOn
                                    ? 'bg-[#1fc7d4]/[0.04] border-[#1fc7d4]/30'
                                    : 'bg-white dark:bg-white/[0.04] border-gray-200 dark:border-slate-700'
                            }`}>
                                <div className="flex items-center gap-3">
                                    <div className={`h-7 w-7 rounded-lg flex items-center justify-center shrink-0 transition-colors ${
                                        isOn ? 'bg-[#1fc7d4]/15' : 'bg-white/[0.04]'
                                    }`}>
                                        <FeatureIcon className={`w-3.5 h-3.5 transition-colors ${isOn ? 'text-[#1fc7d4]' : 'text-muted-foreground'}`} />
                                    </div>
                                    <Label className="flex items-center gap-2 text-sm cursor-pointer">
                                        {fp.label}
                                        {fp.tooltip != null && <TooltipIcon text={fp.tooltip} />}
                                    </Label>
                                </div>
                                <Switch
                                    checked={isOn}
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

function StatCard({
    icon: Icon,
    iconClass,
    iconBg,
    children,
}: {
    icon: React.ElementType;
    iconClass: string;
    iconBg: string;
    children: React.ReactNode;
}) {
    return (
        <div className="flex items-center gap-3 p-3 rounded-xl bg-white/[0.03] border border-gray-200 dark:border-slate-700">
            <div className={`h-9 w-9 rounded-lg ${iconBg} flex items-center justify-center shrink-0`}>
                <Icon className={`w-4 h-4 ${iconClass}`} />
            </div>
            <div className="min-w-0">{children}</div>
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
