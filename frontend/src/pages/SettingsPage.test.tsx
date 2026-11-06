import { describe, expect, it, vi, beforeEach, afterEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { MemoryRouter, Route, Routes } from 'react-router-dom';
import { SettingsPage } from './SettingsPage';
import { useSession, type SessionContextValue } from '../auth/session';
import { useTheme } from '../theme/ThemeProvider';

vi.mock('../auth/session', () => ({
  useSession: vi.fn(),
}));

vi.mock('../theme/ThemeProvider', () => ({
  useTheme: vi.fn(),
}));

const mockedUseSession = vi.mocked(useSession);
const mockedUseTheme = vi.mocked(useTheme);

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

  beforeEach(() => {
    setMode.mockReset();
    mockedUseSession.mockReturnValue(session());
    mockedUseTheme.mockReturnValue({
      mode: 'dark',
      resolvedMode: 'dark',
      setMode,
    });
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

  it('no muestra ningún botón ni control de cierre de sesión', () => {
    renderSettings();

    expect(
      screen.queryByRole('button', { name: /salir|cerrar sesión/i }),
    ).not.toBeInTheDocument();
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
});
