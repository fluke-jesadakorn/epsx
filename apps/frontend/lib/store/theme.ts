import { create } from 'zustand';

interface ThemeState {
  isDarkMode: boolean;
  setDarkMode: (isDarkMode: boolean) => void;
}

export const useThemeStore = create<ThemeState>((set) => ({
  isDarkMode: false,
  setDarkMode: (isDarkMode: boolean) => set({ isDarkMode }),
}));
