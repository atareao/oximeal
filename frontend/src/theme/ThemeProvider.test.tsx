import { describe, expect, it, vi, beforeEach, afterEach } from 'vitest';
import { act, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { ThemeProvider, useTheme } from './ThemeProvider';

type MediaListener = (event: MediaQueryListEvent) => void;

let systemDark = false;
let listeners: MediaListener[] = [];

function installMatchMedia(initialDark: boolean) {
  systemDark = initialDark;
  listeners = [];
  window.matchMedia = vi.fn().mockImplementation((query: string) => ({
    get matches() {
      return systemDark;
    },
    media: query,
    onchange: null,
    addEventListener: (_type: string, listener: MediaListener) => {
      listeners.push(listener);
    },
    removeEventListener: (_type: string, listener: MediaListener) => {
      listeners = listeners.filter((current) => current !== listener);
    },
    addListener: (listener: MediaListener) => {
      listeners.push(listener);
    },
    removeListener: (listener: MediaListener) => {
      listeners = listeners.filter((current) => current !== listener);
    },
    dispatchEvent: vi.fn(),
  }));
}

function emitSystemChange(dark: boolean) {
  systemDark = dark;
  act(() => {
    for (const listener of listeners) {
      listener({ matches: dark } as MediaQueryListEvent);
    }
  });
}

function Probe() {
  const { mode, resolvedMode, setMode } = useTheme();
  return (
    <div>
      <span data-testid="mode">{mode}</span>
      <span data-testid="resolved">{resolvedMode}</span>
      <button onClick={() => setMode('light')}>Claro</button>
      <button onClick={() => setMode('system')}>Sistema</button>
    </div>
  );
}

function renderProvider() {
  return render(
    <ThemeProvider>
      <Probe />
    </ThemeProvider>,
  );
}

function resetDocumentTheme() {
  document.documentElement.removeAttribute('data-theme');
  document.documentElement.style.colorScheme = '';
}

describe('ThemeProvider', () => {
  beforeEach(() => {
    localStorage.clear();
    resetDocumentTheme();
    installMatchMedia(false);
  });

  afterEach(() => {
    localStorage.clear();
    resetDocumentTheme();
    vi.restoreAllMocks();
  });

  it('sin preferencia guardada resuelve oscuro por defecto', () => {
    renderProvider();

    expect(screen.getByTestId('resolved')).toHaveTextContent('dark');
    expect(document.documentElement).toHaveAttribute('data-theme', 'dark');
    expect(document.documentElement.style.colorScheme).toBe('dark');
  });

  it('setMode light persiste en localStorage y aplica el tema claro', async () => {
    const user = userEvent.setup();
    renderProvider();

    await user.click(screen.getByRole('button', { name: 'Claro' }));

    expect(localStorage.getItem('oximeal.theme')).toBe('light');
    expect(document.documentElement).toHaveAttribute('data-theme', 'light');
    expect(document.documentElement.style.colorScheme).toBe('light');
    expect(screen.getByTestId('resolved')).toHaveTextContent('light');
  });

  it('el modo Sistema sigue prefers-color-scheme y reacciona a sus cambios', async () => {
    installMatchMedia(true);
    const user = userEvent.setup();
    renderProvider();

    await user.click(screen.getByRole('button', { name: 'Sistema' }));

    expect(screen.getByTestId('mode')).toHaveTextContent('system');
    expect(screen.getByTestId('resolved')).toHaveTextContent('dark');

    emitSystemChange(false);

    expect(screen.getByTestId('resolved')).toHaveTextContent('light');
    expect(document.documentElement).toHaveAttribute('data-theme', 'light');
  });

  it('al montar con localStorage en light resuelve light', () => {
    localStorage.setItem('oximeal.theme', 'light');

    renderProvider();

    expect(screen.getByTestId('resolved')).toHaveTextContent('light');
    expect(document.documentElement).toHaveAttribute('data-theme', 'light');
  });
});
