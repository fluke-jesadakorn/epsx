# ButtonIcon Component

A reusable button icon component built with React, TypeScript, and Tailwind CSS. This component provides accessible icon buttons with various variants, sizes, and states.

## Features

- 🎨 **Multiple Variants**: default, secondary, outline, ghost, destructive, link
- 📏 **Multiple Sizes**: sm, default, lg, xl  
- ♿ **Accessibility**: Built-in screen reader support and ARIA labels
- 🎯 **Interactive States**: hover, active, disabled, focus
- 🎭 **Customizable**: Easy to customize with additional CSS classes
- 💡 **Tooltip Support**: Optional tooltip text
- 🔄 **Active State**: Visual feedback for toggle-like behavior

## Installation

The ButtonIcon component is available in three locations:

1. **Shared UI Package**: `/packages/ui/src/components/button/icon.tsx`
2. **Frontend App**: `/apps/frontend/components/ui/button-icon.tsx`
3. **Admin Frontend**: `/apps/admin-frontend/components/ui/button-icon.tsx`

## Basic Usage

```tsx
import { ButtonIcon } from '@/components/ui/button-icon';

function MyComponent() {
  return (
    <ButtonIcon srLabel="Search">
      <SearchIcon />
    </ButtonIcon>
  );
}
```

## Props

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `variant` | `'default' \| 'secondary' \| 'outline' \| 'ghost' \| 'destructive' \| 'link'` | `'ghost'` | Visual style variant |
| `size` | `'sm' \| 'default' \| 'lg' \| 'xl'` | `'default'` | Size of the button |
| `active` | `boolean` | `false` | Whether the button is in an active state |
| `srLabel` | `string` | - | Screen reader label for accessibility |
| `tooltip` | `string` | - | Tooltip text on hover |
| `disabled` | `boolean` | `false` | Whether the button is disabled |
| `className` | `string` | - | Additional CSS classes |
| `children` | `ReactNode` | - | Icon or content to display |

## Examples

### Basic Variants

```tsx
<ButtonIcon variant="default" srLabel="Search">
  <SearchIcon />
</ButtonIcon>

<ButtonIcon variant="outline" srLabel="Menu">
  <MenuIcon />
</ButtonIcon>

<ButtonIcon variant="ghost" srLabel="Settings">
  <SettingsIcon />
</ButtonIcon>
```

### Different Sizes

```tsx
<ButtonIcon size="sm" srLabel="Small icon">
  <Icon />
</ButtonIcon>

<ButtonIcon size="default" srLabel="Default icon">
  <Icon />
</ButtonIcon>

<ButtonIcon size="lg" srLabel="Large icon">
  <Icon />
</ButtonIcon>

<ButtonIcon size="xl" srLabel="Extra large icon">
  <Icon />
</ButtonIcon>
```

### Interactive Example

```tsx
function LikeButton() {
  const [liked, setLiked] = useState(false);
  
  return (
    <ButtonIcon
      variant="ghost"
      active={liked}
      onClick={() => setLiked(!liked)}
      className={liked ? "text-red-500 hover:text-red-600" : ""}
      srLabel={liked ? "Unlike" : "Like"}
      tooltip={liked ? "Unlike this item" : "Like this item"}
    >
      <HeartIcon />
    </ButtonIcon>
  );
}
```

### With Tooltip

```tsx
<ButtonIcon 
  variant="outline"
  srLabel="Open settings"
  tooltip="Open application settings"
>
  <SettingsIcon />
</ButtonIcon>
```

### Disabled State

```tsx
<ButtonIcon 
  variant="default" 
  disabled 
  srLabel="Disabled action"
>
  <Icon />
</ButtonIcon>
```

## Styling

The component uses Tailwind CSS classes and CSS custom properties. Key styling features:

- **Rounded**: Always uses `rounded-full` for circular appearance
- **Focus Ring**: Includes focus-visible ring for keyboard navigation
- **Transitions**: Smooth transitions for all interactive states
- **Icon Sizing**: Automatic icon sizing with `[&_svg:not([class*='size-'])]:size-4`
- **Color Management**: Intelligent color handling for icons and text

### CSS Classes Applied

```css
/* Base classes */
inline-flex items-center justify-center rounded-full
text-sm font-medium transition-all duration-200
focus-visible:outline-none focus-visible:ring-4 focus-visible:ring-ring/10
disabled:pointer-events-none disabled:opacity-50 active:scale-[0.98]

/* Icon-specific classes */
[&_svg:not([class*='text-'])]:text-muted-foreground
[&_svg:not([class*='size-'])]:size-4
```

## Accessibility

The ButtonIcon component follows WCAG guidelines:

- **Screen Reader Support**: Use `srLabel` prop for descriptive labels
- **ARIA Labels**: Automatic `aria-label` handling  
- **Keyboard Navigation**: Full keyboard support with focus indicators
- **Color Contrast**: Proper contrast ratios for all variants
- **Focus Management**: Clear focus indicators

```tsx
// Good accessibility
<ButtonIcon srLabel="Delete item" tooltip="Delete this item permanently">
  <TrashIcon />
</ButtonIcon>

// Also good with explicit aria-label
<ButtonIcon aria-label="Close dialog">
  <XIcon />
</ButtonIcon>
```

## Customization

You can customize the component by:

1. **Adding custom classes**:
```tsx
<ButtonIcon className="my-custom-class border-2 border-blue-500">
  <Icon />
</ButtonIcon>
```

2. **Modifying the variants** (in the component file):
```tsx
const buttonIconVariants = cva(
  // base classes
  "...",
  {
    variants: {
      variant: {
        // Add your custom variant
        custom: "bg-purple-500 text-white hover:bg-purple-600",
      }
    }
  }
);
```

## Icon Libraries

The component works well with popular icon libraries:

- **Lucide React**: `lucide-react`
- **Heroicons**: `@heroicons/react`
- **React Icons**: `react-icons`
- **Tabler Icons**: `@tabler/icons-react`

```tsx
// With Lucide React
import { Search, Menu, Settings } from 'lucide-react';

<ButtonIcon srLabel="Search">
  <Search />
</ButtonIcon>

// With Heroicons
import { MagnifyingGlassIcon } from '@heroicons/react/24/outline';

<ButtonIcon srLabel="Search">
  <MagnifyingGlassIcon />
</ButtonIcon>
```

## Common Patterns

### Navigation Icons
```tsx
<nav className="flex gap-2">
  <ButtonIcon srLabel="Home" tooltip="Go to home">
    <HomeIcon />
  </ButtonIcon>
  <ButtonIcon srLabel="Profile" tooltip="View profile">
    <UserIcon />
  </ButtonIcon>
  <ButtonIcon srLabel="Settings" tooltip="Open settings">
    <SettingsIcon />
  </ButtonIcon>
</nav>
```

### Action Bar
```tsx
<div className="flex items-center gap-1 p-2 border rounded-lg">
  <ButtonIcon srLabel="Bold" active={isBold} onClick={toggleBold}>
    <BoldIcon />
  </ButtonIcon>
  <ButtonIcon srLabel="Italic" active={isItalic} onClick={toggleItalic}>
    <ItalicIcon />
  </ButtonIcon>
  <ButtonIcon srLabel="Underline" active={isUnderline} onClick={toggleUnderline}>
    <UnderlineIcon />
  </ButtonIcon>
</div>
```

### Modal Controls
```tsx
<div className="flex justify-between items-center">
  <h2>Modal Title</h2>
  <ButtonIcon 
    variant="ghost" 
    srLabel="Close modal" 
    onClick={onClose}
  >
    <XIcon />
  </ButtonIcon>
</div>
```

## Performance

The component is optimized for performance:

- ✅ **Lightweight**: Minimal bundle size impact
- ✅ **Tree-shakable**: Only imports what you use
- ✅ **Memoizable**: Works well with `React.memo()`
- ✅ **SSR Compatible**: No client-side only dependencies

## Browser Support

- ✅ Chrome 90+
- ✅ Firefox 88+
- ✅ Safari 14+
- ✅ Edge 90+

## Migration from Button

If you're migrating from a regular Button component:

```tsx
// Before
<Button size="icon" variant="ghost">
  <Icon />
</Button>

// After  
<ButtonIcon variant="ghost" srLabel="Descriptive label">
  <Icon />
</ButtonIcon>
```

## Contributing

When contributing to the ButtonIcon component:

1. Ensure accessibility standards are maintained
2. Test with keyboard navigation
3. Verify all variants and sizes work correctly
4. Update examples if adding new features
5. Follow the existing code style and patterns
