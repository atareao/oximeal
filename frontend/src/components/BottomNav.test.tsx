import { describe, expect, it, vi, afterEach, beforeEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { MemoryRouter, Route, Routes, useLocation } from 'react-router-dom';
import { BottomNav } from './BottomNav';

const configState = vi.hoisted(() => ({ hasValet: true, url: 'http://valet.test' }));

vi.mock('../lib/config', () => ({
  get VALET_URL() {
    return configState.url;
  },
  get hasValet() {
    return configState.hasValet;
  },
}));

function LocationProbe() {
  const location = useLocation();
  return <div data-testid="location">{location.pathname}</div>;
}

function renderNav() {
  return render(
    <MemoryRouter initialEntries={['/']}>
      <Routes>
        <Route
          path="*"
          element={
            <>
              <LocationProbe />
              <BottomNav />
            </>
          }
        />
      </Routes>
    </MemoryRouter>,
  );
}

describe('BottomNav', () => {
  beforeEach(() => {
    configState.hasValet = true;
    configState.url = 'http://valet.test';
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('muestra las tres pestañas cuando hay Valet', () => {
    renderNav();

    expect(screen.getByRole('button', { name: /menú/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /compra/i })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: /asistente/i })).toBeInTheDocument();
  });

  it('oculta la pestaña Asistente cuando no hay Valet', () => {
    configState.hasValet = false;
    renderNav();

    expect(screen.queryByRole('link', { name: /asistente/i })).not.toBeInTheDocument();
  });

  it('navega a /shopping al pulsar Compra', async () => {
    const user = userEvent.setup();
    renderNav();

    expect(screen.getByTestId('location')).toHaveTextContent('/');

    await user.click(screen.getByRole('button', { name: /compra/i }));

    expect(screen.getByTestId('location')).toHaveTextContent('/shopping');
  });

  it('enlaza a Valet en una pestaña nueva con rel seguro', () => {
    renderNav();

    const link = screen.getByRole('link', { name: /asistente/i });
    expect(link).toHaveAttribute('href', 'http://valet.test');
    expect(link).toHaveAttribute('target', '_blank');
    expect(link).toHaveAttribute('rel', 'noopener noreferrer');
  });
});
