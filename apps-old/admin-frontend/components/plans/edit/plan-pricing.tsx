'use client'

import type { PlanFormProps } from '@/components/plans/edit/types'

export function PlanPricing({ formData, setFormData }: PlanFormProps) {
    return (
        <div className="bg-gradient-to-r from-green-50 to-emerald-50 dark:from-green-900/20 dark:to-emerald-900/20 rounded-2xl p-6">
            <h3 className="text-lg font-bold text-foreground mb-4">Pricing</h3>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div>
                    <label className="block text-sm font-semibold text-gray-700 dark:text-muted-foreground mb-2">
                        Price (USD)
                    </label>
                    <input
                        type="number"
                        step="0.01"
                        min="0"
                        value={formData.current_price}
                        onChange={(e) =>
                            setFormData({
                                ...formData,
                                current_price: Number(e.target.value),
                            })
                        }
                        className="w-full px-4 py-3 rounded-xl border-2 border-border/40 bg-card text-foreground focus:border-green-500 focus:ring-2 focus:ring-green-500 focus:outline-none"
                        required
                    />
                </div>
                <div>
                    <label className="block text-sm font-semibold text-gray-700 dark:text-muted-foreground mb-2">
                        Status
                    </label>
                    <div className="flex items-center gap-3 p-4 bg-card rounded-xl border-2 border-border/40">
                        <input
                            type="checkbox"
                            id="is_active"
                            checked={formData.is_active}
                            onChange={(e) =>
                                setFormData({ ...formData, is_active: e.target.checked })
                            }
                            className="w-6 h-6 rounded border-2 border-border/40 text-green-500 focus:ring-2 focus:ring-green-500"
                        />
                        <label
                            htmlFor="is_active"
                            className="text-sm font-semibold text-gray-700 dark:text-muted-foreground cursor-pointer"
                        >
                            {formData.is_active ? 'Active' : 'Inactive'}
                        </label>
                    </div>
                </div>
            </div>
        </div>
    )
}
