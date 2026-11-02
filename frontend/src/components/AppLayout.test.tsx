import { describe, expect, it, vi, afterEach, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { MemoryRouter, Route, Routes } from 'react-router-dom';
import { AppLayout } from './AppLayout';
import { useSession, type SessionContextValue } from '../auth/session';

vi.mock('../lib/config', () => ({
  VALET_URL: 'http://valet.test',
  hasValet: true,
}));

vi.mock('../auth/session', () => ({
  useSession: vi.fn(),
}));

const mockedUseSession = vi.mocked(useSession);

function session(overrides: Partial<SessionContextValue>): SessionContextValue {
  return {
    status: 'disabled',
    permissions: [],
    logout: vi.fn(),
    ...overrides,
  };
}

function installMatchMedia(matches: boolean) {
  window.matchMedia = vi.fn().mockImplementation((query: string) => ({
    matches,
    media: query,
    onchange: null,
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    addListener: vi.fn(),
    removeListener: vi.fn(),
    dispatchEvent: vi.fn(),
  }));
}

function mockLocationAssign() {
  const assign = vi.fn();
  Object.defineProperty(window, 'location', {
    configurable: true,
    writable: true,
    value: { assign },
  });
  return assign;
}

function renderLayout() {
  return render(
    <MemoryRouter initialEntries={['/']}>
      <Routes>
        <Route element={<AppLayout />}>
          <Route index element={<div>contenido</div>} />
        </Route>
      </Routes>
    </MemoryRouter>,
  );
}

describe('AppLayout', () => {
  beforeEach(() => {
    mockedUseSession.mockReset();
    mockedUseSession.mockReturnValue(session({}));
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('en móvil muestra la navegación inferior y oculta el Sider', () => {
    installMatchMedia(false);
    renderLayout();

    expect(screen.getByTestId('bottom-nav')).toBeInTheDocument();
    expect(screen.queryByTestId('app-sider')).not.toBeInTheDocument();
    expect(screen.getByText('contenido')).toBeInTheDocument();
  });

  it('en desktop muestra el Sider y oculta la navegación inferior', () => {
    installMatchMedia(true);
    renderLayout();

    expect(screen.getByTestId('app-sider')).toBeInTheDocument();
    expect(screen.queryByTestId('bottom-nav')).not.toBeInTheDocument();
    expect(screen.getByText('Asistente Valet')).toBeInTheDocument();
  });

  it('en modo abierto (disabled) no muestra identidad ni cierre de sesión', () => {
    installMatchMedia(true);
    renderLayout();

    expect(screen.queryByTestId('session-identity')).not.toBeInTheDocument();
    expect(screen.queryByTestId('logout-button')).not.toBeInTheDocument();
  });

  it('muestra el nombre del usuario cuando hay sesión (desktop)', () => {
    installMatchMedia(true);
    mockedUseSession.mockReturnValue(session({ status: 'authenticated', name: 'Ana' }));
    renderLayout();

    expect(screen.getByTestId('session-identity')).toHaveTextContent('Ana');
  });

  it('muestra el nombre del usuario cuando hay sesión (móvil)', () => {
    installMatchMedia(false);
    mockedUseSession.mockReturnValue(session({ status: 'authenticated', name: 'Ana' }));
    renderLayout();

    expect(screen.getByTestId('session-identity')).toHaveTextContent('Ana');
  });

  it('al pulsar cerrar sesión invoca logout y delega la redirección en AuthGuard', async () => {
    installMatchMedia(true);
    const assign = mockLocationAssign();
    const logout = vi.fn().mockResolvedValue(undefined);
    mockedUseSession.mockReturnValue(session({ status: 'authenticated', name: 'Ana', logout }));
    const user = userEvent.setup();
    renderLayout();

    await user.click(screen.getByTestId('logout-button'));

    expect(logout).toHaveBeenCalledTimes(1);
    // Sin doble navegación: la redirección a /auth/login la hace AuthGuard
    // cuando la sesión pasa a anónima, no AppLayout.
    expect(assign).not.toHaveBeenCalled();
  });

  it('muestra la sección Recetas en el Sider y navega a /recipes', async () => {
    installMatchMedia(true);
    const user = userEvent.setup();
    render(
      <MemoryRouter initialEntries={['/']}>
        <Routes>
          <Route element={<AppLayout />}>
            <Route index element={<div>contenido</div>} />
            <Route path="recipes" element={<div>catálogo de recetas</div>} />
          </Route>
        </Routes>
      </MemoryRouter>,
    );

    await user.click(screen.getByText(/Recetas/));

    expect(screen.getByText('catálogo de recetas')).toBeInTheDocument();
  });

  it('marca Recetas como activa en /recipes', () => {
    installMatchMedia(true);
    render(
      <MemoryRouter initialEntries={['/recipes']}>
        <Routes>
          <Route element={<AppLayout />}>
            <Route path="recipes" element={<div>catálogo de recetas</div>} />
          </Route>
        </Routes>
      </MemoryRouter>,
    );

    const selected = document.querySelector('.ant-menu-item-selected');
    expect(selected).toHaveTextContent('Recetas');
  });
});
