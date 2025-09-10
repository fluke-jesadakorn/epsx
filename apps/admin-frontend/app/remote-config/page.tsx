import { RemoteConfigDashboard } from '@/components/remote-config/RemoteConfigDashboard'

export default function RemoteConfigPage() {
  return (
    <div className="container mx-auto py-6">
      <RemoteConfigDashboard />
    </div>
  )
}

export const metadata = {
  title: 'Remote Config Management - EPSX Admin',
  description: 'Manage Firebase Remote Config parameters, user targeting, and A/B testing experiments'
}