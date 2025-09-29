export default function AdminLoginPage() {
  return (
    <div className="container mx-auto p-6">
      <div className="max-w-2xl mx-auto text-center space-y-6">
        <h1 className="text-3xl font-bold text-gray-900 dark:text-gray-100">
          🌐 Admin Dashboard
        </h1>
        
        <div className="bg-white/90 dark:bg-gray-800/90 backdrop-blur-sm rounded-xl p-6 shadow-lg">
          <p className="text-gray-600 dark:text-gray-300 text-lg mb-4">
            Use the navigation bar above to manage your wallet connection.
          </p>
          
          <p className="text-sm text-gray-500 dark:text-gray-400">
            You can disconnect your wallet using the wallet button in the top navigation bar.
          </p>
        </div>
      </div>
    </div>
  );
}