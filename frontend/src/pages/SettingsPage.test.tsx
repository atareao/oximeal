import { describe, expect, it, vi, beforeEach, afterEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { MemoryRouter, Route, Routes } from 'react-router-dom';
import { SettingsPage } from './SettingsPage';
import { useSession, type SessionContextValue } from '../auth/session';
import { useTheme } from '../theme/ThemeProvider';
import { useSettings, useUpdateSettings } from '../api/hooks';

vi.mock('../auth/session', () => ({
  useSession: vi.fn(),
}));

vi.mock('../theme/ThemeProvider', () => ({
  useTheme: vi.fn(),
}));

vi.mock('../api/hooks', () => ({
  useSettings: vi.fn(),
  useUpdateSettings: vi.fn(),
}));

const mockedUseSession = vi.mocked(useSession);
const mockedUseTheme = vi.mocked(useTheme);
const mockedUseSettings = vi.mocked(useSettings);
const mockedUseUpdateSettings = vi.mocked(useUpdateSettings);

function session(overrides: Partial<SessionContextValue> = {}): SessionContextValue {
  return {
    status: 'authenticated',
    name: 'Ana',
    permissions: [],
    logout: vi.fn(),
    ...overrides,
  };
}

function renderSettings() {
  return render(
    <MemoryRouter initialEntries={['/settings']}>
      <Routes>
        <Route path="/settings" element={<SettingsPage />} />
      </Routes>
    </MemoryRouter>,
  );
}

describe('SettingsPage', () => {
  const setMode = vi.fn();
  const mutate = vi.fn();
  const reset = vi.fn();

  beforeEach(() => {
    setMode.mockReset();
    mutate.mockReset();
    reset.mockReset();
    mockedUseSession.mockReturnValue(session());
    mockedUseTheme.mockReturnValue({
      mode: 'dark',
      resolvedMode: 'dark',
      setMode,
    });
    mockedUseSettings.mockReturnValue({
      data: { default_servings: 6 },
      isLoading: false,
      isError: false,
      error: null,
    } as unknown as ReturnType<typeof useSettings>);
    mockedUseUpdateSettings.mockReturnValue({
      mutate,
      reset,
      isPending: false,
      isError: false,
      error: null,
    } as unknown as ReturnType<typeof useUpdateSettings>);
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('muestra el selector de tema con las tres opciones', () => {
    renderSettings();

    expect(screen.getByText('Oscuro')).toBeInTheDocument();
    expect(screen.getByText('Claro')).toBeInTheDocument();
    expect(screen.getByText('Sistema')).toBeInTheDocument();
  });

  it('cambiar a Claro llama a setMode con light', async () => {
    const user = userEvent.setup();
    renderSettings();

    await user.click(screen.getByText('Claro'));

    expect(setMode).toHaveBeenCalledWith('light');
  });

  it('muestra el nombre del usuario de la sesión', () => {
    renderSettings();

    expect(screen.getByText('Ana')).toBeInTheDocument();
  });

  it('sin sesión autenticada no muestra la sección de identidad', () => {
    mockedUseSession.mockReturnValue(session({ status: 'disabled', name: undefined }));

    renderSettings();

    expect(screen.queryByText('Sesión')).not.toBeInTheDocument();
  });

  it('expone el encabezado de la vista oculto para lectores de pantalla', () => {
    renderSettings();

    expect(screen.getByRole('heading', { name: 'Configuración' })).toHaveClass('visually-hidden');
  });

  it('no muestra ningún botón ni control de cierre de sesión', () => {
    renderSettings();

    expect(screen.queryByRole('button', { name: /salir|cerrar sesión/i })).not.toBeInTheDocument();
    expect(document.querySelector('.settings-page__logout')).toBeNull();

    const exitPattern = /salir|cerrar\s+sesi[oó]n|logout|log\s*out|sign\s*out/i;
    const exitControls = screen
      .queryAllByRole('button')
      .filter((element) => exitPattern.test(element.textContent ?? ''))
      .concat(
        Array.from(
          document.querySelectorAll<HTMLElement>(
            '[role="button"], a, [data-testid*="logout"], [data-testid*="salir"]',
          ),
        ).filter((element) => exitPattern.test(element.textContent ?? '')),
      );

    expect(exitControls).toHaveLength(0);
  });

  it('carga los comensales habituales con el valor del servidor', () => {
    renderSettings();

    expect(screen.getByRole('spinbutton', { name: /comensales habituales/i })).toHaveValue('6');
  });

  it('usa 2 cuando el servidor no devuelve la preferencia', () => {
    mockedUseSettings.mockReturnValue({
      data: undefined,
      isLoading: false,
      isError: false,
      error: null,
    } as unknown as ReturnType<typeof useSettings>);

    renderSettings();

    expect(screen.getByRole('spinbutton', { name: /comensales habituales/i })).toHaveValue('2');
  });

  it('deshabilita el campo y el botón mientras carga la preferencia', () => {
    mockedUseSettings.mockReturnValue({
      data: undefined,
      isLoading: true,
      isError: false,
      error: null,
    } as unknown as ReturnType<typeof useSettings>);

    renderSettings();

    expect(screen.getByRole('spinbutton', { name: /comensales habituales/i })).toBeDisabled();
    expect(screen.getByRole('button', { name: /guardar/i })).toBeDisabled();
  });

  it('muestra un aviso visible si falla la carga de la preferencia', () => {
    mockedUseSettings.mockReturnValue({
      data: undefined,
      isLoading: false,
      isError: true,
      error: new Error('No se pudo cargar la preferencia'),
    } as unknown as ReturnType<typeof useSettings>);

    renderSettings();

    expect(screen.getByText(/no se pudo cargar la preferencia/i)).toBeInTheDocument();
  });

  it('guarda el nuevo valor de comensales', async () => {
    const user = userEvent.setup();
    renderSettings();

    const input = screen.getByRole('spinbutton', { name: /comensales habituales/i });
    await user.clear(input);
    await user.type(input, '4');
    await user.click(screen.getByRole('button', { name: /guardar/i }));

    expect(mutate).toHaveBeenCalledTimes(1);
    expect(mutate.mock.calls[0][0]).toEqual({ default_servings: 4 });
  });

  it('muestra confirmación tras guardar con éxito', async () => {
    mutate.mockImplementation((_input, options?: { onSuccess?: () => void }) => {
      options?.onSuccess?.();
    });
    const user = userEvent.setup();
    renderSettings();

    const input = screen.getByRole('spinbutton', { name: /comensales habituales/i });
    await user.clear(input);
    await user.type(input, '4');
    await user.click(screen.getByRole('button', { name: /guardar/i }));

    expect(await screen.findByText(/preferencia guardada/i)).toBeInTheDocument();
  });

  it('limpia el error del servidor al cambiar el valor', async () => {
    const user = userEvent.setup();
    renderSettings();

    const input = screen.getByRole('spinbutton', { name: /comensales habituales/i });
    await user.clear(input);
    await user.type(input, '4');

    expect(reset).toHaveBeenCalled();
  });

  it('deshabilita Guardar mientras no cambia el valor', () => {
    renderSettings();

    expect(screen.getByRole('button', { name: /guardar/i })).toBeDisabled();
  });

  it('deshabilita Guardar si la preferencia cargada está fuera de rango', () => {
    mockedUseSettings.mockReturnValue({
      data: { default_servings: 21 },
      isLoading: false,
      isError: false,
      error: null,
    } as unknown as ReturnType<typeof useSettings>);

    renderSettings();

    expect(screen.getByRole('button', { name: /guardar/i })).toBeDisabled();
  });

  it('muestra el error del servidor (422) sin toast global', () => {
    mockedUseUpdateSettings.mockReturnValue({
      mutate,
      isPending: false,
      isError: true,
      error: new Error('default_servings debe estar entre 1 y 20'),
    } as unknown as ReturnType<typeof useUpdateSettings>);

    renderSettings();

    expect(screen.getByText(/default_servings debe estar entre 1 y 20/i)).toBeInTheDocument();
  });
});
