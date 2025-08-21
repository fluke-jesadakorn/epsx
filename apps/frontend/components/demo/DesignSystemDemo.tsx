/**
 * Design System Demo Component
 * 
 * This component demonstrates the new CVA-based design system and shows
 * how to migrate from the old CSS classes to the new type-safe variants.
 * 
 * Before (old approach):
 * - className="btn-pancake-primary"
 * - className="card-pancake p-8"
 * - className="pancake-gradient-text"
 * 
 * After (new approach):
 * - buttonVariants({ variant: 'primary', size: 'lg' })
 * - cardVariants({ variant: 'pancake', padding: 'lg' })
 * - gradientTextVariants({ gradient: 'primary' })
 */

'use client';

import React from 'react';
import { 
  buttonVariants, 
  cardVariants, 
  badgeVariants,
  gradientTextVariants,
  animationVariants,
  tokens,
  createButtonClass,
  createCardClass,
  commonClasses,
} from '@/design-system';
import { cn } from '@/lib/utils';

export function DesignSystemDemo() {
  return (
    <div className="w-full max-w-6xl mx-auto p-8 space-y-12">
      <header className="text-center space-y-4">
        <h1 className={cn(
          gradientTextVariants({ gradient: 'primary' }),
          'text-4xl md:text-6xl font-black'
        )}>
          EPSX Design System
        </h1>
        <p className="text-lg text-gray-600 dark:text-gray-400 max-w-2xl mx-auto">
          A unified, type-safe design system that replaces fragmented CSS with 
          composable, performant components.
        </p>
      </header>

      {/* Button Variants Section */}
      <section className={createCardClass({ variant: 'default', padding: 'lg' })}>
        <h2 className="text-2xl font-bold mb-6">Button Variants</h2>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          
          {/* Primary Button */}
          <div className="space-y-2">
            <button className={buttonVariants({ variant: 'primary', size: 'lg' })}>
              Primary Button
            </button>
            <code className="text-xs block bg-gray-100 dark:bg-gray-800 p-2 rounded">
              {`buttonVariants({ variant: 'primary', size: 'lg' })`}
            </code>
          </div>

          {/* Secondary Button */}
          <div className="space-y-2">
            <button className={buttonVariants({ variant: 'secondary', size: 'lg' })}>
              Secondary Button
            </button>
            <code className="text-xs block bg-gray-100 dark:bg-gray-800 p-2 rounded">
              {`buttonVariants({ variant: 'secondary', size: 'lg' })`}
            </code>
          </div>

          {/* Success Button with Glow */}
          <div className="space-y-2">
            <button className={buttonVariants({ 
              variant: 'success', 
              size: 'lg', 
              glow: true 
            })}>
              Success + Glow
            </button>
            <code className="text-xs block bg-gray-100 dark:bg-gray-800 p-2 rounded">
              {`buttonVariants({ variant: 'success', glow: true })`}
            </code>
          </div>

          {/* Outline Button */}
          <div className="space-y-2">
            <button className={buttonVariants({ variant: 'outline', size: 'lg' })}>
              Outline Button
            </button>
            <code className="text-xs block bg-gray-100 dark:bg-gray-800 p-2 rounded">
              {`buttonVariants({ variant: 'outline', size: 'lg' })`}
            </code>
          </div>

          {/* Custom Button with Additional Classes */}
          <div className="space-y-2">
            <button className={createButtonClass(
              { variant: 'primary', size: 'lg', rounded: 'full' },
              'shadow-2xl transform hover:rotate-1'
            )}>
              Custom Enhanced
            </button>
            <code className="text-xs block bg-gray-100 dark:bg-gray-800 p-2 rounded">
              {`createButtonClass({ variant: 'primary', rounded: 'full' }, 'custom-classes')`}
            </code>
          </div>

        </div>
      </section>

      {/* Card Variants Section */}
      <section className={createCardClass({ variant: 'default', padding: 'lg' })}>
        <h2 className="text-2xl font-bold mb-6">Card Variants</h2>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          
          {/* Default Card */}
          <div className={cardVariants({ variant: 'default', padding: 'md' })}>
            <h3 className="font-semibold mb-2">Default Card</h3>
            <p className="text-sm text-gray-600 dark:text-gray-400">
              Basic card with hover effects and clean styling.
            </p>
            <code className="text-xs block bg-gray-100 dark:bg-gray-800 p-2 rounded mt-3">
              {`cardVariants({ variant: 'default' })`}
            </code>
          </div>

          {/* PancakeSwap Style Card */}
          <div className={cardVariants({ 
            variant: 'pancake', 
            padding: 'md', 
            glow: true,
            interactive: true
          })}>
            <h3 className="font-semibold mb-2">PancakeSwap Card</h3>
            <p className="text-sm text-gray-600 dark:text-gray-400">
              Enhanced card with gradient background and glow effects.
            </p>
            <code className="text-xs block bg-gray-100 dark:bg-gray-800 p-2 rounded mt-3">
              {`cardVariants({ variant: 'pancake', glow: true, interactive: true })`}
            </code>
          </div>

          {/* Glass Card */}
          <div className={cardVariants({ variant: 'glass', padding: 'md' })}>
            <h3 className="font-semibold mb-2">Glass Card</h3>
            <p className="text-sm text-gray-600 dark:text-gray-400">
              Glassmorphism effect with backdrop blur.
            </p>
            <code className="text-xs block bg-gray-100 dark:bg-gray-800 p-2 rounded mt-3">
              {`cardVariants({ variant: 'glass' })`}
            </code>
          </div>

        </div>
      </section>

      {/* Text Variants Section */}
      <section className={createCardClass({ variant: 'default', padding: 'lg' })}>
        <h2 className="text-2xl font-bold mb-6">Text & Badge Variants</h2>
        
        <div className="space-y-6">
          
          {/* Gradient Text Examples */}
          <div>
            <h3 className="text-lg font-semibold mb-4">Gradient Text</h3>
            <div className="space-y-2">
              <h4 className={gradientTextVariants({ 
                gradient: 'primary', 
                animation: 'normal' 
              })}>
                Primary Gradient Text
              </h4>
              <h4 className={gradientTextVariants({ 
                gradient: 'secondary', 
                animation: 'slow' 
              })}>
                Secondary Gradient Text
              </h4>
              <h4 className={gradientTextVariants({ 
                gradient: 'rainbow', 
                animation: 'fast' 
              })}>
                Rainbow Gradient Text
              </h4>
            </div>
          </div>

          {/* Badge Examples */}
          <div>
            <h3 className="text-lg font-semibold mb-4">Badges</h3>
            <div className="flex flex-wrap gap-2">
              <span className={badgeVariants({ variant: 'primary' })}>
                Primary
              </span>
              <span className={badgeVariants({ variant: 'success' })}>
                Success
              </span>
              <span className={badgeVariants({ variant: 'warning' })}>
                Warning
              </span>
              <span className={badgeVariants({ variant: 'destructive' })}>
                Error
              </span>
              <span className={badgeVariants({ variant: 'outline', size: 'lg' })}>
                Large Outline
              </span>
            </div>
          </div>

        </div>
      </section>

      {/* Animation Examples */}
      <section className={createCardClass({ variant: 'default', padding: 'lg' })}>
        <h2 className="text-2xl font-bold mb-6">Animation Examples</h2>
        
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
          
          <div className="text-center space-y-2">
            <div className={cn(
              'w-16 h-16 bg-gradient-to-r from-orange-500 to-yellow-500 rounded-full mx-auto',
              animationVariants({ float: 'gentle', scale: 'hover' })
            )} />
            <p className="text-sm">Float Gentle</p>
            <code className="text-xs block bg-gray-100 dark:bg-gray-800 p-1 rounded">
              float: 'gentle'
            </code>
          </div>

          <div className="text-center space-y-2">
            <div className={cn(
              'w-16 h-16 bg-gradient-to-r from-blue-500 to-cyan-500 rounded-full mx-auto',
              animationVariants({ bounce: 'gentle', scale: 'hover' })
            )} />
            <p className="text-sm">Bounce Gentle</p>
            <code className="text-xs block bg-gray-100 dark:bg-gray-800 p-1 rounded">
              bounce: 'gentle'
            </code>
          </div>

          <div className="text-center space-y-2">
            <div className={cn(
              'w-16 h-16 bg-gradient-to-r from-green-500 to-emerald-500 rounded-full mx-auto',
              animationVariants({ pulse: 'gentle', scale: 'hover' })
            )} />
            <p className="text-sm">Pulse Gentle</p>
            <code className="text-xs block bg-gray-100 dark:bg-gray-800 p-1 rounded">
              pulse: 'gentle'
            </code>
          </div>

          <div className="text-center space-y-2">
            <div className={cn(
              'w-16 h-16 bg-gradient-to-r from-purple-500 to-pink-500 rounded-full mx-auto',
              animationVariants({ spin: 'slow', scale: 'hover' })
            )} />
            <p className="text-sm">Spin Slow</p>
            <code className="text-xs block bg-gray-100 dark:bg-gray-800 p-1 rounded">
              spin: 'slow'
            </code>
          </div>

        </div>
      </section>

      {/* Design Tokens Section */}
      <section className={createCardClass({ variant: 'default', padding: 'lg' })}>
        <h2 className="text-2xl font-bold mb-6">Design Tokens</h2>
        
        <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
          
          {/* Color Palette */}
          <div>
            <h3 className="text-lg font-semibold mb-4">Color Palette</h3>
            <div className="space-y-3">
              <div className="flex items-center gap-3">
                <div className="w-8 h-8 rounded" style={{ backgroundColor: tokens.colors.primary[500] }} />
                <span className="text-sm font-mono">primary.500</span>
                <span className="text-xs text-gray-500">{tokens.colors.primary[500]}</span>
              </div>
              <div className="flex items-center gap-3">
                <div className="w-8 h-8 rounded" style={{ backgroundColor: tokens.colors.secondary[500] }} />
                <span className="text-sm font-mono">secondary.500</span>
                <span className="text-xs text-gray-500">{tokens.colors.secondary[500]}</span>
              </div>
              <div className="flex items-center gap-3">
                <div className="w-8 h-8 rounded" style={{ backgroundColor: tokens.colors.success[500] }} />
                <span className="text-sm font-mono">success.500</span>
                <span className="text-xs text-gray-500">{tokens.colors.success[500]}</span>
              </div>
            </div>
          </div>

          {/* Spacing Scale */}
          <div>
            <h3 className="text-lg font-semibold mb-4">Spacing Scale</h3>
            <div className="space-y-2">
              {[4, 8, 12, 16, 20, 24].map((space) => (
                <div key={space} className="flex items-center gap-3">
                  <div 
                    className="bg-blue-500 rounded"
                    style={{ 
                      width: tokens.spacing[space as keyof typeof tokens.spacing], 
                      height: '16px' 
                    }}
                  />
                  <span className="text-sm font-mono">spacing.{space}</span>
                  <span className="text-xs text-gray-500">
                    {tokens.spacing[space as keyof typeof tokens.spacing]}
                  </span>
                </div>
              ))}
            </div>
          </div>

        </div>
      </section>

      {/* Migration Guide */}
      <section className={createCardClass({ variant: 'pancake', padding: 'lg', glow: true })}>
        <h2 className="text-2xl font-bold mb-6">Migration Guide</h2>
        
        <div className="space-y-6">
          <div>
            <h3 className="text-lg font-semibold mb-2 text-red-600 dark:text-red-400">
              ❌ Old Approach (CSS Classes)
            </h3>
            <div className="bg-red-50 dark:bg-red-900/20 p-4 rounded-lg">
              <code className="text-sm">
                {`<button className="btn-pancake-primary">Click me</button>`}<br/>
                {`<div className="card-pancake p-8">Content</div>`}<br/>
                {`<h1 className="pancake-gradient-text">Title</h1>`}
              </code>
            </div>
          </div>

          <div>
            <h3 className="text-lg font-semibold mb-2 text-green-600 dark:text-green-400">
              ✅ New Approach (Type-Safe Variants)
            </h3>
            <div className="bg-green-50 dark:bg-green-900/20 p-4 rounded-lg">
              <code className="text-sm">
                {`<button className={buttonVariants({ variant: 'primary' })}>Click me</button>`}<br/>
                {`<div className={cardVariants({ variant: 'pancake', padding: 'lg' })}>Content</div>`}<br/>
                {`<h1 className={gradientTextVariants({ gradient: 'primary' })}>Title</h1>`}
              </code>
            </div>
          </div>

          <div className="bg-blue-50 dark:bg-blue-900/20 p-4 rounded-lg">
            <h4 className="font-semibold mb-2">Benefits of the New System:</h4>
            <ul className="list-disc list-inside space-y-1 text-sm">
              <li>🔒 Type safety with IntelliSense support</li>
              <li>🎯 Tree-shaking - only load what you use</li>
              <li>🔧 Composable variants and compound variants</li>
              <li>📦 Better performance and smaller bundle size</li>
              <li>🎨 Consistent design language across components</li>
              <li>🛠️ Easy to extend and customize</li>
            </ul>
          </div>
        </div>
      </section>

      {/* Footer */}
      <footer className="text-center py-8">
        <p className="text-gray-600 dark:text-gray-400">
          EPSX Design System v1.0.0 - Built with ❤️ and TypeScript
        </p>
      </footer>
    </div>
  );
}