import { describe, expect, it, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { MemoryRouter } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { App as AntdApp } from 'antd';
import recipesCss from './RecipesPage.css?raw';
import type {
  RecipeInput,
  RecipeListResponse,
  RecipeQueryParams,
  RecipeSummary,
} from '../api/types';
import { createRecipe, fetchRecipes } from '../api/client';
import { useOnline } from '../offline/connectivity';
import { RecipesPage } from './RecipesPage';

// Se mockea el cliente HTTP, no los hooks: así se ejerce la integración real
// (React Query + invalidación de caché) de la página.
vi.mock('../api/client', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../api/client')>();
  return {
    ...actual,
    fetchRecipes: vi.fn(),
    fetchRecipe: vi.fn(),
    createRecipe: vi.fn(),
    updateRecipe: vi.fn(),
    deleteRecipe: vi.fn(),
  };
});

vi.mock('../offline/connectivity', () => ({
  useOnline: vi.fn(() => true),
  isOnline: vi.fn(() => true),
}));

const mockedFetchRecipes = vi.mocked(fetchRecipes);
const mockedCreateRecipe = vi.mocked(createRecipe);
const mockedUseOnline = vi.mocked(useOnline);

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

function summary(
  overrides: Partial<RecipeSummary> & Pick<RecipeSummary, 'id' | 'name'>,
): RecipeSummary {
  return {
    category: 'main',
    diet_type: 'mediterranean',
    prep_time_minutes: 20,
    servings: 2,
    created_at: '2026-11-01T10:00:00Z',
    ...overrides,
  };
}

/** Estado mutable que simula la "base de datos" del backend. */
let recipes: RecipeSummary[] = [];

function page(params?: RecipeQueryParams): RecipeListResponse {
  const q = (params?.q ?? '').toLowerCase();
  const filtered = recipes.filter((recipe) => recipe.name.toLowerCase().includes(q));
  const pageNumber = params?.page ?? 1;
  const pageSize = params?.page_size ?? 20;
  const start = (pageNumber - 1) * pageSize;
  return {
    items: filtered.slice(start, start + pageSize),
    total: filtered.length,
    page: pageNumber,
    page_size: pageSize,
  };
}

function renderRecipes() {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false, gcTime: 0 },
      mutations: { retry: false },
    },
  });
  return render(
    <QueryClientProvider client={queryClient}>
      <AntdApp>
        <MemoryRouter>
          <RecipesPage />
        </MemoryRouter>
      </AntdApp>
    </QueryClientProvider>,
  );
}

describe('RecipesPage', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    installMatchMedia();
    recipes = [];
    mockedUseOnline.mockReturnValue(true);
    mockedFetchRecipes.mockImplementation((params) => Promise.resolve(page(params)));
    mockedCreateRecipe.mockResolvedValue({
      id: 'new',
      name: 'Nueva',
      category: 'main',
      diet_type: 'mediterranean',
      prep_time_minutes: null,
      servings: 2,
      created_at: '2026-11-01T10:00:00Z',
      ingredients: [],
      instructions: null,
      source_url: null,
    });
  });

  it('lista las recetas con su nombre y categoría', async () => {
    recipes = [
      summary({ id: 'r1', name: 'Tortilla', category: 'main' }),
      summary({ id: 'r2', name: 'César', category: 'salad' }),
    ];
    renderRecipes();

    expect(await screen.findByText('Tortilla')).toBeInTheDocument();
    expect(screen.getByText('Principal')).toBeInTheDocument();
    expect(screen.getByText('Ensalada')).toBeInTheDocument();
    expect(screen.getByRole('link', { name: /tortilla/i })).toHaveAttribute('href', '/recipes/r1');
  });

  it('busca por texto y filtra el listado', async () => {
    recipes = [summary({ id: 'r1', name: 'Lentejas' }), summary({ id: 'r2', name: 'Tortilla' })];
    const user = userEvent.setup();
    renderRecipes();

    await screen.findByText('Lentejas');
    await user.type(screen.getByLabelText(/buscar recetas/i), 'len');

    await waitFor(() =>
      expect(mockedFetchRecipes).toHaveBeenLastCalledWith(
        expect.objectContaining({ q: 'len' }),
        expect.anything(),
      ),
    );
    await waitFor(() => expect(screen.queryByText('Tortilla')).not.toBeInTheDocument());
    expect(screen.getByText('Lentejas')).toBeInTheDocument();
  });

  it('pagina el listado', async () => {
    recipes = Array.from({ length: 25 }, (_, index) =>
      summary({ id: `r${index + 1}`, name: `Receta ${index + 1}` }),
    );
    const user = userEvent.setup();
    renderRecipes();

    await screen.findByText('Receta 1');
    expect(screen.getByRole('button', { name: /anterior/i })).toBeDisabled();

    await user.click(screen.getByRole('button', { name: /siguiente/i }));

    await waitFor(() =>
      expect(mockedFetchRecipes).toHaveBeenLastCalledWith(
        expect.objectContaining({ page: 2 }),
        expect.anything(),
      ),
    );
    expect(await screen.findByText('Receta 21')).toBeInTheDocument();
  });

  it('muestra el estado vacío sin lanzar error', async () => {
    recipes = [];
    renderRecipes();

    expect(await screen.findByText(/no hay recetas/i)).toBeInTheDocument();
  });

  it('muestra el error de red con opción de reintentar', async () => {
    mockedFetchRecipes.mockRejectedValueOnce(new Error('boom'));
    recipes = [summary({ id: 'r1', name: 'Tortilla' })];
    const user = userEvent.setup();
    renderRecipes();

    expect(await screen.findByText(/no se pudieron cargar las recetas/i)).toBeInTheDocument();
    await user.click(screen.getByRole('button', { name: /reintentar/i }));

    expect(await screen.findByText('Tortilla')).toBeInTheDocument();
  });

  it('crea una receta desde el formulario y refresca la lista', async () => {
    recipes = [summary({ id: 'r1', name: 'Tortilla' })];
    const user = userEvent.setup();
    renderRecipes();

    await screen.findByText('Tortilla');
    await user.click(screen.getByRole('button', { name: /nueva receta/i }));

    await user.type(await screen.findByLabelText('Nombre'), 'Paella');
    await user.selectOptions(screen.getByLabelText('Categoría'), 'main');
    await user.type(screen.getByLabelText('Nombre del ingrediente 1'), 'Arroz');
    await user.type(screen.getByLabelText('Cantidad del ingrediente 1'), '200');
    await user.type(screen.getByLabelText('Unidad del ingrediente 1'), 'g');
    await user.click(screen.getByRole('button', { name: /guardar/i }));

    await waitFor(() => expect(mockedCreateRecipe).toHaveBeenCalledTimes(1));
    expect(mockedCreateRecipe).toHaveBeenCalledWith(
      expect.objectContaining<Partial<RecipeInput>>({
        name: 'Paella',
        category: 'main',
        ingredients: [expect.objectContaining({ name: 'Arroz', amount: 200, unit: 'g' })],
      }),
    );
  });

  it('expone etiquetas y objetivos táctiles accesibles', async () => {
    recipes = [summary({ id: 'r1', name: 'Tortilla' })];
    renderRecipes();

    expect(await screen.findByLabelText(/buscar recetas/i)).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /nueva receta/i })).toHaveClass('recipes-page__new');

    expect(recipesCss).toMatch(/\.recipes-page__new\s*\{[^}]*min-height:\s*48px/);
  });

  it('en modo offline deshabilita el alta y avisa', async () => {
    mockedUseOnline.mockReturnValue(false);
    recipes = [summary({ id: 'r1', name: 'Tortilla' })];
    renderRecipes();

    expect(await screen.findByTestId('recipes-offline')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /nueva receta/i })).toBeDisabled();
  });
});
