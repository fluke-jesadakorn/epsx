import { useAccount } from 'wagmi';

export default function BillingDashboard() {
  const { address } = useAccount();

  const user = address ? { address } : null;
  
  return (
    <div className="min-h-screen bg-gray-50 p-6">
      <div className="max-w-4xl mx-auto">
        <div className="bg-white rounded-lg shadow p-8 text-center">
          <div className="mb-6">
            <h1 className="text-3xl font-bold text-gray-900 mb-2">Billing Dashboard</h1>
            <p className="text-gray-600">
              Backend billing endpoints are not implemented yet
            </p>
          </div>
          
          {user && (
            <div className="bg-blue-50 border border-blue-200 rounded-lg p-6 mb-6">
              <div className="text-left">
                <h3 className="font-medium text-gray-900 mb-2">Current Wallet</h3>
                <p className="text-sm text-gray-600 font-mono">
                  {user.address?.slice(0, 8)}...{user.address?.slice(-4)}
                </p>
                <p className="text-sm text-gray-600 mt-1">
                  Access managed by backend
                </p>
              </div>
            </div>
          )}
          
          <div className="text-left bg-yellow-50 border border-yellow-200 rounded-lg p-6">
            <h3 className="font-medium text-gray-900 mb-2">Development Note</h3>
            <p className="text-sm text-gray-600">
              The backend currently only implements Web3 wallet authentication and permission endpoints. 
              Enterprise billing features are not yet available.
            </p>
            <ul className="text-sm text-gray-600 mt-2 list-disc list-inside space-y-1">
              <li>Subscription management</li>
              <li>Payment history</li>
              <li>API key generation</li>
              <li>Usage analytics</li>
            </ul>
          </div>
        </div>
      </div>
    </div>
  );
}