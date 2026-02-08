'use client'

import { PlanFormProps } from '@/components/plans/edit/types'
import * as Promo from '@/shared/utils/promo'

export function PlanPromotions({ formData, setFormData }: PlanFormProps) {
    const status = Promo.getStatus(
        formData.promo_enabled,
        formData.promo_start,
        formData.promo_end
    )

    return (
        <div className="bg-gradient-to-r from-rose-50 to-red-50 dark:from-rose-900/20 dark:to-red-900/20 rounded-2xl p-6">
            <div className="flex items-center justify-between mb-4">
                <h3 className="text-lg font-bold text-gray-900 dark:text-white">
                    Promotion & Discounts
                </h3>
                {formData.promo_enabled &&
                    formData.promo_start &&
                    formData.promo_end && (
                        <div
                            className={`px-4 py-2 rounded-xl font-semibold ${Promo.getStatusColor(status)}`}
                        >
                            {Promo.getStatusIcon(status)} {Promo.getStatusText(status)}
                        </div>
                    )}
            </div>

            <div className="mb-6">
                <label className="flex items-center gap-3 cursor-pointer">
                    <input
                        type="checkbox"
                        checked={formData.promo_enabled}
                        onChange={(e) =>
                            setFormData({ ...formData, promo_enabled: e.target.checked })
                        }
                        className="w-6 h-6 rounded border-2 border-gray-300 dark:border-gray-600 text-rose-500 focus:ring-2 focus:ring-rose-500"
                    />
                    <span className="text-sm font-semibold text-gray-700 dark:text-gray-300">
                        Enable Promotion
                    </span>
                </label>
            </div>

            {formData.promo_enabled && (
                <div className="space-y-4">
                    <div>
                        <label className="block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
                            Discount Type
                        </label>
                        <div className="grid grid-cols-2 gap-4">
                            <button
                                type="button"
                                onClick={() =>
                                    setFormData({ ...formData, promo_type: 'percentage' })
                                }
                                className={`p-4 rounded-xl border-2 font-semibold ${formData.promo_type === 'percentage' ? 'border-rose-500 bg-rose-50 dark:bg-rose-900/20 text-rose-700 dark:text-rose-300' : 'border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300'}`}
                            >
                                % Percentage
                            </button>
                            <button
                                type="button"
                                onClick={() =>
                                    setFormData({ ...formData, promo_type: 'fixed' })
                                }
                                className={`p-4 rounded-xl border-2 font-semibold ${formData.promo_type === 'fixed' ? 'border-rose-500 bg-rose-50 dark:bg-rose-900/20 text-rose-700 dark:text-rose-300' : 'border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300'}`}
                            >
                                $ Fixed Amount
                            </button>
                        </div>
                    </div>

                    <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                        <div>
                            <label className="block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
                                {formData.promo_type === 'percentage'
                                    ? 'Discount (%)'
                                    : 'Discount Amount ($)'}
                            </label>
                            <input
                                type="number"
                                step={formData.promo_type === 'percentage' ? '1' : '0.01'}
                                min="0"
                                max={
                                    formData.promo_type === 'percentage' ? '100' : undefined
                                }
                                value={formData.promo_value}
                                onChange={(e) => {
                                    const value = parseFloat(e.target.value) || 0
                                    const newValue =
                                        formData.promo_type === 'percentage'
                                            ? Math.min(value, 100)
                                            : value
                                    setFormData({ ...formData, promo_value: newValue })
                                }}
                                className="w-full px-4 py-3 rounded-xl border-2 border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:border-rose-500 focus:ring-2 focus:ring-rose-500 focus:outline-none"
                            />
                        </div>
                        <div>
                            <label className="block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
                                Final Promotional Price ($)
                            </label>
                            <input
                                type="number"
                                step="0.01"
                                min="0"
                                value={formData.promo_price}
                                onChange={(e) =>
                                    setFormData({
                                        ...formData,
                                        promo_price: parseFloat(e.target.value) || 0,
                                    })
                                }
                                className="w-full px-4 py-3 rounded-xl border-2 border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:border-rose-500 focus:ring-2 focus:ring-rose-500 focus:outline-none"
                            />
                            <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                                Auto: $
                                {Promo.calcPrice(
                                    formData.current_price,
                                    formData.promo_type,
                                    formData.promo_value
                                ).toFixed(2)}
                            </p>
                        </div>
                    </div>

                    <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                        <div>
                            <label className="block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
                                Start Date
                            </label>
                            <input
                                type="datetime-local"
                                value={formData.promo_start}
                                onChange={(e) =>
                                    setFormData({ ...formData, promo_start: e.target.value })
                                }
                                className="w-full px-4 py-3 rounded-xl border-2 border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:border-rose-500 focus:ring-2 focus:ring-rose-500 focus:outline-none"
                            />
                        </div>
                        <div>
                            <label className="block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
                                End Date
                            </label>
                            <input
                                type="datetime-local"
                                value={formData.promo_end}
                                onChange={(e) =>
                                    setFormData({ ...formData, promo_end: e.target.value })
                                }
                                className="w-full px-4 py-3 rounded-xl border-2 border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:border-rose-500 focus:ring-2 focus:ring-rose-500 focus:outline-none"
                            />
                        </div>
                    </div>
                </div>
            )}
        </div>
    )
}
