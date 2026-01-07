import { PlanSelection } from '@/components/plans/PlanSelection'
import { getCurrentUser } from '@/lib/server-actions'

export const dynamic = 'force-dynamic'

export default async function PlansPage() {
  const user = await getCurrentUser()

  return (
    <div className="min-h-screen bg-gradient-to-br from-slate-50 via-blue-50 to-indigo-50 dark:from-gray-900 dark:via-gray-900 dark:to-indigo-900">
      <div className="container mx-auto px-4 py-12">
        {/* Hero Section */}
        <div className="text-center mb-16">
          <h1 className="text-4xl md:text-6xl font-bold bg-gradient-to-r from-emerald-600 via-blue-600 to-purple-600 bg-clip-text text-transparent mb-6">
            Choose Your EPSX Plan
          </h1>
          <p className="text-xl text-gray-600 dark:text-gray-300 max-w-3xl mx-auto leading-relaxed">
            Unlock powerful analytics features, API access, and premium tools to supercharge your analytics experience
          </p>
        </div>

        {/* Plan Selection Component */}
        <PlanSelection currentUser={user} />

        {/* FAQ Section */}
        <div className="mt-20">
          <h2 className="text-3xl font-bold text-center text-gray-900 dark:text-white mb-12">
            Frequently Asked Questions
          </h2>
          <div className="max-w-3xl mx-auto space-y-6">
            <div className="bg-white dark:bg-gray-800 rounded-2xl p-6 shadow-lg">
              <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-3">
                Can I change my plan later?
              </h3>
              <p className="text-gray-600 dark:text-gray-300">
                Yes! You can upgrade or downgrade your plan at any time. Changes take effect immediately,
                and we'll prorate any billing adjustments.
              </p>
            </div>

            <div className="bg-white dark:bg-gray-800 rounded-2xl p-6 shadow-lg">
              <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-3">
                What happens to my API keys when I change plans?
              </h3>
              <p className="text-gray-600 dark:text-gray-300">
                Your API keys remain valid when upgrading. If downgrading removes API access,
                we'll notify you 7 days in advance so you can adjust your integrations.
              </p>
            </div>

            <div className="bg-white dark:bg-gray-800 rounded-2xl p-6 shadow-lg">
              <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-3">
                Do you offer custom enterprise plans?
              </h3>
              <p className="text-gray-600 dark:text-gray-300">
                Absolutely! We can create custom plans with specific features, higher limits,
                and dedicated support. <a href="/contact" className="text-emerald-600 hover:underline">Contact us</a> to discuss your needs.
              </p>
            </div>

            <div className="bg-white dark:bg-gray-800 rounded-2xl p-6 shadow-lg">
              <h3 className="text-lg font-semibold text-gray-900 dark:text-white mb-3">
                Is there a free trial?
              </h3>
              <p className="text-gray-600 dark:text-gray-300">
                We offer a 7-day free trial for all premium plans. No credit card required -
                just sign up and start exploring advanced features immediately.
              </p>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}