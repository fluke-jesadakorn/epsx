'use client';

import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@epsx/ui";
import { 
  ThemeVariantSelector, 
  DarkModeToggle,
  useTheme
} from '@epsx/ui';
import { Suspense } from 'react';

// Force dynamic rendering
export const dynamic = 'force-dynamic';

function ThemeTestContent() {
  const { theme, isDarkMode, tokens, config } = useTheme();

  return (
    <div className="min-h-screen p-8 space-y-8">
      <div className="max-w-4xl mx-auto">
        <h1 className="text-4xl font-bold mb-2">Theme System Test</h1>
        <p className="text-lg text-muted-foreground mb-8">
          Testing PancakeSwap-inspired theming with variance-based design system
        </p>

        {/* Theme Controls */}
        <Card variant="pancake">
          <CardHeader>
            <CardTitle>Theme Controls</CardTitle>
            <CardDescription>Interactive theme switching and customization</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="flex items-center gap-4 mb-4">
              <div>
                <label className="block text-sm font-medium mb-2">Theme Variant:</label>
                <ThemeVariantSelector className="flex-wrap" />
              </div>
              <div>
                <label className="block text-sm font-medium mb-2">Dark Mode:</label>
                <DarkModeToggle showLabel />
              </div>
            </div>
            <div className="text-sm text-muted-foreground">
              Current: <strong>{theme}</strong> - {isDarkMode ? 'Dark' : 'Light'} Mode
            </div>
          </CardContent>
        </Card>

        {/* Button Showcase */}
        <Card variant="elevated">
          <CardHeader>
            <CardTitle>Button Variants</CardTitle>
            <CardDescription>All available button styles and variants</CardDescription>
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
              <Button variant="default">Default</Button>
              <Button variant="pancake">Pancake</Button>
              <Button variant="pancake-secondary">Pancake Secondary</Button>
              <Button variant="pancake-outline">Pancake Outline</Button>
              <Button variant="pancake-ghost">Pancake Ghost</Button>
              <Button variant="pancake-soft">Pancake Soft</Button>
              <Button variant="bullish">Bullish</Button>
              <Button variant="bearish">Bearish</Button>
              <Button variant="neutral">Neutral</Button>
              <Button variant="gradient">Gradient</Button>
              <Button variant="outline">Outline</Button>
              <Button variant="ghost">Ghost</Button>
            </div>
          </CardContent>
        </Card>

        {/* Card Showcase */}
        <div className="space-y-4">
          <h2 className="text-2xl font-bold">Card Variants</h2>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            <Card>
              <CardHeader>
                <CardTitle>Default Card</CardTitle>
                <CardDescription>Standard card appearance</CardDescription>
              </CardHeader>
              <CardContent>
                <p className="text-sm">Basic card with default styling and border.</p>
              </CardContent>
            </Card>

            <Card variant="pancake">
              <CardHeader>
                <CardTitle>Pancake Card</CardTitle>
                <CardDescription>PancakeSwap-inspired styling</CardDescription>
              </CardHeader>
              <CardContent>
                <p className="text-sm">Enhanced with backdrop blur and orange accents.</p>
              </CardContent>
            </Card>

            <Card variant="elevated">
              <CardHeader>
                <CardTitle>Elevated Card</CardTitle>
                <CardDescription>Higher elevation with stronger shadows</CardDescription>
              </CardHeader>
              <CardContent>
                <p className="text-sm">Perfect for important content and overlays.</p>
              </CardContent>
            </Card>

            <Card variant="glowing">
              <CardHeader>
                <CardTitle>Glowing Card</CardTitle>
                <CardDescription>Subtle glow effect</CardDescription>
              </CardHeader>
              <CardContent>
                <p className="text-sm">Eye-catching glow for featured content.</p>
              </CardContent>
            </Card>

            <Card variant="trading">
              <CardHeader>
                <CardTitle>Trading Card</CardTitle>
                <CardDescription>Neutral trading interface</CardDescription>
              </CardHeader>
              <CardContent>
                <p className="text-sm">Clean design for trading interfaces.</p>
              </CardContent>
            </Card>

            <Card variant="bullish" interactive>
              <CardHeader>
                <CardTitle>Bullish Card</CardTitle>
                <CardDescription>Green accent for positive data</CardDescription>
              </CardHeader>
              <CardContent>
                <p className="text-sm">Interactive card with green theme (hover me!).</p>
              </CardContent>
            </Card>

            <Card variant="bearish" interactive>
              <CardHeader>
                <CardTitle>Bearish Card</CardTitle>
                <CardDescription>Red accent for negative data</CardDescription>
              </CardHeader>
              <CardContent>
                <p className="text-sm">Interactive card with red theme (hover me!).</p>
              </CardContent>
            </Card>

            <Card variant="glassmorphism">
              <CardHeader>
                <CardTitle>Glassmorphism</CardTitle>
                <CardDescription>Modern glass effect</CardDescription>
              </CardHeader>
              <CardContent>
                <p className="text-sm">Trendy glass morphism design.</p>
              </CardContent>
            </Card>
          </div>
        </div>

        {/* Color Palette */}
        <Card className="p-6 mb-8">
          <h2 className="text-2xl font-semibold mb-4">Color Palette</h2>
          <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
            <div className="space-y-2">
              <div className="w-full h-16 bg-primary rounded border"></div>
              <div className="text-sm">Primary</div>
            </div>
            <div className="space-y-2">
              <div className="w-full h-16 bg-secondary rounded border"></div>
              <div className="text-sm">Secondary</div>
            </div>
            <div className="space-y-2">
              <div className="w-full h-16 bg-accent rounded border"></div>
              <div className="text-sm">Accent</div>
            </div>
            <div className="space-y-2">
              <div className="w-full h-16 bg-muted rounded border"></div>
              <div className="text-sm">Muted</div>
            </div>
          </div>
        </Card>

        {/* Gradients */}
        <Card className="p-6 mb-8">
          <h2 className="text-2xl font-semibold mb-4">Gradients</h2>
          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div className="space-y-2">
              <div className="w-full h-16 bg-gradient-to-r from-primary to-secondary rounded"></div>
              <div className="text-sm">Primary Gradient</div>
            </div>
            <div className="space-y-2">
              <div className="w-full h-16 bg-gradient-to-r from-accent to-primary rounded"></div>
              <div className="text-sm">Accent Gradient</div>
            </div>
          </div>
        </Card>

        {/* Responsive Utilities */}
        <Card className="p-6">
          <h2 className="text-2xl font-semibold mb-4">Responsive Design</h2>
          <div className="space-y-4">
            <div className="p-4 bg-muted/20 rounded">
              <div className="text-sm xs:text-base md:text-lg lg:text-xl">
                Responsive text: xs(sm) → sm(base) → md(lg) → lg(xl)
              </div>
            </div>
            <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
              <div className="p-4 bg-primary/10 rounded text-center">Mobile First</div>
              <div className="p-4 bg-secondary/10 rounded text-center">Responsive</div>
              <div className="p-4 bg-accent/10 rounded text-center">Grid</div>
              <div className="p-4 bg-muted/10 rounded text-center">System</div>
            </div>
          </div>
        </Card>

        {/* Debug Info */}
        <Card className="p-6">
          <h2 className="text-2xl font-semibold mb-4">Theme Debug Info</h2>
          <div className="space-y-2 text-sm font-mono">
            <div>Theme: {theme}</div>
            <div>Dark Mode: {isDarkMode ? 'true' : 'false'}</div>
            <div>Available Variants: {Object.keys(config.variants).join(', ')}</div>
          </div>
        </Card>
      </div>
    </div>
  );
}

export default function ThemeTestPage() {
  return (
    <Suspense fallback={<div className="min-h-screen flex items-center justify-center">Loading theme test...</div>}>
      <ThemeTestContent />
    </Suspense>
  );
}