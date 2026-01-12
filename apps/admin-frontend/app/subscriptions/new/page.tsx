'use client'

import { ArrowLeft } from 'lucide-react'
import Link from 'next/link'
import { useRouter } from 'next/navigation'
import { useEffect, useState } from 'react'

import { toast } from '@/hooks/use-toast'
import { logger } from '@/lib/logger'
import { createPlansClient, type CreateSubscriptionRequest, isApiSuccess } from '@/shared/api/plans'
import { createAdminApiClient } from '@/shared/utils/api-client'

/**
 *
 */
export default function NewSubscriptionPage() {
    const router = useRouter()
    const [loading, setLoading] = useState(false)
    const [plans, setPlans] = useState<any[]>([])
    const [formData, setFormData] = useState<CreateSubscriptionRequest>({
        user_id: '',
        plan_id: 0,
        access_context: 'internal',
        api_key_name: '',
        expires_at: '',
        auto_renew: true,
        metadata: {}
    })
    const [showApiKeyField, setShowApiKeyField] = useState(false)

    useEffect(() => {
        loadPlans()
    }, [])

    const loadPlans = async () => {
        const adminClient = createPlansClient(createAdminApiClient())
        try {
            const response = await adminClient.getPlans({ is_active: true })
            if (isApiSuccess(response)) {
                setPlans((response.data as any)?.plans || response.data as any || [])
            }
        } catch (_error) {
            logger.error('Failed to load plans', { _error })
        }
    }

    const handleSubmit = async (e: React.FormEvent) => {
        e.preventDefault()

        if (!formData.user_id.trim()) {
            toast({
                title: "Error",
                description: "User ID is required",
                variant: "destructive"
            })
            return
        }

        if (formData.plan_id === 0) {
            toast({
                title: "Error",
                description: "Please select a plan",
                variant: "destructive"
            })
            return
        }

        if ((formData.access_context === 'external' || formData.access_context === 'both') && !formData.api_key_name?.trim()) {
            toast({
                title: "Error",
                description: "API key name is required for external access",
                variant: "destructive"
            })
            return
        }

        const adminClient = createPlansClient(createAdminApiClient())
        try {
            setLoading(true)
            const subscriptionData: CreateSubscriptionRequest = {
                ...formData,
                api_key_name: showApiKeyField ? formData.api_key_name : undefined,
                expires_at: formData.expires_at || undefined
            }

            const response = await adminClient.createSubscription(subscriptionData)

            if (isApiSuccess(response)) {
                toast({
                    title: "Success",
                    description: "Subscription created successfully",
                })
                router.push('/subscriptions')
            } else {
                toast({
                    title: "Error",
                    description: response.error || "Failed to create subscription",
                    variant: "destructive"
                })
            }
        } catch (_error) {
            toast({
                title: "Error",
                description: "Failed to create subscription",
                variant: "destructive"
            })
        } finally {
            setLoading(false)
        }
    }

    const handleAccessContextChange = (context: string) => {
        setFormData({ ...formData, access_context: context })
        setShowApiKeyField(context === 'external' || context === 'both')
    }

    // Generate default expiry date (1 year from now)
    const defaultExpiryDate = new Date()
    defaultExpiryDate.setFullYear(defaultExpiryDate.getFullYear() + 1)
    const defaultExpiryString = defaultExpiryDate.toISOString().split('T')[0]

    return (
        <div className="min-h-screen p-6">
            <div className="max-w-4xl mx-auto">
                {/* Header */}
                <div className="flex items-center gap-4 mb-8">
                    <Link
                        href="/subscriptions"
                        className="p-2 rounded-xl bg-card border border-border/50 hover:border-primary/50 transition-colors shadow-sm"
                    >
                        <ArrowLeft className="h-5 w-5" />
                    </Link>
                    <h1 className="text-2xl font-bold bg-gradient-to-r from-primary via-secondary to-primary bg-clip-text text-transparent">
                        Create New Subscription
                    </h1>
                </div>

                <div className="bg-card rounded-3xl p-8 border border-border/50 shadow-sm relative overflow-hidden transition-all">
                    {/* Subtle decoration */}
                    <div className="absolute top-0 right-0 w-32 h-32 bg-primary/5 rounded-full blur-3xl -mr-16 -mt-16"></div>

                    <form onSubmit={handleSubmit} className="relative space-y-6">
                        {/* User and Plan Selection */}
                        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                            <div>
                                <label className="block text-sm font-semibold text-muted-foreground mb-2">
                                    User ID *
                                </label>
                                <input
                                    type="text"
                                    value={formData.user_id}
                                    onChange={(e) => setFormData({ ...formData, user_id: e.target.value })}
                                    className="w-full px-4 py-3 rounded-xl border border-border bg-card text-foreground focus:ring-2 focus:ring-primary outline-none transition-all"
                                    placeholder="Enter user ID"
                                    required
                                />
                                <p className="text-xs text-muted-foreground/70 mt-1">
                                    The wallet address or email of the user
                                </p>
                            </div>

                            <div>
                                <label className="block text-sm font-semibold text-muted-foreground mb-2">
                                    Plan *
                                </label>
                                <select
                                    value={formData.plan_id}
                                    onChange={(e) => setFormData({ ...formData, plan_id: parseInt(e.target.value) })}
                                    className="w-full px-4 py-3 rounded-xl border border-border bg-card text-foreground focus:ring-2 focus:ring-primary outline-none transition-all"
                                    required
                                >
                                    <option value={0}>Select a plan</option>
                                    {plans.map(plan => (
                                        <option key={plan.id} value={plan.id}>
                                            {plan.name} - {Number(plan.current_price) === 0 ? 'Free' : `$${plan.current_price} ${plan.currency}`} ({plan.plan_category})
                                        </option>
                                    ))}
                                </select>
                            </div>
                        </div>

                        {/* Access Context */}
                        <div>
                            <label className="block text-sm font-semibold text-muted-foreground mb-4">
                                Access Context *
                            </label>
                            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                                {[
                                    { value: 'internal', label: 'Internal', desc: 'Web app access only', icon: '🖥️' },
                                    { value: 'external', label: 'External', desc: 'API access with key', icon: '🔧' },
                                    { value: 'both', label: 'Both', desc: 'Web + API access', icon: '🔄' }
                                ].map(option => (
                                    <label
                                        key={option.value}
                                        className={`relative flex flex-col p-4 rounded-xl border-2 cursor-pointer transition-all ${formData.access_context === option.value
                                            ? 'border-primary bg-primary/5'
                                            : 'border-border hover:border-border/80 bg-muted/30'
                                            }`}
                                    >
                                        <input
                                            type="radio"
                                            name="access_context"
                                            value={option.value}
                                            checked={formData.access_context === option.value}
                                            onChange={(e) => handleAccessContextChange(e.target.value)}
                                            className="sr-only"
                                        />
                                        <div className="flex items-center gap-3 mb-2">
                                            <span className="text-2xl">{option.icon}</span>
                                            <span className={`font-semibold ${formData.access_context === option.value ? 'text-primary' : 'text-foreground'}`}>
                                                {option.label}
                                            </span>
                                        </div>
                                        <span className="text-sm text-muted-foreground">{option.desc}</span>
                                    </label>
                                ))}
                            </div>
                        </div>

                        {/* API Key Name (conditional) */}
                        {showApiKeyField && (
                            <div className="animate-in fade-in slide-in-from-top-2 duration-300">
                                <label className="block text-sm font-semibold text-muted-foreground mb-2">
                                    API Key Name *
                                </label>
                                <input
                                    type="text"
                                    value={formData.api_key_name}
                                    onChange={(e) => setFormData({ ...formData, api_key_name: e.target.value })}
                                    className="w-full px-4 py-3 rounded-xl border border-border bg-card text-foreground focus:ring-2 focus:ring-primary outline-none transition-all"
                                    placeholder="e.g., Production API Key"
                                    required
                                />
                                <p className="text-xs text-muted-foreground/70 mt-1">
                                    A descriptive name for the API key
                                </p>
                            </div>
                        )}

                        {/* Subscription Settings */}
                        <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                            <div>
                                <label className="block text-sm font-semibold text-muted-foreground mb-2">
                                    Expiry Date (Optional)
                                </label>
                                <input
                                    type="date"
                                    value={formData.expires_at}
                                    onChange={(e) => setFormData({ ...formData, expires_at: e.target.value })}
                                    min={new Date().toISOString().split('T')[0]}
                                    className="w-full px-4 py-3 rounded-xl border border-border bg-card text-foreground focus:ring-2 focus:ring-primary outline-none transition-all"
                                />
                                <p className="text-xs text-muted-foreground/70 mt-1">
                                    Leave empty for no expiry. Default: {defaultExpiryString}
                                </p>
                            </div>

                            <div className="flex items-center justify-center">
                                <label className="flex items-center gap-3 cursor-pointer group">
                                    <div className="relative">
                                        <input
                                            type="checkbox"
                                            checked={formData.auto_renew}
                                            onChange={(e) => setFormData({ ...formData, auto_renew: e.target.checked })}
                                            className="w-5 h-5 text-primary border-border rounded focus:ring-primary"
                                        />
                                    </div>
                                    <div>
                                        <span className="text-sm font-semibold text-foreground group-hover:text-primary transition-colors">Auto-renew</span>
                                        <p className="text-xs text-muted-foreground">Automatically renew when expired</p>
                                    </div>
                                </label>
                            </div>
                        </div>

                        {/* Advanced Settings */}
                        <div className="bg-muted/30 p-4 rounded-xl border border-border/50">
                            <h3 className="font-semibold text-foreground mb-3">Advanced Settings</h3>
                            <div>
                                <label className="block text-sm font-semibold text-muted-foreground mb-2">
                                    Metadata (JSON)
                                </label>
                                <textarea
                                    value={JSON.stringify(formData.metadata, null, 2)}
                                    onChange={(e) => {
                                        try {
                                            const parsed = JSON.parse(e.target.value || '{}')
                                            setFormData({ ...formData, metadata: parsed })
                                        } catch {
                                            // Invalid JSON - ignore for now
                                        }
                                    }}
                                    rows={3}
                                    className="w-full px-4 py-3 rounded-xl border border-border bg-card text-foreground focus:ring-2 focus:ring-primary outline-none font-mono text-sm resize-none"
                                    placeholder='{"custom_field": "value"}'
                                />
                                <p className="text-xs text-muted-foreground/70 mt-1">
                                    Custom metadata in JSON format
                                </p>
                            </div>
                        </div>

                        {/* Submit Buttons */}
                        <div className="flex gap-4 pt-6">
                            <Link href="/subscriptions" className="flex-1">
                                <button
                                    type="button"
                                    className="w-full px-6 py-3 rounded-xl border border-border text-foreground font-semibold hover:bg-muted transition-colors"
                                >
                                    Cancel
                                </button>
                            </Link>
                            <button
                                type="submit"
                                disabled={loading}
                                className="flex-1 px-6 py-3 rounded-xl bg-primary text-primary-foreground font-semibold hover:opacity-90 disabled:opacity-50 disabled:cursor-not-allowed transition-opacity shadow-lg shadow-primary/20"
                            >
                                {loading ? 'Creating...' : 'Create Subscription'}
                            </button>
                        </div>
                    </form>
                </div>
            </div>
        </div>
    )
}
