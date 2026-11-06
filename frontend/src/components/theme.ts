import { theme as antdTheme } from 'antd';
import type { ThemeConfig } from 'antd';
import type { ResolvedTheme } from '../theme/ThemeProvider';

const FONT_FAMILY =
  "system-ui, -apple-system, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif";

const sharedToken: ThemeConfig['token'] = {
  colorPrimary: '#2f855a',
  colorInfo: '#2f855a',
  colorSuccess: '#38a169',
  colorWarning: '#d69e2e',
  colorError: '#e53e3e',
  borderRadius: 10,
  controlHeight: 48,
  fontSize: 16,
  fontFamily: FONT_FAMILY,
};

/**
 * Tokens de tema de Ant Design para OxiMeal (mobile-first, táctil).
 *
 * `controlHeight` es 48px para cumplir el objetivo táctil mínimo en móvil.
 * La familia tipográfica usa la pila del sistema: no se carga ninguna fuente
 * web, así que no se declara `'Inter'` (evita un fallback silencioso).
 */
export const lightTheme: ThemeConfig = {
  algorithm: antdTheme.defaultAlgorithm,
  token: {
    ...sharedToken,
    colorBgBase: '#f7fafc',
    colorTextBase: '#1a202c',
  },
  components: {
    Layout: {
      bodyBg: '#f7fafc',
      headerBg: '#ffffff',
      siderBg: '#ffffff',
    },
    Button: {
      controlHeight: 48,
    },
  },
};

export const darkTheme: ThemeConfig = {
  algorithm: antdTheme.darkAlgorithm,
  token: {
    ...sharedToken,
    colorBgBase: '#0f172a',
    colorTextBase: '#e2e8f0',
  },
  components: {
    Layout: {
      bodyBg: '#0f172a',
      headerBg: '#111827',
      siderBg: '#111827',
    },
    Button: {
      controlHeight: 48,
    },
  },
};

/** Devuelve la configuración de Ant Design para el modo resuelto. */
export function getTheme(resolvedMode: ResolvedTheme): ThemeConfig {
  return resolvedMode === 'dark' ? darkTheme : lightTheme;
}
