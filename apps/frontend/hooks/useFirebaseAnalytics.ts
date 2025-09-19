// Firebase Analytics disabled for security - no external data transmission
// All analytics functions are no-ops to prevent data leakage

export const logEvent = () => {};
export const setUserId = () => {};
export const setUserProperties = () => {};
export const setCurrentScreen = () => {};
export const setAnalyticsCollectionEnabled = () => {};

// Stub implementations for other common Firebase Analytics functions
export const trackEvent = () => {};
export const identify = () => {};
export const page = () => {};
export const track = () => {};
export const reset = () => {};