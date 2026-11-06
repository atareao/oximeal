import { Button, Segmented, Typography } from 'antd';
import { useSession } from '../auth/session';
import { useTheme } from '../theme/ThemeProvider';
import type { ThemeMode } from '../theme/ThemeProvider';
import './SettingsPage.css';

const { Title, Paragraph } = Typography;

const THEME_OPTIONS: { label: string; value: ThemeMode }[] = [
  { label: 'Oscuro', value: 'dark' },
  { label: 'Claro', value: 'light' },
  { label: 'Sistema', value: 'system' },
];

/** Página de ajustes: selector de tema e identidad de la sesión. */
export function SettingsPage() {
  const { mode, setMode } = useTheme();
  const { name, logout } = useSession();

  const handleLogout = () => {
    void logout();
  };

  return (
    <section className="settings-page" aria-labelledby="settings-title">
      <Title id="settings-title" level={2} className="settings-page__title">
        Configuración
      </Title>

      <div className="settings-page__section">
        <h3 className="settings-page__section-title">Tema</h3>
        <Paragraph type="secondary" className="settings-page__hint">
          Elige cómo se muestra OxiMeal.
        </Paragraph>
        <Segmented
          className="settings-page__theme"
          options={THEME_OPTIONS}
          value={mode}
          onChange={(value) => setMode(value as ThemeMode)}
          block
        />
      </div>

      <div className="settings-page__section">
        <h3 className="settings-page__section-title">Cuenta</h3>
        <div className="settings-page__identity">
          <span className="settings-page__name">{name ?? 'Sesión'}</span>
          <Button
            danger
            className="settings-page__logout"
            onClick={handleLogout}
            aria-label="Salir"
          >
            Salir
          </Button>
        </div>
      </div>
    </section>
  );
}
