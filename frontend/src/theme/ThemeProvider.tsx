import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  useSyncExternalStore,
} from 'react';
import type { ReactNode } from 'react';

export type ThemeMode = 'dark' | 'light' | 'system';
export type ResolvedTheme = 'dark' | 'light';

export interface ThemeContextValue {
  mode: ThemeMode;
  resolvedMode: ResolvedTheme;
  setMode: (mode: ThemeMode) => void;
}

const STORAGE_KEY = 'oximeal.theme';
const DARK_QUERY = '(prefers-color-scheme: dark)';

const ThemeContext = createContext<ThemeContextValue | null>(null);

function isThemeMode(value: string | null): value is ThemeMode {
  return value === 'dark' || value === 'light' || value === 'system';
}

function readStoredMode(): ThemeMode {
  if (typeof window === 'undefined') {
    return 'dark';
  }
  try {
    const stored = window.localStorage.getItem(STORAGE_KEY);
    return isThemeMode(stored) ? stored : 'dark';
  } catch {
    return 'dark';
  }
}

const THEME_COLORS: Record<ResolvedTheme, string> = {
  dark: '#0f172a',
  light: '#f7fafc',
};

function subscribeSystemTheme(onChange: () => void): () => void {
  if (typeof window === 'undefined' || typeof window.matchMedia !== 'function') {
    return () => {};
  }
  const mediaQuery = window.matchMedia(DARK_QUERY);
  mediaQuery.addEventListener('change', onChange);
  return () => {
    mediaQuery.removeEventListener('change', onChange);
  };
}

function getSystemSnapshot(): boolean {
  if (typeof window === 'undefined' || typeof window.matchMedia !== 'function') {
    return false;
  }
  return window.matchMedia(DARK_QUERY).matches;
}

function getServerSnapshot(): boolean {
  return false;
}

export interface ThemeProviderProps {
  children: ReactNode;
}

/**
 * Provee el modo de tema (oscuro, claro o sistema) y aplica el tema resuelto
 * al documento. La preferencia se persiste en `localStorage`.
 */
export function ThemeProvider({ children }: ThemeProviderProps) {
  const [mode, setModeState] = useState<ThemeMode>(readStoredMode);
  const systemDark = useSyncExternalStore(
    subscribeSystemTheme,
    getSystemSnapshot,
    getServerSnapshot,
  );

  const resolvedMode: ResolvedTheme = mode === 'system' ? (systemDark ? 'dark' : 'light') : mode;

  useEffect(() => {
    document.documentElement.dataset.theme = resolvedMode;
    document.documentElement.style.colorScheme = resolvedMode;
    const meta = document.querySelector<HTMLMetaElement>('meta[name="theme-color"]');
    if (meta) {
      meta.content = THEME_COLORS[resolvedMode];
    }
  }, [resolvedMode]);

  const setMode = useCallback((next: ThemeMode) => {
    if (typeof window !== 'undefined') {
      try {
        window.localStorage.setItem(STORAGE_KEY, next);
      } catch {
        // Almacenamiento bloqueado: se aplica el tema sin persistir.
      }
    }
    setModeState(next);
  }, []);

  const value = useMemo<ThemeContextValue>(
    () => ({ mode, resolvedMode, setMode }),
    [mode, resolvedMode, setMode],
  );

  return <ThemeContext.Provider value={value}>{children}</ThemeContext.Provider>;
}

// El hook se co-loca con el provider para mantener una única API pública.
// eslint-disable-next-line react-refresh/only-export-components
export function useTheme(): ThemeContextValue {
  const context = useContext(ThemeContext);
  if (!context) {
    throw new Error('useTheme debe usarse dentro de <ThemeProvider>');
  }
  return context;
}
