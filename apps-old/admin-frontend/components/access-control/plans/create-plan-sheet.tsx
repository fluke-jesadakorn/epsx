import { Loader2, Plus } from 'lucide-react';
import React, { useEffect, useState } from 'react';

import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import {
    Sheet,
    SheetContent,
    SheetDescription,
    SheetFooter,
    SheetHeader,
    SheetTitle,
    SheetTrigger,
} from '@/components/ui/sheet';
import { Textarea } from '@/components/ui/textarea';
import { type PlanGroup, type PermissionPlan } from '@/lib/api/plan-management-client';
import {
    Tooltip,
    TooltipContent,
    TooltipTrigger,
} from '@/shared/components/ui/tooltip';

import { isSystemPlan } from './types';
import { submitCreatePlan } from './use-plans-logic';

interface FormData {
    name: string;
    description: string;
    priority: number;
    price: number;
    default_expiry_days: number;
    permissions: string[];
    plan_group: PlanGroup;
}

const emptyForm: FormData = {
    name: '',
    description: '',
    priority: 0,
    price: 0,
    default_expiry_days: 30,
    permissions: [],
    plan_group: 'personal',
};

interface PlanFormFieldsProps {
    formData: FormData;
    setFormData: (f: FormData) => void;
    submitting: boolean;
}

function PlanFormFields({ formData, setFormData, submitting }: PlanFormFieldsProps) {
    const set = (patch: Partial<FormData>) => setFormData({ ...formData, ...patch });
    return (
        <>
            <div className="space-y-2">
                <Label>Plan Name *</Label>
                <Input required value={formData.name} onChange={(e) => set({ name: e.target.value })} className="bg-muted/30" />
            </div>
            <div className="space-y-2">
                <Label>Display Group</Label>
                <select value={formData.plan_group} onChange={(e) => set({ plan_group: e.target.value as PlanGroup })} className="w-full h-9 rounded-md border border-border/20 bg-muted/30 px-3 text-sm text-white">
                    <option value="personal">Personal</option>
                    <option value="enterprise">Enterprise</option>
                    <option value="api">API</option>
                    <option value="custom">Custom</option>
                </select>
                <p className="text-xs text-muted-foreground">Pricing page section</p>
            </div>
            <div className="grid grid-cols-2 gap-4">
                <div className="space-y-2">
                    <Label>Priority</Label>
                    <Input type="number" value={formData.priority} onChange={(e) => set({ priority: parseInt(e.target.value) })} className="bg-muted/30" />
                </div>
                <div className="space-y-2">
                    <Label>Price</Label>
                    <Input type="number" step="0.01" value={formData.price} onChange={(e) => set({ price: parseFloat(e.target.value) })} className="bg-muted/30" />
                </div>
            </div>
            <div className="space-y-2">
                <Label className="flex items-center gap-2">
                    Expiry (Days)
                    <Tooltip>
                        <TooltipTrigger asChild>
                            <div className="h-3.5 w-3.5 rounded-full bg-[#1fc7d4]/20 flex items-center justify-center cursor-help">
                                <span className="text-[10px] font-bold text-[#1fc7d4]">?</span>
                            </div>
                        </TooltipTrigger>
                        <TooltipContent side="right" className="bg-card border-border/20 text-white max-w-[200px]">
                            <p className="text-xs">Set to -1 for permanent expiry (never expires).</p>
                        </TooltipContent>
                    </Tooltip>
                </Label>
                <Input type="text" inputMode="numeric" value={formData.default_expiry_days}
                    onChange={(e) => {
                        const val = e.target.value;
                        if (val === '-' || val === '') {
                            set({ default_expiry_days: val as unknown as number });
                        } else {
                            const parsed = parseInt(val);
                            if (!isNaN(parsed)) { set({ default_expiry_days: parsed }); }
                        }
                    }}
                    className="bg-muted/30" placeholder="-1 for permanent" />
            </div>
            <div className="space-y-2">
                <Label>Description</Label>
                <Textarea value={formData.description} onChange={(e) => set({ description: e.target.value })} className="bg-muted/30 min-h-[100px]" />
            </div>
            <SheetFooter className="mt-auto pt-6">
                <Button type="submit" disabled={submitting} className="bg-[#1fc7d4] text-white w-full">
                    {submitting ? <Loader2 className="animate-spin w-4 h-4" /> : 'Create Plan'}
                </Button>
            </SheetFooter>
        </>
    );
}

export function CreatePlanSheet({
    open,
    onOpenChange,
    onSuccess,
    sourcePlan,
    onSourceClear,
}: {
    open: boolean;
    onOpenChange: (o: boolean) => void;
    onSuccess: () => void;
    sourcePlan?: PermissionPlan | null;
    onSourceClear?: () => void;
}) {
    const [formData, setFormData] = useState(emptyForm);
    const [submitting, setSubmitting] = useState(false);

    useEffect(() => {
        if (open && sourcePlan !== null && sourcePlan !== undefined) {
            const isSys = isSystemPlan(sourcePlan);
            setFormData({
                name: isSys ? `${sourcePlan.name} Template` : `${sourcePlan.name} (Copy)`,
                description: sourcePlan.description,
                priority: sourcePlan.tier_level,
                price: sourcePlan.price ?? 0,
                default_expiry_days: sourcePlan.default_expiry_days ?? 30,
                permissions: sourcePlan.permissions,
                plan_group: sourcePlan.plan_group ?? 'custom',
            });
        } else if (!open) {
            setFormData(emptyForm);
            onSourceClear?.();
        }
    }, [open, sourcePlan, onSourceClear]);

    const isDuplicate = sourcePlan !== null && sourcePlan !== undefined;
    const isTemplate = isDuplicate && isSystemPlan(sourcePlan);

    const handleSubmit = (e: React.FormEvent) => {
        e.preventDefault();
        setSubmitting(true);
        void (async () => {
            try {
                await submitCreatePlan(formData, onSuccess);
                setFormData(emptyForm);
                onOpenChange(false);
            } catch {
                // Error handled in submitCreatePlan
            } finally {
                setSubmitting(false);
            }
        })();
    };

    const planName = sourcePlan !== null && sourcePlan !== undefined ? sourcePlan.name : '';
    const title = isTemplate ? 'Create from Template' : isDuplicate ? 'Duplicate Plan' : 'Create Plan';
    const desc = isTemplate
        ? `Create an editable admin plan based on "${planName}".`
        : isDuplicate
            ? `Create a new plan based on "${planName}".`
            : 'Create a new access plan.';

    return (
        <Sheet open={open} onOpenChange={onOpenChange}>
            <SheetTrigger asChild>
                <Button size="icon" className="h-9 w-9 bg-[#1fc7d4] hover:bg-[#1fc7d4]/90">
                    <Plus className="h-4 w-4" />
                </Button>
            </SheetTrigger>
            <SheetContent side="right" className="w-[400px] sm:w-[540px] bg-card border-border/20 text-white flex flex-col h-full">
                <SheetHeader>
                    <SheetTitle>{title}</SheetTitle>
                    <SheetDescription>{desc}</SheetDescription>
                </SheetHeader>
                <form onSubmit={handleSubmit} className="space-y-6 pt-6 flex-1 flex flex-col overflow-y-auto">
                    <PlanFormFields formData={formData} setFormData={setFormData} submitting={submitting} />
                </form>
            </SheetContent>
        </Sheet>
    );
}
