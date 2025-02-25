import { create } from "zustand";
import { persist, createJSONStorage } from "zustand/middleware";
import { theme, ThemeConfig } from "antd";

const lightTheme: ThemeConfig = {
  token: {
    wireframe: true,
    borderRadius: 16,
    colorPrimary: "#1fc7d4",
    colorInfo: "#1fc7d4",
    colorBgBase: "#ffffff",
  },
  components: {
    Button: {
      algorithm: true,
    },
    // Menu: {
    //   colorPrimaryActive: "#ffffff",
    // },
  },
  algorithm: theme.defaultAlgorithm,
};

const darkTheme: ThemeConfig = {
  token: {
    wireframe: true,
    borderRadius: 16,
    colorPrimary: "#1fc7d4",
    colorInfo: "#1fc7d4",
    colorBgBase: "#27262c",
  },
  components: {
    Button: {
      algorithm: true,
    },
    // Menu: {
    //   colorPrimaryActive: "#ffffff",
    // },
  },
  algorithm: theme.darkAlgorithm,
};

export const ThemeType = {
  lightTheme,
  darkTheme,
};

interface ThemeState {
  theme: ThemeConfig;
  isDarkMode: boolean;
  setTheme: (theme: ThemeConfig) => void;
  toggleTheme: () => void;
  initializeTheme: () => void;
}

export const useThemeStore = create<ThemeState>()(
  persist(
    (set) => ({
      theme: darkTheme,
      isDarkMode: true,
      setTheme: (theme) => set({ theme }),
toggleTheme: () =>
  set((state) => ({
    isDarkMode: !state.isDarkMode,
    theme: !state.isDarkMode ? lightTheme : darkTheme,
  })),
      initializeTheme: () => {
        if (typeof window !== "undefined") {
          const prefersDark = window.matchMedia(
            "(prefers-color-scheme: dark)"
          ).matches;
          set({
            theme: prefersDark ? darkTheme : lightTheme,
            isDarkMode: prefersDark,
          });
        }
      },
    }),
    {
      name: "theme-storage",
      storage: createJSONStorage(() => {
        if (typeof window !== "undefined") {
          return localStorage;
        }
        // Return a dummy storage implementation for SSR
        return {
          getItem: () => null,
          setItem: () => {},
          removeItem: () => {},
        };
      }),
    }
  )
);
