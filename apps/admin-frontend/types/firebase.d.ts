// Firebase global types for window.firebase
declare global {
  interface Window {
    firebase: {
      initializeApp: (config: any) => any;
      app: () => any;
      messaging: () => any;
    };
  }
}

export {};