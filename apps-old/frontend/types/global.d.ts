/**
 * Global type declarations for the frontend application
 */

import type React from 'react';

// Extend global namespace for JSX
declare global {
  namespace JSX {
    interface IntrinsicElements extends React.JSX.IntrinsicElements {}
    interface Element extends React.JSX.Element {}
    interface IntrinsicAttributes extends React.JSX.IntrinsicAttributes {}
    interface IntrinsicClassAttributes<T> extends React.JSX.IntrinsicClassAttributes<T> {}
    interface ElementClass extends React.JSX.ElementClass {}
    interface ElementAttributesProperty extends React.JSX.ElementAttributesProperty {}
    interface ElementChildrenAttribute extends React.JSX.ElementChildrenAttribute {}
  }
}

// React namespace declaration
declare global {
  namespace React {
    interface FC<P = {}> extends FunctionComponent<P> {}
    interface Component<P = {}, S = {}> extends Component<P, S> {}
  }
}

// Window extensions for development tools
declare global {
  interface Window {
    __EPSX_DEV_TOOLS__?: {
      stateHistory: unknown[];
      actions: unknown[];
      performance: {
        start: (name: string) => void;
        end: (name: string) => void;
        measure: (name: string) => number;
      };
    };
  }
}

// NextRequest extensions for middleware
declare global {
  namespace NodeJS {
    interface Global {
      // Add any global Node.js type extensions here
    }
  }
}

// Extend NextRequest to include commonly used properties in Edge Runtime
declare module 'next/server' {
  interface NextRequest {
    ip?: string;
    geo?: {
      country?: string;
      region?: string;
      city?: string;
      latitude?: string;
      longitude?: string;
    };
  }
}

export {};