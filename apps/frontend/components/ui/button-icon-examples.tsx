'use client';

import * as React from 'react';
import { ButtonIcon } from './button-icon';

// Example icons - you would typically import these from a library like lucide-react
const MenuIcon = () => (
  <svg 
    xmlns="http://www.w3.org/2000/svg" 
    width="24" 
    height="24" 
    viewBox="0 0 24 24" 
    fill="none" 
    stroke="currentColor" 
    strokeWidth="2" 
    strokeLinecap="round" 
    strokeLinejoin="round"
  >
    <line x1="3" y1="6" x2="21" y2="6"/>
    <line x1="3" y1="12" x2="21" y2="12"/>
    <line x1="3" y1="18" x2="21" y2="18"/>
  </svg>
);

const SearchIcon = () => (
  <svg 
    xmlns="http://www.w3.org/2000/svg" 
    width="24" 
    height="24" 
    viewBox="0 0 24 24" 
    fill="none" 
    stroke="currentColor" 
    strokeWidth="2" 
    strokeLinecap="round" 
    strokeLinejoin="round"
  >
    <circle cx="11" cy="11" r="8"/>
    <path d="m21 21-4.35-4.35"/>
  </svg>
);

const HeartIcon = () => (
  <svg 
    xmlns="http://www.w3.org/2000/svg" 
    width="24" 
    height="24" 
    viewBox="0 0 24 24" 
    fill="none" 
    stroke="currentColor" 
    strokeWidth="2" 
    strokeLinecap="round" 
    strokeLinejoin="round"
  >
    <path d="M19 14c1.49-1.46 3-3.21 3-5.5A5.5 5.5 0 0 0 16.5 3c-1.76 0-3 .5-4.5 2-1.5-1.5-2.74-2-4.5-2A5.5 5.5 0 0 0 2 8.5c0 2.29 1.51 4.04 3 5.5Z"/>
  </svg>
);

const SettingsIcon = () => (
  <svg 
    xmlns="http://www.w3.org/2000/svg" 
    width="24" 
    height="24" 
    viewBox="0 0 24 24" 
    fill="none" 
    stroke="currentColor" 
    strokeWidth="2" 
    strokeLinecap="round" 
    strokeLinejoin="round"
  >
    <path d="M12 20a8 8 0 1 0 0-16 8 8 0 0 0 0 16Z"/>
    <path d="M12 14a2 2 0 1 0 0-4 2 2 0 0 0 0 4Z"/>
    <path d="M12 2v2"/>
    <path d="M12 20v2"/>
    <path d="m4.93 4.93 1.41 1.41"/>
    <path d="m17.66 17.66 1.41 1.41"/>
    <path d="M2 12h2"/>
    <path d="M20 12h2"/>
    <path d="m6.34 17.66-1.41 1.41"/>
    <path d="m19.07 4.93-1.41 1.41"/>
  </svg>
);

export default function ButtonIconExamples() {
  const [liked, setLiked] = React.useState(false);
  const [menuOpen, setMenuOpen] = React.useState(false);

  return (
    <div className="p-8 space-y-8">
      <div className="space-y-4">
        <h2 className="text-2xl font-bold">Button Icon Examples</h2>
        <p className="text-muted-foreground">
          Various examples of the ButtonIcon component with different variants and sizes.
        </p>
      </div>

      {/* Default variant examples */}
      <div className="space-y-4">
        <h3 className="text-lg font-semibold">Variants</h3>
        <div className="flex items-center gap-4">
          <ButtonIcon variant="default" srLabel="Search">
            <SearchIcon />
          </ButtonIcon>
          <ButtonIcon variant="secondary" srLabel="Menu">
            <MenuIcon />
          </ButtonIcon>
          <ButtonIcon variant="outline" srLabel="Settings">
            <SettingsIcon />
          </ButtonIcon>
          <ButtonIcon variant="ghost" srLabel="More options">
            <MenuIcon />
          </ButtonIcon>
          <ButtonIcon variant="destructive" srLabel="Delete">
            <svg className="h-4 w-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
            </svg>
          </ButtonIcon>
        </div>
      </div>

      {/* Size examples */}
      <div className="space-y-4">
        <h3 className="text-lg font-semibold">Sizes</h3>
        <div className="flex items-center gap-4">
          <ButtonIcon size="sm" variant="outline" srLabel="Small search">
            <SearchIcon />
          </ButtonIcon>
          <ButtonIcon size="default" variant="outline" srLabel="Default search">
            <SearchIcon />
          </ButtonIcon>
          <ButtonIcon size="lg" variant="outline" srLabel="Large search">
            <SearchIcon />
          </ButtonIcon>
          <ButtonIcon size="xl" variant="outline" srLabel="Extra large search">
            <SearchIcon />
          </ButtonIcon>
        </div>
      </div>

      {/* Interactive examples */}
      <div className="space-y-4">
        <h3 className="text-lg font-semibold">Interactive Examples</h3>
        <div className="flex items-center gap-4">
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
          
          <ButtonIcon 
            variant="ghost"
            active={menuOpen}
            onClick={() => setMenuOpen(!menuOpen)}
            srLabel={menuOpen ? "Close menu" : "Open menu"}
            tooltip={menuOpen ? "Close menu" : "Open menu"}
          >
            <MenuIcon />
          </ButtonIcon>
          
          <ButtonIcon 
            variant="outline"
            onClick={() => alert('Settings clicked!')}
            srLabel="Open settings"
            tooltip="Open settings"
          >
            <SettingsIcon />
          </ButtonIcon>
        </div>
      </div>

      {/* Disabled state */}
      <div className="space-y-4">
        <h3 className="text-lg font-semibold">Disabled State</h3>
        <div className="flex items-center gap-4">
          <ButtonIcon variant="default" disabled srLabel="Disabled search">
            <SearchIcon />
          </ButtonIcon>
          <ButtonIcon variant="outline" disabled srLabel="Disabled menu">
            <MenuIcon />
          </ButtonIcon>
          <ButtonIcon variant="ghost" disabled srLabel="Disabled settings">
            <SettingsIcon />
          </ButtonIcon>
        </div>
      </div>
    </div>
  );
}
