# @epsx/ui - PancakeSwap-Inspired Design System

A comprehensive UI component library for the EPSX trading platform, featuring PancakeSwap-inspired theming with mobile-first responsive design.

## ✨ Features

- **🎨 PancakeSwap-Inspired Theming**: Orange/yellow gradients with 3 theme variants
- **📱 Mobile-First Responsive**: Optimized for all screen sizes
- **🌓 Dark Mode Support**: Automatic system theme detection
- **⚡ Performance Optimized**: Minimal bundle size with tree-shaking
- **🧩 Modular Architecture**: Import only what you need
- **♿ Accessibility First**: WCAG 2.1 compliant components

## 🚀 Quick Start

### Installation

```bash
pnpm add @epsx/ui
```

### Basic Usage

```tsx
import { ThemeProvider, Button, Card } from '@epsx/ui';

function App() {
  return (
    <ThemeProvider defaultTheme="pancake">
      <Card className="p-6">
        <Button>Get Started</Button>
      </Card>
    </ThemeProvider>
  );
}
```

## 🎨 Theme System

### Theme Variants

- **default**: Clean, professional design
- **pancake**: PancakeSwap-inspired orange/yellow theme
- **trading**: Specialized for trading interfaces with bullish/bearish colors

### Theme Provider

```tsx
import { ThemeProvider } from '@epsx/ui';

<ThemeProvider 
  defaultTheme="pancake"
  enableSystem={true}
  storageKey="epsx-theme"
>
  <App />
</ThemeProvider>
```

### Theme Controls

```tsx
import { ThemeVariantSelector, DarkModeToggle, useTheme } from '@epsx/ui';

function ThemeControls() {
  const { theme, isDarkMode, setTheme, toggleDarkMode } = useTheme();
  
  return (
    <div className="flex gap-4">
      <ThemeVariantSelector />
      <DarkModeToggle showLabel />
    </div>
  );
}
```

## 🧱 Components

### Button

```tsx
import { Button } from '@epsx/ui';

<Button variant="default">Primary</Button>
<Button variant="secondary">Secondary</Button>
<Button variant="outline">Outline</Button>
<Button variant="ghost">Ghost</Button>
```

### Card

```tsx
import { Card } from '@epsx/ui';

<Card className="p-6">
  <h3>Card Title</h3>
  <p>Card content goes here.</p>
</Card>
```

### Form Components

```tsx
import { Input, Label, Form } from '@epsx/ui';

<Form>
  <Label htmlFor="email">Email</Label>
  <Input 
    id="email" 
    type="email" 
    placeholder="Enter your email"
  />
</Form>
```

## 📱 Responsive Design

### Breakpoints

```ts
const breakpoints = {
  xs: '0px',      // Mobile
  sm: '640px',    // Small tablets
  md: '768px',    // Tablets
  lg: '1024px',   // Desktop
  xl: '1280px',   // Large desktop
  '2xl': '1536px' // Extra large
};
```

### Responsive Utilities

```tsx
import { useBreakpoint, useMediaQuery } from '@epsx/ui';

function ResponsiveComponent() {
  const breakpoint = useBreakpoint();
  const isMobile = useMediaQuery('(max-width: 768px)');
  
  return (
    <div className="text-sm md:text-base lg:text-lg">
      Current breakpoint: {breakpoint}
    </div>
  );
}
```

### Responsive Classes

```tsx
// Typography
<h1 className="text-2xl sm:text-3xl md:text-4xl lg:text-5xl">
  Responsive Heading
</h1>

// Layout
<div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
  <Card>Item 1</Card>
  <Card>Item 2</Card>
  <Card>Item 3</Card>
</div>

// Spacing
<section className="py-8 sm:py-12 md:py-16 lg:py-20">
  Content with responsive padding
</section>
```

## 🎯 Trading-Specific Components

### Trading Colors

```tsx
// Bullish/Bearish indicators
<div className="text-success">Bullish: +2.5%</div>
<div className="text-error">Bearish: -1.8%</div>
<div className="text-warning">Neutral</div>

// Background variants
<Card className="bg-success/10 border-success/20">
  Bullish signal card
</Card>
```

### Trading Theme

```tsx
<ThemeProvider defaultTheme="trading">
  <Button variant="bullish">Buy</Button>
  <Button variant="bearish">Sell</Button>
  <Button variant="neutral">Hold</Button>
</ThemeProvider>
```

## 🛠️ Advanced Usage

### Custom Theme Configuration

```tsx
import { themeConfig } from '@epsx/ui';

// Access theme tokens
const { variants, components, responsive } = themeConfig;

// Custom CSS variables
const customTheme = {
  colors: {
    primary: 'hsl(25 95% 53%)',
    secondary: 'hsl(45 93% 47%)',
  },
  gradients: {
    primary: 'linear-gradient(135deg, hsl(25 95% 53%) 0%, hsl(45 93% 47%) 100%)',
  }
};
```

### Utility Classes

```tsx
import { cn, generateResponsiveClasses } from '@epsx/ui';

// Combine classes conditionally
const buttonClasses = cn(
  'px-4 py-2 rounded',
  isActive && 'bg-primary',
  isDisabled && 'opacity-50'
);

// Generate responsive classes
const responsiveText = generateResponsiveClasses({
  base: 'text-sm',
  md: 'text-base',
  lg: 'text-lg'
});
```

## 🎨 Design Tokens

### Colors

```css
/* Primary colors */
--color-primary: hsl(25 95% 53%);
--color-secondary: hsl(45 93% 47%);
--color-accent: hsl(142 71% 45%);

/* Trading colors */
--color-success: hsl(142 71% 45%);  /* Bullish */
--color-error: hsl(0 85% 60%);      /* Bearish */
--color-warning: hsl(45 93% 47%);   /* Neutral */

/* Gradients */
--gradient-primary: linear-gradient(135deg, hsl(25 95% 53%) 0%, hsl(45 93% 47%) 100%);
--gradient-rainbow: linear-gradient(135deg, hsl(142 71% 45%) 0%, hsl(25 95% 53%) 25%, hsl(45 93% 47%) 50%, hsl(217 91% 60%) 75%, hsl(271 81% 56%) 100%);
```

### Typography

```css
/* Responsive headings */
.heading-responsive-h1 { @apply text-2xl sm:text-3xl md:text-4xl lg:text-5xl font-bold; }
.heading-responsive-h2 { @apply text-xl sm:text-2xl md:text-3xl lg:text-4xl font-bold; }

/* Body text */
.body-responsive-large { @apply text-base sm:text-lg; }
.body-responsive-default { @apply text-sm sm:text-base; }
```

## 📊 Performance

- **Bundle Size**: ~50KB gzipped
- **Tree Shaking**: Supported
- **CSS-in-JS**: CSS custom properties for themes
- **SSR Compatible**: Server-side rendering ready

## 🔧 Development

### Building

```bash
pnpm build
```

### Type Checking

```bash
pnpm type-check
```

### Testing Theme Integration

Visit `/theme-test` in your application to see all components and theme variants in action.

## 📚 API Reference

### ThemeProvider Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `defaultTheme` | `ThemeVariant` | `"default"` | Initial theme variant |
| `enableSystem` | `boolean` | `true` | Enable system theme detection |
| `storageKey` | `string` | `"epsx-theme"` | localStorage key for theme persistence |
| `disableTransitions` | `boolean` | `false` | Disable theme transition animations |

### useTheme Hook

```tsx
const {
  theme,           // Current theme variant
  setTheme,        // Function to change theme
  isDarkMode,      // Dark mode state
  toggleDarkMode,  // Function to toggle dark mode
  tokens,          // Design tokens object
  config           // Theme configuration object
} = useTheme();
```

## 🤝 Contributing

1. Follow the existing code patterns
2. Ensure mobile-first responsive design
3. Test with all theme variants
4. Update documentation for new components

## 📄 License

MIT License - see LICENSE file for details