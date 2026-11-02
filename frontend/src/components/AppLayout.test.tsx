import { describe, expect, it, vi, afterEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import { MemoryRouter, Route, Routes } from 'react-router-dom';
import { AppLayout } from './AppLayout';

vi.mock('../lib/config', () => ({
  VALET_URL: 'http://valet.test',
  hasValet: true,
}));

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
});
