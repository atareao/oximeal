import { describe, expect, it, vi, beforeEach, afterEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import { AuthGuard } from './AuthGuard';
import { useSession, type SessionContextValue } from './session';

vi.mock('./session', () => ({
  useSession: vi.fn(),
}));

const mockedUseSession = vi.mocked(useSession);

function session(overrides: Partial<SessionContextValue>): SessionContextValue {
  return {
    status: 'loading',
    permissions: [],
    logout: vi.fn(),
    ...overrides,
  };
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

function renderGuard(initialPath = '/') {
  return render(
    <MemoryRouter initialEntries={[initialPath]}>
      <AuthGuard>
        <div data-testid="protected">datos</div>
      </AuthGuard>
    </MemoryRouter>,
  );
}

describe('AuthGuard', () => {
  beforeEach(() => {
    mockedUseSession.mockReset();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('muestra carga y no renderiza los datos mientras la sesión es loading', () => {
    mockedUseSession.mockReturnValue(session({ status: 'loading' }));
    renderGuard();

    expect(screen.getByTestId('auth-loading')).toBeInTheDocument();
    expect(screen.queryByTestId('protected')).not.toBeInTheDocument();
  });

  it('redirige a /auth/login conservando el destino cuando es anónimo', () => {
    const assign = mockLocationAssign();
    mockedUseSession.mockReturnValue(session({ status: 'anonymous' }));
    renderGuard('/shopping');

    expect(assign).toHaveBeenCalledWith('/auth/login?return_to=%2Fshopping');
    expect(screen.queryByTestId('protected')).not.toBeInTheDocument();
  });

  it('renderiza los datos cuando hay sesión autenticada', () => {
    mockedUseSession.mockReturnValue(session({ status: 'authenticated', name: 'Ana' }));
    renderGuard();

    expect(screen.getByTestId('protected')).toBeInTheDocument();
  });

  it('renderiza los datos en modo abierto (auth disabled)', () => {
    mockedUseSession.mockReturnValue(session({ status: 'disabled' }));
    renderGuard();

    expect(screen.getByTestId('protected')).toBeInTheDocument();
  });
});
