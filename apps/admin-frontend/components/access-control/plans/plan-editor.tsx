import { Loader2, Package, RotateCcw, Trash2 } from 'lucide-react';
import React from 'react';

import { Button } from '@/components/ui/button';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
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
import { FREE_PLAN_ID, type PlanEditFormState } from './types';

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
    permissions: PermissionDefinition[];
}

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

    return (
        <Card className="h-full border border-white/5 bg-slate-900/40 backdrop-blur-xl shadow-xl rounded-[32px] overflow-hidden flex flex-col">
            <CardHeader className="py-6 px-8 border-b border-white/5 bg-white/5 flex flex-row items-center justify-between shrink-0">
                <div className="flex items-center gap-3">
                    <div className="h-10 w-10 rounded-xl bg-[#1fc7d4]/20 flex items-center justify-center">
                        <Package className="w-5 h-5 text-[#1fc7d4]" />
                    </div>
                    <div>
                        <CardTitle className="text-lg font-bold">Edit Plan</CardTitle>
                        <p className="text-xs text-muted-foreground font-mono">
                            {selectedPlan.id}
                        </p>
                    </div>
                </div>
                <div className="flex gap-2">
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
                        Save Changes
                    </Button>
                </div>
            </CardHeader>
            <CardContent className="p-8 space-y-8 overflow-y-auto flex-1">
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
                        />
                    </div>
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
                            {isFree && (
                                <Tooltip>
                                    <TooltipTrigger asChild>
                                        <div className="h-3.5 w-3.5 rounded-full bg-slate-500/20 flex items-center justify-center cursor-help">
                                            <span className="text-[10px] font-bold">?</span>
                                        </div>
                                    </TooltipTrigger>
                                    <TooltipContent
                                        side="right"
                                        className="bg-slate-900 border-white/10 text-white max-w-[200px]"
                                    >
                                        <p className="text-xs">
                                            Pricing for the Free Plan is permanent and cannot be
                                            modified.
                                        </p>
                                    </TooltipContent>
                                </Tooltip>
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
                            disabled={isFree}
                        />
                    </div>
                    <div className="space-y-2">
                        <Label className="flex items-center gap-2">
                            Expiry (Days)
                            <Tooltip>
                                <TooltipTrigger asChild>
                                    <div className="h-3.5 w-3.5 rounded-full bg-[#1fc7d4]/20 flex items-center justify-center cursor-help">
                                        <span className="text-[10px] font-bold text-[#1fc7d4]">
                                            ?
                                        </span>
                                    </div>
                                </TooltipTrigger>
                                <TooltipContent
                                    side="right"
                                    className="bg-slate-900 border-white/10 text-white max-w-[200px]"
                                >
                                    <p className="text-xs">
                                        Set to -1 for permanent expiry (never expires).
                                    </p>
                                </TooltipContent>
                            </Tooltip>
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
                            <Tooltip>
                                <TooltipTrigger asChild>
                                    <div className="h-3.5 w-3.5 rounded-full bg-[#1fc7d4]/20 flex items-center justify-center cursor-help">
                                        <span className="text-[10px] font-bold text-[#1fc7d4]">
                                            ?
                                        </span>
                                    </div>
                                </TooltipTrigger>
                                <TooltipContent
                                    side="right"
                                    className="bg-slate-900 border-white/10 text-white max-w-[200px]"
                                >
                                    <p className="text-xs">
                                        Hours after expiry where access is
                                        maintained. 0 = no grace period. Max 168
                                        (1 week).
                                    </p>
                                </TooltipContent>
                            </Tooltip>
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
                    <div className="col-span-2 space-y-2">
                        <Label>Description</Label>
                        <Textarea
                            value={form.description}
                            onChange={(e) => {
                                setForm((p) => ({ ...p, description: e.target.value }));
                                setHasChanges(true);
                            }}
                            className="bg-white/5 border-white/10"
                        />
                    </div>
                    <div className="col-span-2 space-y-2">
                        <Label>Features (One per line)</Label>
                        <p className="text-xs text-muted-foreground">
                            These features will be displayed on the public pricing page.
                        </p>
                        <Textarea
                            value={form.features.join('\n')}
                            onChange={(e) => {
                                const features_list = e.target.value.split('\n');
                                setForm((p) => ({ ...p, features: features_list }));
                                setHasChanges(true);
                            }}
                            className="bg-white/5 border-white/10 min-h-[120px] font-mono text-sm"
                            placeholder={'Advanced analytics\nUnlimited stock analysis\nPriority support'}
                        />
                    </div>
                    <div className="col-span-2 flex items-center justify-between p-4 rounded-lg bg-white/5 border border-white/10">
                        <div className="flex flex-col">
                            <span className="text-sm font-medium text-white/80">
                                Public Visibility
                            </span>
                            <span className="text-xs text-white/40">
                                Show on pricing page
                            </span>
                        </div>
                        <Tooltip>
                            <TooltipTrigger asChild>
                                <div
                                    className={
                                        isFree ? 'cursor-not-allowed opacity-80' : ''
                                    }
                                >
                                    <Switch
                                        checked={form.is_public}
                                        disabled={isFree}
                                        onCheckedChange={(checked) => {
                                            setForm((p) => ({ ...p, is_public: checked }));
                                            setHasChanges(true);
                                        }}
                                    />
                                </div>
                            </TooltipTrigger>
                            {isFree && (
                                <TooltipContent
                                    side="left"
                                    className="bg-slate-900 border-white/10 text-white max-w-[200px]"
                                >
                                    <p className="text-xs">
                                        Default system plan visibility cannot be changed
                                    </p>
                                </TooltipContent>
                            )}
                        </Tooltip>
                    </div>
                    <div className="col-span-2 flex items-center justify-between p-4 rounded-lg bg-white/5 border border-white/10">
                        <div className="flex flex-col">
                            <span className="text-sm font-medium text-white/80">
                                Active Status
                            </span>
                            <span className="text-xs text-white/40">
                                Plan assignments allowed
                            </span>
                        </div>
                        <Tooltip>
                            <TooltipTrigger asChild>
                                <div
                                    className={
                                        isFree ? 'cursor-not-allowed opacity-80' : ''
                                    }
                                >
                                    <Switch
                                        checked={form.is_active}
                                        disabled={isFree}
                                        onCheckedChange={(checked) => {
                                            setForm((p) => ({ ...p, is_active: checked }));
                                            setHasChanges(true);
                                        }}
                                    />
                                </div>
                            </TooltipTrigger>
                            {isFree && (
                                <TooltipContent
                                    side="left"
                                    className="bg-slate-900 border-white/10 text-white max-w-[200px]"
                                >
                                    <p className="text-xs">
                                        Default system plan status cannot be changed
                                    </p>
                                </TooltipContent>
                            )}
                        </Tooltip>
                    </div>
                </div>

                <div className="space-y-4">
                    <Label className="text-[#1fc7d4] uppercase tracking-wider font-bold text-xs">
                        Permission Assignment
                    </Label>
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
                        />
                    </div>
                </div>
            </CardContent>
        </Card>
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
