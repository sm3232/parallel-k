import { create } from 'zustand';
import { persist } from 'zustand/middleware';

export type ThemeType = 'light' | 'dark';

export interface ThemeStoreType {
    theme: ThemeType,
    toggleTheme: () => void,
    setTheme: (newTheme: ThemeType) => void,
}

const getIntitialTheme = (): ThemeType => {
    if(typeof window !== 'undefined' && window.matchMedia('(prefers-color-scheme: dark)').matches) {
        return 'dark';
    }

    return 'light';
}

export const useThemeStore = create<ThemeStoreType>()(
    persist(
        (set) => ({
            theme: getIntitialTheme(),
            
            toggleTheme: () => 
                set((state) => ({
                    theme: state.theme === 'light' ? 'dark' : 'light'
                })),
            setTheme: (newTheme: ThemeType) => set({ theme: newTheme })
        }),
        {
            name: 'theme-storage',
        }
    )
);

export const useTheme = () => useThemeStore((state) => state.theme);
export const useThemeToggle = () => useThemeStore((state) => state.toggleTheme);
export const useThemeSet = () => useThemeStore((state) => state.setTheme);