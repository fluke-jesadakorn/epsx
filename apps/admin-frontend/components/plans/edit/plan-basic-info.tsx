'use client'

import { PlanFormProps } from '@/components/plans/edit/types'

export function PlanBasicInfo({ formData, setFormData }: PlanFormProps) {
    return (
        <div className="bg-gradient-to-r from-blue-50 to-indigo-50 dark:from-blue-900/20 dark:to-indigo-900/20 rounded-2xl p-6">
            <h3 className="text-lg font-bold text-gray-900 dark:text-white mb-4">Basic Information</h3>
            <div className="space-y-4">
                <div>
                    <label className="block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
                        Plan Name
                    </label>
                    <input
                        type="text"
                        value={formData.name}
                        onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                        className="w-full px-4 py-3 rounded-xl border-2 border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:border-emerald-500 focus:ring-2 focus:ring-emerald-500 focus:outline-none"
                        required
                    />
                </div>
                <div>
                    <label className="block text-sm font-semibold text-gray-700 dark:text-gray-300 mb-2">
                        Description
                    </label>
                    <textarea
                        value={formData.description}
                        onChange={(e) => setFormData({ ...formData, description: e.target.value })}
                        rows={3}
                        className="w-full px-4 py-3 rounded-xl border-2 border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-800 text-gray-900 dark:text-white focus:border-emerald-500 focus:ring-2 focus:ring-emerald-500 focus:outline-none"
                    />
                </div>
            </div>
        </div>
    )
}
