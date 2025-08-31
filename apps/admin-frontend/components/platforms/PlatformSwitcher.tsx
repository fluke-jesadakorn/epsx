'use client';

import { useState } from 'react';
import { ChevronDown, Globe, Coins, Vote } from 'lucide-react';

interface Platform {
  id: string;
  code: string;
  name: string;
  description: string;
  icon: typeof Globe;
  baseUrl?: string;
}

interface PlatformSwitcherProps {
  currentPlatform: Platform;
  availablePlatforms: Platform[];
  userPlatformAccess: string[];
  onPlatformSwitch: (platformCode: string) => void;
}

const platformIcons: { [key: string]: typeof Globe } = {
  'epsx': Globe,
  'epsx-pay': Coins,
  'epsx-token': Vote,
};

const platformColors: { [key: string]: string } = {
  'epsx': 'bg-blue-500 text-white',
  'epsx-pay': 'bg-green-500 text-white',
  'epsx-token': 'bg-purple-500 text-white',
};

export function PlatformSwitcher({
  currentPlatform,
  availablePlatforms,
  userPlatformAccess,
  onPlatformSwitch,
}: PlatformSwitcherProps) {
  const [isOpen, setIsOpen] = useState(false);

  const handlePlatformSwitch = (platformCode: string) => {
    onPlatformSwitch(platformCode);
    setIsOpen(false);
  };

  const accessiblePlatforms = availablePlatforms.filter(platform =>
    userPlatformAccess.includes(platform.code)
  );

  const CurrentIcon = platformIcons[currentPlatform.code] || Globe;
  const currentColor = platformColors[currentPlatform.code] || 'bg-gray-500 text-white';

  return (
    <div className="relative">
      <button
        onClick={() => setIsOpen(!isOpen)}
        className={`inline-flex items-center gap-2 px-4 py-2 rounded-lg font-medium transition-colors hover:opacity-90 ${currentColor}`}
      >
        <CurrentIcon className="h-4 w-4" />
        <span className="hidden sm:inline">{currentPlatform.name}</span>
        <span className="sm:hidden">{currentPlatform.code.toUpperCase()}</span>
        <ChevronDown className="h-4 w-4" />
      </button>

      {isOpen && (
        <div className="absolute top-full left-0 mt-2 w-64 bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700 py-2 z-50">
          <div className="px-4 py-2 border-b border-gray-200 dark:border-gray-700">
            <h3 className="text-sm font-medium text-gray-900 dark:text-white">
              Switch Platform
            </h3>
            <p className="text-xs text-gray-500 dark:text-gray-400">
              Select a platform to manage
            </p>
          </div>
          
          {accessiblePlatforms.map((platform) => {
            const PlatformIcon = platformIcons[platform.code] || Globe;
            const isActive = platform.code === currentPlatform.code;
            const color = platformColors[platform.code] || 'bg-gray-500 text-white';
            
            return (
              <button
                key={platform.id}
                onClick={() => handlePlatformSwitch(platform.code)}
                className={`w-full flex items-center gap-3 px-4 py-3 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors ${
                  isActive ? 'bg-gray-50 dark:bg-gray-700' : ''
                }`}
              >
                <div className={`p-2 rounded-lg ${color} opacity-80`}>
                  <PlatformIcon className="h-4 w-4" />
                </div>
                <div className="flex-1 text-left">
                  <div className="text-sm font-medium text-gray-900 dark:text-white">
                    {platform.name}
                  </div>
                  <div className="text-xs text-gray-500 dark:text-gray-400">
                    {platform.description}
                  </div>
                </div>
                {isActive && (
                  <div className="w-2 h-2 bg-blue-500 rounded-full"></div>
                )}
              </button>
            );
          })}
          
          {accessiblePlatforms.length === 0 && (
            <div className="px-4 py-3 text-sm text-gray-500 dark:text-gray-400">
              No platforms accessible
            </div>
          )}
        </div>
      )}
      
      {/* Overlay to close dropdown */}
      {isOpen && (
        <div
          className="fixed inset-0 z-40"
          onClick={() => setIsOpen(false)}
        />
      )}
    </div>
  );
}