'use client';

export function CardDashboardViewSimple() {
  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
        <div>
          <h2 className="text-2xl font-bold bg-gradient-to-r from-orange-500 to-yellow-500 bg-clip-text text-transparent">
            📋 Performance Monitor
          </h2>
          <p className="text-gray-600 dark:text-gray-300">
            Analytics dashboard is loading...
          </p>
        </div>
      </div>

      {/* Simple Cards grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6">
        {[1, 2, 3, 4].map((item) => (
          <div key={item} className="bg-white rounded-lg border shadow-sm p-6">
            <h3 className="text-xl font-bold text-gray-900 mb-2">
              Sample Stock {item}
            </h3>
            <p className="text-sm text-gray-600 mb-4">
              This is a test card to verify the grid layout is working properly.
            </p>
            <div className="space-y-2">
              <div className="flex justify-between">
                <span className="text-sm text-gray-500">Current:</span>
                <span className="text-sm font-medium">$100.00</span>
              </div>
              <div className="flex justify-between">
                <span className="text-sm text-gray-500">Growth:</span>
                <span className="text-sm font-medium text-green-600">+5.0%</span>
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}