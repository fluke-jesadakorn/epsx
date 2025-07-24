'use client';

import { useTheme, ThemeVariantSelector, DarkModeToggle } from '@/packages/ui';

export function ThemeControls() {
  return (
    <div>
      <h3 className="text-lg font-semibold mb-4">Theme & Appearance</h3>
      <div className="space-y-4">
        <div className="flex items-center justify-between p-4 border rounded-lg">
          <div className="flex items-center gap-3">
            <div className="w-8 h-8 bg-purple-100 rounded-full flex items-center justify-center">
              🎨
            </div>
            <div>
              <p className="font-medium">Theme Variant</p>
              <p className="text-sm text-muted-foreground">Choose your preferred theme style</p>
            </div>
          </div>
          <ThemeVariantSelector />
        </div>
        
        <div className="flex items-center justify-between p-4 border rounded-lg">
          <div className="flex items-center gap-3">
            <div className="w-8 h-8 bg-gray-100 rounded-full flex items-center justify-center">
              🌙
            </div>
            <div>
              <p className="font-medium">Dark Mode</p>
              <p className="text-sm text-muted-foreground">Toggle between light and dark appearance</p>
            </div>
          </div>
          <DarkModeToggle showLabel />
        </div>
      </div>
    </div>
  );
}