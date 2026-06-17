'use client';

export default function UpgradeButton() {
  return (
    <button 
      className="px-6 py-2 bg-gradient-to-r from-orange-500 to-yellow-500 text-white rounded-xl font-medium hover:shadow-lg"
      onClick={() => window.location.href = '/billing'}
    >
      Upgrade Plan
    </button>
  );
}