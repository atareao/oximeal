import { describe, expect, it, vi, beforeEach, afterEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { MemoryRouter, Route, Routes, useLocation } from 'react-router-dom';
import { AppHeader } from './AppHeader';
import { useSession, type SessionContextValue } from '../auth/session';
import { useMediaQuery } from '../hooks/useMediaQuery';
import { reloadApp } from '../auth/reload';

vi.mock('../auth/session', () => ({
  useSession: vi.fn(),
}));

vi.mock('../hooks/useMediaQuery', () => ({
  useMediaQuery: vi.fn(),
}));

vi.mock('../auth/reload', () => ({
  reloadApp: vi.fn(),
}));

const mockedUseSession = vi.mocked(useSession);
const mockedUseMediaQuery = vi.mocked(useMediaQuery);
const mockedReloadApp = vi.mocked(reloadApp);

function session(overrides: Partial<SessionContextValue> = {}): SessionContextValue {
  return {
    status: 'authenticated',
    name: 'Ana',
    permissions: [],
    logout: vi.fn(),
    ...overrides,
  };
}

function LocationProbe() {
  const location = useLocation();
  return <div data-testid="location">{location.pathname}</div>;
}

/**
 * Configura `useMediaQuery` por consulta: `wide` controla la query compacta
 * `(min-width: 480px)` y `desktop` el resto (la query de escritorio de 768px).
 */
function mockViewport({ desktop, wide }: { desktop: boolean; wide: boolean }) {
  mockedUseMediaQuery.mockImplementation((query: string) =>
    query === '(min-width: 480px)' ? wide : desktop,
  );
}

function renderHeader(initialEntry = '/') {
  return render(
    <MemoryRouter initialEntries={[initialEntry]}>
      <LocationProbe />
      <Routes>
        <Route path="*" element={<AppHeader />} />
      </Routes>
    </MemoryRouter>,
  );
}

describe('AppHeader', () => {
  beforeEach(() => {
    mockedUseSession.mockReturnValue(session());
    mockedUseMediaQuery.mockReturnValue(true);
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('muestra el logo OxiMeal y las entradas principales', () => {
    renderHeader();

    expect(screen.getByText(/OxiMeal/)).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Menú' })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Recetas' })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Compra' })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Configuración' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Salir' })).toBeInTheDocument();
  });

  it('no renderiza el Sider ni la navegación inferior', () => {
    renderHeader();

    expect(screen.queryByTestId('app-sider')).not.toBeInTheDocument();
    expect(screen.queryByTestId('bottom-nav')).not.toBeInTheDocument();
  });

  it('en móvil muestra solo iconos con nombre accesible', () => {
    mockedUseMediaQuery.mockReturnValue(false);
    renderHeader();

    expect(screen.getByRole('link', { name: 'Menú' })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Recetas' })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Compra' })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'Configuración' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Salir' })).toBeInTheDocument();

    // Sin texto visible: el nombre accesible viene del aria-label del icono.
    expect(screen.queryByText('Menú')).not.toBeInTheDocument();
    expect(screen.queryByText('Recetas')).not.toBeInTheDocument();
    expect(screen.queryByText('Compra')).not.toBeInTheDocument();
    expect(screen.queryByText('Configuración')).not.toBeInTheDocument();
  });

  it('en desktop muestra icono y texto', () => {
    mockedUseMediaQuery.mockReturnValue(true);
    renderHeader();

    expect(screen.getByText('Menú')).toBeInTheDocument();
    expect(screen.getByText('Recetas')).toBeInTheDocument();
    expect(screen.getByText('Compra')).toBeInTheDocument();
    expect(screen.getByText('Configuración')).toBeInTheDocument();
  });

  it('marca la entrada de la ruta activa con aria-current', () => {
    renderHeader('/recipes');

    expect(screen.getByRole('link', { name: 'Recetas' })).toHaveAttribute('aria-current', 'page');
    expect(screen.getByRole('link', { name: 'Menú' })).not.toHaveAttribute('aria-current');
    expect(screen.getByRole('link', { name: 'Compra' })).not.toHaveAttribute('aria-current');
  });

  it('navega a /recipes y a /settings', async () => {
    const user = userEvent.setup();
    renderHeader();

    await user.click(screen.getByRole('link', { name: 'Recetas' }));
    expect(screen.getByTestId('location')).toHaveTextContent('/recipes');

    await user.click(screen.getByRole('link', { name: 'Configuración' }));
    expect(screen.getByTestId('location')).toHaveTextContent('/settings');
  });

  it('al pulsar Salir invoca el logout de la sesión', async () => {
    const logout = vi.fn().mockResolvedValue(undefined);
    mockedUseSession.mockReturnValue(session({ logout }));
    const user = userEvent.setup();
    renderHeader();

    await user.click(screen.getByRole('button', { name: 'Salir' }));

    expect(logout).toHaveBeenCalledTimes(1);
    expect(mockedReloadApp).not.toHaveBeenCalled();
  });

  it('todos los controles interactivos tienen altura mínima de 48px', () => {
    renderHeader();

    const controls = [
      ...screen.getAllByRole('link'),
      screen.getByRole('button', { name: 'Salir' }),
    ];

    for (const control of controls) {
      expect(control).toHaveStyle({ minHeight: '48px' });
    }
  });

  it('bajo 480px muestra el logo compacto sin el texto OxiMeal pero con nombre accesible', () => {
    mockViewport({ desktop: false, wide: false });
    renderHeader();

    expect(screen.queryByText('OxiMeal')).not.toBeInTheDocument();
    expect(screen.getByRole('link', { name: 'OxiMeal' })).toBeInTheDocument();
  });

  it('Configuración y Salir comparten el grupo de acciones y Salir es el último control', () => {
    mockViewport({ desktop: false, wide: false });
    renderHeader();

    const settings = screen.getByRole('link', { name: 'Configuración' });
    const logout = screen.getByRole('button', { name: 'Salir' });
    const actions = settings.closest('.app-header__actions');

    expect(actions).not.toBeNull();
    expect(actions).toBe(logout.closest('.app-header__actions'));
    expect(actions?.lastElementChild).toBe(logout);
  });

  it('Salir está presente en todos los estados de sesión', () => {
    for (const status of ['loading', 'disabled', 'anonymous', 'degraded'] as const) {
      mockedUseSession.mockReturnValue(session({ status }));
      mockViewport({ desktop: false, wide: false });

      const { unmount } = renderHeader();
      expect(screen.getByRole('button', { name: 'Salir' })).toBeInTheDocument();
      unmount();
    }
  });

  it('sin sesión autenticada, pulsar Salir recarga la app y no invoca el logout', async () => {
    const logout = vi.fn().mockResolvedValue(undefined);
    mockedUseSession.mockReturnValue(session({ status: 'disabled', logout }));
    const user = userEvent.setup();
    renderHeader();

    await user.click(screen.getByRole('button', { name: 'Salir' }));

    expect(mockedReloadApp).toHaveBeenCalledTimes(1);
    expect(logout).not.toHaveBeenCalled();
  });

  it('con sesión autenticada y viewport estrecho la acción agrupa Configuración y Salir sin texto OxiMeal', () => {
    mockViewport({ desktop: false, wide: false });
    renderHeader();

    const settings = screen.getByRole('link', { name: 'Configuración' });
    const logout = screen.getByRole('button', { name: 'Salir' });
    const actions = settings.closest('.app-header__actions');

    expect(actions).not.toBeNull();
    expect(actions).toBe(logout.closest('.app-header__actions'));
    expect(actions).toContainElement(settings);
    expect(actions).toContainElement(logout);
    expect(screen.queryByText('OxiMeal')).not.toBeInTheDocument();
  });

  it('en viewport ancho el logo muestra el texto OxiMeal y no la marca compacta', () => {
    mockViewport({ desktop: true, wide: true });
    renderHeader();

    expect(screen.getByText('OxiMeal')).toBeInTheDocument();
    expect(document.querySelector('.app-header__brand-mark')).not.toBeInTheDocument();
  });
});
