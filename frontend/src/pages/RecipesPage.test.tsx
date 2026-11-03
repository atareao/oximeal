import { describe, expect, it, vi, beforeEach } from 'vitest';
import { render, screen, waitFor, fireEvent } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { MemoryRouter, useLocation } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { App as AntdApp } from 'antd';
import recipesCss from './RecipesPage.css?raw';
import type {
  RecipeInput,
  RecipeListResponse,
  RecipeQueryParams,
  RecipeSummary,
} from '../api/types';
import { createRecipe, fetchRecipes, fetchTags, importRecipe } from '../api/client';
import { ApiError } from '../api/client';
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
    importRecipe: vi.fn(),
    fetchTags: vi.fn(),
  };
});

vi.mock('../offline/connectivity', () => ({
  useOnline: vi.fn(() => true),
  isOnline: vi.fn(() => true),
}));

const mockedFetchRecipes = vi.mocked(fetchRecipes);
const mockedCreateRecipe = vi.mocked(createRecipe);
const mockedImportRecipe = vi.mocked(importRecipe);
const mockedFetchTags = vi.mocked(fetchTags);
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
    tags: [],
    ...overrides,
  };
}

/** Estado mutable que simula la "base de datos" del backend. */
let recipes: RecipeSummary[] = [];

function page(params?: RecipeQueryParams): RecipeListResponse {
  const q = (params?.q ?? '').toLowerCase();
  const tag = params?.tag;
  const filtered = recipes.filter(
    (recipe) =>
      recipe.name.toLowerCase().includes(q) &&
      (!tag || recipe.tags.some((recipeTag) => recipeTag.slug === tag || recipeTag.id === tag)),
  );
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

/** Expone la query string actual para aseverar el filtro reflejado en la URL. */
function LocationProbe() {
  const location = useLocation();
  return <span data-testid="location-search">{location.search}</span>;
}

function recipesTree(initialEntries: string[] = ['/recipes']) {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false, gcTime: 0 },
      mutations: { retry: false },
    },
  });
  return (
    <QueryClientProvider client={queryClient}>
      <AntdApp>
        <MemoryRouter initialEntries={initialEntries}>
          <LocationProbe />
          <RecipesPage />
        </MemoryRouter>
      </AntdApp>
    </QueryClientProvider>
  );
}

function renderRecipes(initialEntries: string[] = ['/recipes']) {
  return render(recipesTree(initialEntries));
}

describe('RecipesPage', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    installMatchMedia();
    recipes = [];
    mockedUseOnline.mockReturnValue(true);
    mockedFetchTags.mockResolvedValue([]);
    mockedFetchRecipes.mockImplementation((params) => Promise.resolve(page(params)));
    mockedCreateRecipe.mockResolvedValue({
      id: 'new',
      name: 'Nueva',
      category: 'main',
      diet_type: 'mediterranean',
      prep_time_minutes: null,
      servings: 2,
      created_at: '2026-11-01T10:00:00Z',
      tags: [],
      ingredients: [],
      instructions: null,
      source_url: null,
    });
    mockedImportRecipe.mockResolvedValue({
      recipe: {
        id: 'imported',
        name: 'Importada',
        category: 'main',
        diet_type: 'mediterranean',
        prep_time_minutes: 30,
        servings: 2,
        created_at: '2026-11-01T10:00:00Z',
        tags: [],
        ingredients: [],
        instructions: null,
        source_url: 'https://example.com/r',
      },
      created: true,
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

  it('abre el diálogo de importación desde su botón', async () => {
    recipes = [summary({ id: 'r1', name: 'Tortilla' })];
    const user = userEvent.setup();
    renderRecipes();

    await screen.findByText('Tortilla');
    await user.click(screen.getByRole('button', { name: /importar desde url/i }));

    expect(await screen.findByLabelText(/url de la receta/i)).toBeInTheDocument();
  });

  it('importa una URL válida, refresca la lista y cierra el diálogo', async () => {
    recipes = [summary({ id: 'r1', name: 'Tortilla' })];
    const user = userEvent.setup();
    renderRecipes();

    await screen.findByText('Tortilla');
    const callsBefore = mockedFetchRecipes.mock.calls.length;

    await user.click(screen.getByRole('button', { name: /importar desde url/i }));
    await user.type(await screen.findByLabelText(/url de la receta/i), 'https://example.com/r');
    await user.click(screen.getByRole('button', { name: /^importar$/i }));

    await waitFor(() =>
      expect(mockedImportRecipe).toHaveBeenCalledWith({ url: 'https://example.com/r' }),
    );
    await waitFor(() => expect(mockedFetchRecipes.mock.calls.length).toBeGreaterThan(callsBefore));
    expect(await screen.findByText('Receta importada')).toBeInTheDocument();
  });

  it('avisa cuando la receta ya existía (200 idempotente)', async () => {
    mockedImportRecipe.mockResolvedValue({
      recipe: {
        id: 'r1',
        name: 'Tortilla',
        category: 'main',
        diet_type: 'mediterranean',
        prep_time_minutes: 20,
        servings: 2,
        created_at: '2026-11-01T10:00:00Z',
        tags: [],
        ingredients: [],
        instructions: null,
        source_url: 'https://example.com/r',
      },
      created: false,
    });
    recipes = [summary({ id: 'r1', name: 'Tortilla' })];
    const user = userEvent.setup();
    renderRecipes();

    await screen.findByText('Tortilla');
    await user.click(screen.getByRole('button', { name: /importar desde url/i }));
    await user.type(await screen.findByLabelText(/url de la receta/i), 'https://example.com/r');
    await user.click(screen.getByRole('button', { name: /^importar$/i }));

    expect(await screen.findByText('La receta ya existía')).toBeInTheDocument();
  });

  it('rechaza en cliente una URL que no es http(s) sin llamar al backend', async () => {
    recipes = [];
    const user = userEvent.setup();
    renderRecipes();

    await user.click(screen.getByRole('button', { name: /importar desde url/i }));
    await user.type(await screen.findByLabelText(/url de la receta/i), 'ftp://example.com/r');
    await user.click(screen.getByRole('button', { name: /^importar$/i }));

    expect(await screen.findByText(/introduce una url http/i)).toBeInTheDocument();
    expect(mockedImportRecipe).not.toHaveBeenCalled();
  });

  it('muestra un 400 (URL rechazada) dentro del diálogo sin cerrarlo', async () => {
    mockedImportRecipe.mockRejectedValue(new ApiError(400, 'bad_request', 'URL privada prohibida'));
    recipes = [];
    const user = userEvent.setup();
    renderRecipes();

    await user.click(screen.getByRole('button', { name: /importar desde url/i }));
    await user.type(await screen.findByLabelText(/url de la receta/i), 'https://example.com/r');
    await user.click(screen.getByRole('button', { name: /^importar$/i }));

    // Mensaje propio (no el detalle técnico del backend) y diálogo abierto.
    expect(await screen.findByText(/revisa la url/i)).toBeInTheDocument();
    expect(screen.queryByText(/URL privada prohibida/)).not.toBeInTheDocument();
    expect(screen.getByLabelText(/url de la receta/i)).toBeInTheDocument();
  });

  it('muestra un 422 con el aviso de página sin receta estructurada', async () => {
    mockedImportRecipe.mockRejectedValue(
      new ApiError(422, 'unprocessable_entity', 'json-ld missing'),
    );
    recipes = [];
    const user = userEvent.setup();
    renderRecipes();

    await user.click(screen.getByRole('button', { name: /importar desde url/i }));
    await user.type(await screen.findByLabelText(/url de la receta/i), 'https://example.com/r');
    await user.click(screen.getByRole('button', { name: /^importar$/i }));

    expect(
      await screen.findByText('La página no contiene una receta estructurada.'),
    ).toBeInTheDocument();
  });

  it('muestra un 502 con affordance de reintento', async () => {
    mockedImportRecipe.mockRejectedValueOnce(new ApiError(502, 'bad_gateway', 'origin timeout'));
    recipes = [];
    const user = userEvent.setup();
    renderRecipes();

    await user.click(screen.getByRole('button', { name: /importar desde url/i }));
    await user.type(await screen.findByLabelText(/url de la receta/i), 'https://example.com/r');
    await user.click(screen.getByRole('button', { name: /^importar$/i }));

    expect(
      await screen.findByText('No se pudo contactar con el origen. Inténtalo de nuevo.'),
    ).toBeInTheDocument();

    // El usuario puede reintentar volviendo a pulsar Importar.
    mockedImportRecipe.mockResolvedValueOnce({
      recipe: {
        id: 'r1',
        name: 'Tortilla',
        category: 'main',
        diet_type: 'mediterranean',
        prep_time_minutes: 20,
        servings: 2,
        created_at: '2026-11-01T10:00:00Z',
        tags: [],
        ingredients: [],
        instructions: null,
        source_url: 'https://example.com/r',
      },
      created: true,
    });
    await user.click(screen.getByRole('button', { name: /^importar$/i }));
    expect(await screen.findByText('Receta importada')).toBeInTheDocument();
  });

  it('importa al pulsar Enter en el campo de URL', async () => {
    recipes = [];
    const user = userEvent.setup();
    renderRecipes();

    await user.click(screen.getByRole('button', { name: /importar desde url/i }));
    await user.type(
      await screen.findByLabelText(/url de la receta/i),
      'https://example.com/r{enter}',
    );

    await waitFor(() =>
      expect(mockedImportRecipe).toHaveBeenCalledWith({ url: 'https://example.com/r' }),
    );
  });

  it('deshabilita el OK del diálogo si la conexión cae con el modal abierto', async () => {
    recipes = [];
    const user = userEvent.setup();
    const view = renderRecipes();

    await user.click(screen.getByRole('button', { name: /importar desde url/i }));
    expect(await screen.findByLabelText(/url de la receta/i)).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /^importar$/i })).toBeEnabled();

    mockedUseOnline.mockReturnValue(false);
    view.rerender(recipesTree());

    expect(screen.getByRole('button', { name: /^importar$/i })).toBeDisabled();
    expect(screen.getByTestId('recipes-import-offline')).toBeInTheDocument();
  });

  it('expone etiquetas y objetivos táctiles accesibles', async () => {
    recipes = [summary({ id: 'r1', name: 'Tortilla' })];
    renderRecipes();

    expect(await screen.findByLabelText(/buscar recetas/i)).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /nueva receta/i })).toHaveClass('recipes-page__new');
    expect(screen.getByRole('button', { name: /importar desde url/i })).toHaveClass(
      'recipes-page__import',
    );

    expect(recipesCss).toMatch(/\.recipes-page__new\s*\{[^}]*min-height:\s*48px/);
    expect(recipesCss).toMatch(/\.recipes-page__import\s*\{[^}]*min-height:\s*48px/);
    expect(recipesCss).toMatch(/\.recipes-page__import-confirm\s*\{[^}]*min-height:\s*48px/);
  });

  it('en modo offline deshabilita el alta y avisa', async () => {
    mockedUseOnline.mockReturnValue(false);
    recipes = [summary({ id: 'r1', name: 'Tortilla' })];
    renderRecipes();

    expect(await screen.findByTestId('recipes-offline')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /nueva receta/i })).toBeDisabled();
    expect(screen.getByRole('button', { name: /importar desde url/i })).toBeDisabled();
  });

  it('filtra por etiqueta al pulsar la etiqueta de una tarjeta y lo refleja en la URL', async () => {
    recipes = [
      summary({ id: 'r1', name: 'Tortilla', tags: [{ id: 't1', name: 'Vegano', slug: 'vegano' }] }),
      summary({ id: 'r2', name: 'César' }),
    ];
    mockedFetchTags.mockResolvedValue([
      { id: 't1', name: 'Vegano', slug: 'vegano', recipe_count: 1 },
    ]);
    const user = userEvent.setup();
    renderRecipes();

    await screen.findByText('Tortilla');
    await user.click(screen.getByRole('button', { name: /filtrar por etiqueta vegano/i }));

    await waitFor(() =>
      expect(mockedFetchRecipes).toHaveBeenLastCalledWith(
        expect.objectContaining({ tag: 'vegano' }),
        expect.anything(),
      ),
    );
    await waitFor(() => expect(screen.queryByText('César')).not.toBeInTheDocument());
    expect(screen.getByText('Tortilla')).toBeInTheDocument();
    expect(screen.getByTestId('location-search').textContent).toContain('tag=vegano');
  });

  it('reconstruye el filtro de etiqueta desde la URL', async () => {
    recipes = [
      summary({ id: 'r1', name: 'Tortilla', tags: [{ id: 't1', name: 'Vegano', slug: 'vegano' }] }),
      summary({ id: 'r2', name: 'César' }),
    ];
    mockedFetchTags.mockResolvedValue([
      { id: 't1', name: 'Vegano', slug: 'vegano', recipe_count: 1 },
    ]);
    renderRecipes(['/recipes?tag=vegano']);

    await waitFor(() =>
      expect(mockedFetchRecipes).toHaveBeenCalledWith(
        expect.objectContaining({ tag: 'vegano' }),
        expect.anything(),
      ),
    );
    expect(await screen.findByText('Tortilla')).toBeInTheDocument();
    await waitFor(() => expect(screen.queryByText('César')).not.toBeInTheDocument());
  });

  it('combina el filtro de etiqueta con la búsqueda y permite quitarlo', async () => {
    recipes = [
      summary({ id: 'r1', name: 'Tortilla', tags: [{ id: 't1', name: 'Vegano', slug: 'vegano' }] }),
      summary({
        id: 'r2',
        name: 'Tortilla de patata',
        tags: [{ id: 't1', name: 'Vegano', slug: 'vegano' }],
      }),
      summary({ id: 'r3', name: 'César' }),
    ];
    mockedFetchTags.mockResolvedValue([
      { id: 't1', name: 'Vegano', slug: 'vegano', recipe_count: 2 },
    ]);
    const user = userEvent.setup();
    renderRecipes(['/recipes?tag=vegano']);

    await screen.findByText('Tortilla');
    await user.type(screen.getByLabelText(/buscar recetas/i), 'patata');

    await waitFor(() =>
      expect(mockedFetchRecipes).toHaveBeenLastCalledWith(
        expect.objectContaining({ tag: 'vegano', q: 'patata' }),
        expect.anything(),
      ),
    );

    // Quitar el filtro con el botón de limpiar del control.
    const clear = document.querySelector('.ant-select-clear');
    expect(clear).not.toBeNull();
    fireEvent.click(clear as HTMLElement);

    await waitFor(() => {
      const lastParams = mockedFetchRecipes.mock.calls.at(-1)?.[0];
      expect(lastParams?.tag).toBeUndefined();
    });
    await waitFor(() =>
      expect(screen.getByTestId('location-search').textContent).not.toContain('tag='),
    );
  });

  it('mantiene el filtro de etiqueta al paginar', async () => {
    const veganoTag = { id: 't1', name: 'Vegano', slug: 'vegano' };
    recipes = Array.from({ length: 25 }, (_, index) =>
      summary({ id: `r${index + 1}`, name: `Receta ${index + 1}`, tags: [veganoTag] }),
    );
    mockedFetchTags.mockResolvedValue([{ ...veganoTag, recipe_count: 25 }]);
    const user = userEvent.setup();
    renderRecipes(['/recipes?tag=vegano']);

    await screen.findByText('Receta 1');
    await user.click(screen.getByRole('button', { name: /siguiente/i }));

    await waitFor(() =>
      expect(mockedFetchRecipes).toHaveBeenLastCalledWith(
        expect.objectContaining({ tag: 'vegano', page: 2 }),
        expect.anything(),
      ),
    );
    expect(await screen.findByText('Receta 21')).toBeInTheDocument();
  });

  it('en modo offline deshabilita el filtro por etiqueta y mantiene el aviso', async () => {
    mockedUseOnline.mockReturnValue(false);
    recipes = [summary({ id: 'r1', name: 'Tortilla' })];
    mockedFetchTags.mockResolvedValue([
      { id: 't1', name: 'Vegano', slug: 'vegano', recipe_count: 1 },
    ]);
    renderRecipes();

    expect(await screen.findByTestId('recipes-offline')).toBeInTheDocument();
    expect(screen.getByRole('combobox', { name: /filtrar por etiqueta/i })).toBeDisabled();
  });
});
