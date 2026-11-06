import { describe, expect, it, vi, afterEach, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import { MemoryRouter, Route, Routes } from 'react-router-dom';
import { AppLayout } from './AppLayout';
import { useSession, type SessionContextValue } from '../auth/session';
import { useMediaQuery } from '../hooks/useMediaQuery';

vi.mock('../lib/config', () => ({
  VALET_URL: 'http://valet.test',
  hasValet: true,
}));

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
    mockedUseSession.mockReturnValue(session());
    mockedUseMediaQuery.mockReturnValue(true);
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('muestra la barra superior única con el logo', () => {
    renderLayout();

    expect(screen.getByTestId('app-header')).toBeInTheDocument();
    expect(screen.getByText(/OxiMeal/)).toBeInTheDocument();
  });

  it('no renderiza el Sider ni la navegación inferior', () => {
    renderLayout();

    expect(screen.queryByTestId('app-sider')).not.toBeInTheDocument();
    expect(screen.queryByTestId('bottom-nav')).not.toBeInTheDocument();
  });

  it('renderiza el contenido de la ruta', () => {
    renderLayout();

    expect(screen.getByText('contenido')).toBeInTheDocument();
  });

  it('en móvil también usa la barra superior sin navegación inferior', () => {
    mockedUseMediaQuery.mockReturnValue(false);
    renderLayout();

    expect(screen.getByTestId('app-header')).toBeInTheDocument();
    expect(screen.queryByTestId('app-sider')).not.toBeInTheDocument();
    expect(screen.queryByTestId('bottom-nav')).not.toBeInTheDocument();
  });
});
