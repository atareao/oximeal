import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import { OfflineBanner } from './OfflineBanner';
import { useSession, type SessionContextValue } from '../auth/session';
import { useOnline } from '../offline/connectivity';

vi.mock('../auth/session', () => ({
  useSession: vi.fn(),
}));

vi.mock('../offline/connectivity', () => ({
  useOnline: vi.fn(),
}));

const mockedUseSession = vi.mocked(useSession);
const mockedUseOnline = vi.mocked(useOnline);

function session(overrides: Partial<SessionContextValue>): SessionContextValue {
  return {
    status: 'authenticated',
    permissions: [],
    logout: vi.fn(),
    ...overrides,
  };
}

describe('OfflineBanner', () => {
  beforeEach(() => {
    mockedUseSession.mockReset();
    mockedUseOnline.mockReset();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('muestra el aviso de sin conexión', () => {
    mockedUseOnline.mockReturnValue(false);
    mockedUseSession.mockReturnValue(session({ status: 'authenticated' }));

    render(<OfflineBanner />);

    expect(screen.getByTestId('offline-banner')).toHaveTextContent(
      'Sin conexión. Se muestran los datos guardados.',
    );
  });

  it('muestra el aviso de sesión caducada en modo degradado', () => {
    mockedUseOnline.mockReturnValue(true);
    mockedUseSession.mockReturnValue(session({ status: 'degraded' }));

    render(<OfflineBanner />);

    expect(screen.getByTestId('offline-banner')).toHaveTextContent(
      'Sesión caducada sin conexión. Se muestran los datos guardados.',
    );
  });

  it('no renderiza nada con conexión y sesión válida', () => {
    mockedUseOnline.mockReturnValue(true);
    mockedUseSession.mockReturnValue(session({ status: 'authenticated' }));

    render(<OfflineBanner />);

    expect(screen.queryByTestId('offline-banner')).not.toBeInTheDocument();
  });

  it('no renderiza nada con conexión y modo abierto', () => {
    mockedUseOnline.mockReturnValue(true);
    mockedUseSession.mockReturnValue(session({ status: 'disabled' }));

    render(<OfflineBanner />);

    expect(screen.queryByTestId('offline-banner')).not.toBeInTheDocument();
  });
});
