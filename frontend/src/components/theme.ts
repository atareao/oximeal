import type { ThemeConfig } from 'antd';

/**
 * Tokens de tema de Ant Design para OxiMeal (mobile-first, táctil).
 *
 * `controlHeight` es 48px para cumplir el objetivo táctil mínimo en móvil.
 * La familia tipográfica usa la pila del sistema: no se carga ninguna fuente
 * web, así que no se declara `'Inter'` (evita un fallback silencioso).
 */
export const theme: ThemeConfig = {
  token: {
    colorPrimary: '#2f855a',
    colorInfo: '#2f855a',
    colorSuccess: '#38a169',
    colorWarning: '#d69e2e',
    colorError: '#e53e3e',
    borderRadius: 10,
    controlHeight: 48,
    fontSize: 16,
    fontFamily:
      "system-ui, -apple-system, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif",
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
