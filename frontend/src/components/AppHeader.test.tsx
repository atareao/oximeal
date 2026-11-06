import { describe, expect, it, vi, beforeEach, afterEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { MemoryRouter, Route, Routes, useLocation } from 'react-router-dom';
import { AppHeader } from './AppHeader';
import { useSession, type SessionContextValue } from '../auth/session';
import { useMediaQuery } from '../hooks/useMediaQuery';

vi.mock('../auth/session', () => ({
  useSession: vi.fn(),
}));

vi.mock('../hooks/useMediaQuery', () => ({
  useMediaQuery: vi.fn(),
}));

const mockedUseSession = vi.mocked(useSession);
const mockedUseMediaQuery = vi.mocked(useMediaQuery);

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
});
