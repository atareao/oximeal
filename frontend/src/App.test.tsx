import { describe, expect, it, vi, beforeEach, afterEach } from 'vitest';
import { render, screen } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { MemoryRouter } from 'react-router-dom';
import { ApiError, fetchAuthMe } from './api/client';
import { useMealPlan } from './api/hooks';
import { SessionProvider } from './auth/SessionProvider';
import { App } from './App';

vi.mock('./api/client', async (importOriginal) => {
  const actual = await importOriginal<typeof import('./api/client')>();
  return {
    ...actual,
    fetchAuthMe: vi.fn(),
    logoutAuth: vi.fn(),
    setCsrfToken: vi.fn(),
    setUnauthorizedHandler: vi.fn(),
  };
});

vi.mock('./api/hooks', () => ({
  useMealPlan: vi.fn(),
  useRecipes: vi.fn(() => ({
    data: { items: [], total: 0, page: 1, page_size: 20 },
    isLoading: false,
    isError: false,
    refetch: vi.fn(),
  })),
  useRecipe: vi.fn(),
  useShoppingList: vi.fn(),
  useToggleShoppingItem: vi.fn(),
  useAddShoppingItem: vi.fn(),
  useConsolidateShopping: vi.fn(),
  useClearCompleted: vi.fn(),
  useAddMealPlanEntry: vi.fn(() => ({ mutate: vi.fn(), isPending: false, variables: null })),
  useRemoveMealPlanEntry: vi.fn(() => ({ mutate: vi.fn(), isPending: false, variables: null })),
  useUpdateMealPlanEntry: vi.fn(() => ({ mutate: vi.fn(), isPending: false, variables: null })),
  useAddPlanToShoppingList: vi.fn(() => ({ mutate: vi.fn(), isPending: false, variables: null })),
}));

const mockedFetchAuthMe = vi.mocked(fetchAuthMe);
const mockedUseMealPlan = vi.mocked(useMealPlan);

function installMatchMedia(matches = true) {
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

function mockMealPlan() {
  mockedUseMealPlan.mockReturnValue({
    data: [],
    isLoading: false,
    isError: false,
    error: null,
    refetch: vi.fn(),
  } as unknown as ReturnType<typeof useMealPlan>);
}

function renderApp() {
  const queryClient = new QueryClient({ defaultOptions: { queries: { retry: false } } });
  return render(
    <QueryClientProvider client={queryClient}>
      <MemoryRouter initialEntries={['/']}>
        <SessionProvider>
          <App />
        </SessionProvider>
      </MemoryRouter>
    </QueryClientProvider>,
  );
}

describe('App con sesión', () => {
  beforeEach(() => {
    mockedFetchAuthMe.mockReset();
    mockedUseMealPlan.mockReset();
    installMatchMedia(true);
    mockMealPlan();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('muestra carga y no renderiza datos hasta conocer la sesión', () => {
    mockedFetchAuthMe.mockReturnValue(new Promise(() => {}));
    renderApp();

    expect(screen.getByTestId('auth-loading')).toBeInTheDocument();
    expect(screen.queryByTestId('day-card')).not.toBeInTheDocument();
  });

  it('con OIDC desactivado (404 en /auth/me) renderiza la app con normalidad', async () => {
    mockedFetchAuthMe.mockRejectedValue(new ApiError(404, 'not_found', 'no existe'));
    renderApp();

    expect(await screen.findByText(/no hay comidas planificadas/i)).toBeInTheDocument();
    expect(screen.queryByTestId('auth-loading')).not.toBeInTheDocument();
  });
});
