import { useState } from 'react';
import { Alert, Button, InputNumber, Segmented, Typography } from 'antd';
import { useSession } from '../auth/session';
import { useSettings, useUpdateSettings } from '../api/hooks';
import { useTheme } from '../theme/ThemeProvider';
import type { ThemeMode } from '../theme/ThemeProvider';
import './SettingsPage.css';

const { Title, Paragraph } = Typography;

const THEME_OPTIONS: { label: string; value: ThemeMode }[] = [
  { label: 'Oscuro', value: 'dark' },
  { label: 'Claro', value: 'light' },
  { label: 'Sistema', value: 'system' },
];

const MIN_SERVINGS = 1;
const MAX_SERVINGS = 20;
const FALLBACK_SERVINGS = 2;

/** Página de ajustes: tema, comensales habituales e identidad de la sesión. */
export function SettingsPage() {
  const { mode, setMode } = useTheme();
  const { name } = useSession();
  const { data: settings, isLoading, isError, error: settingsError } = useSettings();
  const updateSettings = useUpdateSettings();

  const current = settings?.default_servings ?? FALLBACK_SERVINGS;
  // `null` mientras no se toca el campo: así el valor mostrado sigue al del
  // servidor aunque la consulta resuelva después del primer render.
  const [draft, setDraft] = useState<number | null>(null);
  const [saved, setSaved] = useState(false);

  const value = draft ?? current;
  const inRange = value >= MIN_SERVINGS && value <= MAX_SERVINGS;
  const changed = draft !== null && draft !== current;
  const canSave = changed && inRange && !isLoading && !isError && !updateSettings.isPending;

  const handleChange = (next: number | null) => {
    setDraft(typeof next === 'number' ? next : null);
    setSaved(false);
    // Al editar se descarta el error anterior del servidor.
    updateSettings.reset();
  };

  const handleSave = () => {
    if (draft === null || !inRange) {
      return;
    }
    setSaved(false);
    updateSettings.mutate(
      { default_servings: draft },
      {
        onSuccess: () => {
          setSaved(true);
          setDraft(null);
        },
      },
    );
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
        <h3 className="settings-page__section-title">Comensales habituales</h3>
        <Paragraph type="secondary" className="settings-page__hint">
          Raciones por defecto al abrir una receta (entre 1 y 20).
        </Paragraph>
        <div className="settings-page__servings">
          <InputNumber
            className="settings-page__servings-input"
            min={MIN_SERVINGS}
            max={MAX_SERVINGS}
            value={value}
            onChange={handleChange}
            disabled={isLoading || isError}
            aria-label="Comensales habituales"
          />
          <Button
            type="primary"
            className="settings-page__save"
            disabled={!canSave}
            loading={updateSettings.isPending}
            onClick={handleSave}
          >
            Guardar
          </Button>
        </div>
        {isError && (
          <Alert
            type="warning"
            showIcon
            className="settings-page__error"
            message={
              settingsError instanceof Error
                ? settingsError.message
                : 'No se pudo cargar la preferencia'
            }
          />
        )}
        {saved && (
          <span className="settings-page__saved" role="status">
            Preferencia guardada
          </span>
        )}
        {updateSettings.isError && (
          <Alert
            type="error"
            showIcon
            className="settings-page__error"
            message={
              updateSettings.error instanceof Error
                ? updateSettings.error.message
                : 'No se pudo guardar la preferencia'
            }
          />
        )}
      </div>

      <div className="settings-page__section">
        <h3 className="settings-page__section-title">Sesión</h3>
        <span className="settings-page__name">{name ?? 'Sesión'}</span>
      </div>
    </section>
  );
}
