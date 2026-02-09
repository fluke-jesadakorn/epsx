'use client'

import type { PlanFormProps } from '@/components/plans/edit/types'

export function PlanApiLimits({ formData, setFormData }: PlanFormProps) {
    return (
        <div className="bg-gradient-to-r from-purple-50 to-pink-50 dark:from-purple-900/20 dark:to-pink-900/20 rounded-2xl p-6">
            <h3 className="text-lg font-bold text-gray-900 dark:text-white mb-4">
                API Limitations
            </h3>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div>
                    <label className="block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
                        API Calls Limit (per month)
                    </label>
                    <input
                        type="number"
                        min="-1"
                        value={formData.api_calls_limit}
                        onChange={(e) =>
                            setFormData({
                                ...formData,
                                api_calls_limit: parseInt(e.target.value) ?? 0,
                            })
                        }
                        className="w-full px-4 py-3 rounded-xl border-2 border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:border-purple-500 focus:ring-2 focus:ring-purple-500 focus:outline-none"
                    />
                    <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                        -1 = unlimited, 0 = not granted
                    </p>
                </div>
                <div>
                    <label className="block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
                        Ranking Offset (Premium Ranks)
                    </label>
                    <input
                        type="number"
                        min="0"
                        value={formData.ranking_offset}
                        onChange={(e) =>
                            setFormData({
                                ...formData,
                                ranking_offset: parseInt(e.target.value) ?? 0,
                            })
                        }
                        className="w-full px-4 py-3 rounded-xl border-2 border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:border-purple-500 focus:ring-2 focus:ring-purple-500 focus:outline-none"
                    />
                    <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                        Number of top ranks locked. 0 = full access.
                    </p>
                </div>
                <div>
                    <label className="block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
                        Analytics Queries (per month)
                    </label>
                    <input
                        type="number"
                        min="-1"
                        value={formData.analytics_queries}
                        onChange={(e) =>
                            setFormData({
                                ...formData,
                                analytics_queries: parseInt(e.target.value) ?? 0,
                            })
                        }
                        className="w-full px-4 py-3 rounded-xl border-2 border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:border-purple-500 focus:ring-2 focus:ring-purple-500 focus:outline-none"
                    />
                    <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                        -1 = unlimited, 0 = not granted
                    </p>
                </div>
                <div>
                    <label className="block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
                        Export Limit (per day)
                    </label>
                    <input
                        type="number"
                        min="-1"
                        value={formData.export_limit}
                        onChange={(e) =>
                            setFormData({
                                ...formData,
                                export_limit: parseInt(e.target.value) ?? 0,
                            })
                        }
                        className="w-full px-4 py-3 rounded-xl border-2 border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:border-purple-500 focus:ring-2 focus:ring-purple-500 focus:outline-none"
                    />
                    <p className="text-xs text-gray-500 dark:text-gray-400 mt-1">
                        -1 = unlimited, 0 = not granted
                    </p>
                </div>
            </div>

            <div className="mt-4">
                <label className="flex items-center gap-3 cursor-pointer">
                    <input
                        type="checkbox"
                        checked={formData.premium_features}
                        onChange={(e) =>
                            setFormData({ ...formData, premium_features: e.target.checked })
                        }
                        className="w-6 h-6 rounded border-2 border-gray-300 dark:border-gray-600 text-purple-500 focus:ring-2 focus:ring-purple-500"
                    />
                    <span className="text-sm font-semibold text-gray-700 dark:text-gray-300">
                        Enable Premium Features (Advanced Trading, Premium Analytics)
                    </span>
                </label>
            </div>
        </div>
    )
}
