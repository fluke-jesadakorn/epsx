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

const emptyForm = {
    name: '',
    description: '',
    priority: 0,
    price: 0,
    default_expiry_days: 30,
    permissions: [] as string[],
    plan_group: 'personal' as PlanGroup,
};

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
        if (open && sourcePlan) {
            const isSys = isSystemPlan(sourcePlan);
            setFormData({
                name: isSys ? `${sourcePlan.name} Template` : `${sourcePlan.name} (Copy)`,
                description: sourcePlan.description ?? '',
                priority: sourcePlan.tier_level ?? 0,
                price: sourcePlan.price ?? 0,
                default_expiry_days: sourcePlan.default_expiry_days ?? 30,
                permissions: sourcePlan.permissions ?? [],
                plan_group: sourcePlan.plan_group ?? 'custom',
            });
        } else if (!open) {
            setFormData(emptyForm);
            onSourceClear?.();
        }
    }, [open, sourcePlan, onSourceClear]);

    const isDuplicate = sourcePlan != null;
    const isTemplate = isDuplicate && isSystemPlan(sourcePlan);

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault();
        setSubmitting(true);
        try {
            await submitCreatePlan(formData, onSuccess);
            setFormData(emptyForm);
            onOpenChange(false);
        } catch {
            // Error handled in submitCreatePlan
        } finally {
            setSubmitting(false);
        }
    };

    return (
        <Sheet open={open} onOpenChange={onOpenChange}>
            <SheetTrigger asChild>
                <Button
                    size="icon"
                    className="h-9 w-9 bg-[#1fc7d4] hover:bg-[#1fc7d4]/90"
                >
                    <Plus className="h-4 w-4" />
                </Button>
            </SheetTrigger>
            <SheetContent
                side="right"
                className="w-[400px] sm:w-[540px] bg-white dark:bg-card border-gray-200 dark:border-border text-white flex flex-col h-full"
            >
                <SheetHeader>
                    <SheetTitle>{isTemplate ? 'Create from Template' : isDuplicate ? 'Duplicate Plan' : 'Create Plan'}</SheetTitle>
                    <SheetDescription>
                        {isTemplate
                            ? `Create an editable admin plan based on "${sourcePlan.name}".`
                            : isDuplicate
                                ? `Create a new plan based on "${sourcePlan.name}".`
                                : 'Create a new access plan.'}
                    </SheetDescription>
                </SheetHeader>
                <form
                    onSubmit={handleSubmit}
                    className="space-y-6 pt-6 flex-1 flex flex-col overflow-y-auto"
                >
                    <div className="space-y-2">
                        <Label>Plan Name *</Label>
                        <Input
                            required
                            value={formData.name}
                            onChange={(e) =>
                                setFormData({ ...formData, name: e.target.value })
                            }
                            className="bg-white dark:bg-white/[0.04]"
                        />
                    </div>
                    <div className="space-y-2">
                        <Label>Display Group</Label>
                        <select
                            value={formData.plan_group}
                            onChange={(e) =>
                                setFormData({ ...formData, plan_group: e.target.value as PlanGroup })
                            }
                            className="w-full h-9 rounded-md border border-gray-200 dark:border-border bg-white dark:bg-white/[0.04] px-3 text-sm text-white"
                        >
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
                            <Input
                                type="number"
                                value={formData.priority}
                                onChange={(e) =>
                                    setFormData({
                                        ...formData,
                                        priority: parseInt(e.target.value),
                                    })
                                }
                                className="bg-white dark:bg-white/[0.04]"
                            />
                        </div>
                        <div className="space-y-2">
                            <Label>Price</Label>
                            <Input
                                type="number"
                                step="0.01"
                                value={formData.price}
                                onChange={(e) =>
                                    setFormData({
                                        ...formData,
                                        price: parseFloat(e.target.value),
                                    })
                                }
                                className="bg-white dark:bg-white/[0.04]"
                            />
                        </div>
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
                                    className="bg-white dark:bg-card border-gray-200 dark:border-border text-white max-w-[200px]"
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
                            value={formData.default_expiry_days}
                            onChange={(e) => {
                                const val = e.target.value;
                                 
                                if (val === '-' || val === '') {
                                    setFormData({
                                        ...formData,
                                        default_expiry_days: val as unknown as number,
                                    });
                                } else {
                                    const parsed = parseInt(val);
                                    if (!isNaN(parsed)) {
                                        setFormData({ ...formData, default_expiry_days: parsed });
                                    }
                                }
                            }}
                            className="bg-white dark:bg-white/[0.04]"
                            placeholder="-1 for permanent"
                        />
                    </div>
                    <div className="space-y-2">
                        <Label>Description</Label>
                        <Textarea
                            value={formData.description}
                            onChange={(e) =>
                                setFormData({ ...formData, description: e.target.value })
                            }
                            className="bg-white dark:bg-white/[0.04] min-h-[100px]"
                        />
                    </div>
                    <SheetFooter className="mt-auto pt-6">
                        <Button
                            type="submit"
                            disabled={submitting}
                            className="bg-[#1fc7d4] text-white w-full"
                        >
                            {submitting ? (
                                <Loader2 className="animate-spin w-4 h-4" />
                            ) : (
                                'Create Plan'
                            )}
                        </Button>
                    </SheetFooter>
                </form>
            </SheetContent>
        </Sheet>
    );
}
