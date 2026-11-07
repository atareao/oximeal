import { describe, expect, it, vi, beforeEach } from 'vitest';
import { render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Link, MemoryRouter, Route, Routes } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { App as AntdApp } from 'antd';
import type { Recipe } from '../api/types';
import {
  ApiError,
  addRecipeToShoppingList,
  deleteRecipe,
  fetchRecipe,
  getSettings,
  fetchTags,
  updateRecipe,
} from '../api/client';
import { useOnline } from '../offline/connectivity';
import recipeDetailCss from './RecipeDetailPage.css?raw';
import { RecipeDetailPage } from './RecipeDetailPage';

vi.mock('../api/client', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../api/client')>();
  return {
    ...actual,
    fetchRecipe: vi.fn(),
    updateRecipe: vi.fn(),
    deleteRecipe: vi.fn(),
    fetchTags: vi.fn(),
    getSettings: vi.fn(),
    addRecipeToShoppingList: vi.fn(),
  };
});

vi.mock('../offline/connectivity', () => ({
  useOnline: vi.fn(() => true),
  isOnline: vi.fn(() => true),
}));

const mockedFetchRecipe = vi.mocked(fetchRecipe);
const mockedUpdateRecipe = vi.mocked(updateRecipe);
const mockedDeleteRecipe = vi.mocked(deleteRecipe);
const mockedFetchTags = vi.mocked(fetchTags);
const mockedFetchSettings = vi.mocked(getSettings);
const mockedAddFromRecipe = vi.mocked(addRecipeToShoppingList);
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

function recipe(overrides: Partial<Recipe> = {}): Recipe {
  return {
    id: 'r1',
    name: 'Tortilla',
    category: 'main',
    diet_type: 'mediterranean',
    prep_time_minutes: 25,
    servings: 4,
    created_at: '2026-11-01T10:00:00Z',
    tags: [],
    ingredients: [{ name: 'Patata', amount: 500, unit: 'g' }],
    instructions: 'Pelar y freír.',
    source_url: null,
    ...overrides,
  };
}

function renderDetail(id = 'r1') {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false, gcTime: 0 },
      mutations: { retry: false },
    },
  });
  return render(
    <QueryClientProvider client={queryClient}>
      <AntdApp>
        <MemoryRouter initialEntries={[`/recipes/${id}`]}>
          <Routes>
            <Route path="/recipes/:id" element={<RecipeDetailPage />} />
            <Route path="/recipes" element={<div data-testid="recipes-list">lista</div>} />
          </Routes>
        </MemoryRouter>
      </AntdApp>
    </QueryClientProvider>,
  );
}

function renderWithNavigation() {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false, gcTime: 0 },
      mutations: { retry: false },
    },
  });
  return render(
    <QueryClientProvider client={queryClient}>
      <AntdApp>
        <MemoryRouter initialEntries={['/recipes/r1']}>
          <Routes>
            <Route
              path="/recipes/:id"
              element={
                <>
                  <Link to="/recipes/r2">ir a r2</Link>
                  <RecipeDetailPage />
                </>
              }
            />
          </Routes>
        </MemoryRouter>
      </AntdApp>
    </QueryClientProvider>,
  );
}

describe('RecipeDetailPage', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    installMatchMedia();
    mockedUseOnline.mockReturnValue(true);
    mockedFetchRecipe.mockResolvedValue(recipe());
    mockedDeleteRecipe.mockResolvedValue(undefined);
    mockedFetchTags.mockResolvedValue([]);
    mockedFetchSettings.mockResolvedValue({ default_servings: 4 });
    mockedAddFromRecipe.mockResolvedValue([]);
  });

  it('muestra el detalle completo con ingredientes e instrucciones', async () => {
    renderDetail();

    expect(await screen.findByRole('heading', { name: 'Tortilla' })).toBeInTheDocument();
    expect(screen.getByText('Patata')).toBeInTheDocument();
    expect(screen.getByText(/500\s*g/)).toBeInTheDocument();
    expect(screen.getByText('Pelar y freír.')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /editar/i })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /borrar receta/i })).toBeInTheDocument();
  });

  it('muestra las etiquetas de la receta', async () => {
    mockedFetchRecipe.mockResolvedValue(
      recipe({ tags: [{ id: 't1', name: 'Vegano', slug: 'vegano' }] }),
    );
    renderDetail();

    await screen.findByRole('heading', { name: 'Tortilla' });
    expect(screen.getByText('Vegano')).toBeInTheDocument();
    expect(screen.getByRole('list', { name: /etiquetas/i })).toBeInTheDocument();
  });

  it('sin etiquetas no muestra ninguna etiqueta', async () => {
    mockedFetchRecipe.mockResolvedValue(recipe({ tags: [] }));
    renderDetail();

    await screen.findByRole('heading', { name: 'Tortilla' });
    expect(screen.queryByRole('list', { name: /etiquetas/i })).not.toBeInTheDocument();
  });

  it('muestra el estado "no encontrada" para un id inexistente', async () => {
    mockedFetchRecipe.mockRejectedValue(new ApiError(404, 'not_found', 'no encontrada'));
    renderDetail('desconocida');

    expect(await screen.findByText(/receta no encontrada/i)).toBeInTheDocument();
    expect(screen.getByRole('link', { name: /volver a recetas/i })).toHaveAttribute(
      'href',
      '/recipes',
    );
  });

  it('edita una receta con el formulario precargado', async () => {
    mockedUpdateRecipe.mockResolvedValue(recipe({ name: 'Tortilla de patata' }));
    const user = userEvent.setup();
    renderDetail();

    await user.click(await screen.findByRole('button', { name: /editar/i }));
    const nameInput = await screen.findByLabelText('Nombre');
    expect(nameInput).toHaveValue('Tortilla');
    await user.clear(nameInput);
    await user.type(nameInput, 'Tortilla de patata');
    await user.click(screen.getByRole('button', { name: /guardar/i }));

    await waitFor(() => expect(mockedUpdateRecipe).toHaveBeenCalledTimes(1));
    expect(mockedUpdateRecipe).toHaveBeenCalledWith(
      'r1',
      expect.objectContaining({ name: 'Tortilla de patata' }),
    );
  });

  it('pide confirmación antes de borrar y actualiza la vista', async () => {
    const user = userEvent.setup();
    renderDetail();

    await screen.findByRole('heading', { name: 'Tortilla' });
    await user.click(screen.getByRole('button', { name: /borrar receta/i }));

    expect(mockedDeleteRecipe).not.toHaveBeenCalled();
    await user.click(await screen.findByRole('button', { name: 'Borrar' }));

    await waitFor(() => expect(mockedDeleteRecipe).toHaveBeenCalledWith('r1'));
    expect(await screen.findByTestId('recipes-list')).toBeInTheDocument();
  });

  it('no borra si el usuario cancela', async () => {
    const user = userEvent.setup();
    renderDetail();

    await screen.findByRole('heading', { name: 'Tortilla' });
    await user.click(screen.getByRole('button', { name: /borrar receta/i }));
    await user.click(await screen.findByRole('button', { name: 'Cancelar' }));

    expect(mockedDeleteRecipe).not.toHaveBeenCalled();
  });

  it('en modo offline deshabilita editar y borrar y avisa', async () => {
    mockedUseOnline.mockReturnValue(false);
    renderDetail();

    await screen.findByRole('heading', { name: 'Tortilla' });
    expect(screen.getByTestId('recipe-detail-offline')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /editar/i })).toBeDisabled();
    expect(screen.getByRole('button', { name: /borrar receta/i })).toBeDisabled();
  });

  it('añade la receta a la lista con las raciones por defecto y ofrece enlace a la compra', async () => {
    const user = userEvent.setup();
    renderDetail();

    await screen.findByRole('heading', { name: 'Tortilla' });
    const servings = screen.getByRole('spinbutton', { name: /raciones/i });
    expect(servings).toHaveValue('4');

    await user.click(screen.getByRole('button', { name: /añadir a la lista/i }));

    await waitFor(() =>
      expect(mockedAddFromRecipe).toHaveBeenCalledWith({ recipe_id: 'r1', servings: 4 }),
    );
    const link = await screen.findByRole('link', { name: /ver la lista/i });
    expect(link).toHaveAttribute('href', '/shopping');
  });

  it('permite cambiar las raciones y las envía', async () => {
    const user = userEvent.setup();
    renderDetail();

    const servings = await screen.findByRole('spinbutton', { name: /raciones/i });
    await user.clear(servings);
    await user.type(servings, '6');
    await user.click(screen.getByRole('button', { name: /añadir a la lista/i }));

    await waitFor(() =>
      expect(mockedAddFromRecipe).toHaveBeenCalledWith({ recipe_id: 'r1', servings: 6 }),
    );
  });

  it('muestra el error del servidor y mantiene la vista del detalle', async () => {
    mockedAddFromRecipe.mockRejectedValue(new ApiError(404, 'not_found', 'La receta no existe'));
    const user = userEvent.setup();
    renderDetail();

    await screen.findByRole('heading', { name: 'Tortilla' });
    await user.click(screen.getByRole('button', { name: /añadir a la lista/i }));

    expect(await screen.findByText('La receta no existe')).toBeInTheDocument();
    expect(screen.getByRole('heading', { name: 'Tortilla' })).toBeInTheDocument();
    expect(screen.getByText('Patata')).toBeInTheDocument();
    expect(screen.queryByRole('link', { name: /ver la lista/i })).not.toBeInTheDocument();
  });

  it('muestra el mensaje si el servidor falla con 500', async () => {
    mockedAddFromRecipe.mockRejectedValue(new ApiError(500, 'internal', 'Error interno'));
    const user = userEvent.setup();
    renderDetail();

    await screen.findByRole('heading', { name: 'Tortilla' });
    await user.click(screen.getByRole('button', { name: /añadir a la lista/i }));

    expect(await screen.findByText('Error interno')).toBeInTheDocument();
    expect(screen.getByRole('heading', { name: 'Tortilla' })).toBeInTheDocument();
    expect(screen.getByText('Patata')).toBeInTheDocument();
    expect(screen.queryByRole('link', { name: /ver la lista/i })).not.toBeInTheDocument();
  });

  it('sin conexión deshabilita la acción y el selector y no envía nada', async () => {
    mockedUseOnline.mockReturnValue(false);
    const user = userEvent.setup();
    renderDetail();

    await screen.findByRole('heading', { name: 'Tortilla' });
    const addButton = screen.getByRole('button', { name: /añadir a la lista/i });
    const servings = screen.getByRole('spinbutton', { name: /raciones/i });

    expect(addButton).toBeDisabled();
    expect(servings).toBeDisabled();

    await user.click(addButton);
    expect(mockedAddFromRecipe).not.toHaveBeenCalled();
  });

  it('la acción y el selector cumplen a11y y objetivo táctil de 48 px', async () => {
    const { container } = renderDetail();

    await screen.findByRole('heading', { name: 'Tortilla' });
    const addButton = screen.getByRole('button', { name: /añadir a la lista/i });
    const servings = screen.getByRole('spinbutton', { name: /raciones/i });

    expect(addButton).toHaveClass('recipe-detail__add-to-list');
    expect(servings).toHaveAttribute('aria-valuemin', '1');
    expect(container.querySelector('.recipe-detail__servings')).not.toBeNull();

    expect(recipeDetailCss).toMatch(/\.recipe-detail__add-to-list\s*\{[^}]*min-height:\s*48px/);
    expect(recipeDetailCss).toMatch(/\.recipe-detail__servings\s*\{[^}]*min-height:\s*48px/);
  });

  it('distingue visualmente la etiqueta manual de la automática', async () => {
    mockedFetchRecipe.mockResolvedValue(
      recipe({
        tags: [
          { id: 't1', name: 'Vegano', slug: 'vegano', source: 'manual' },
          { id: 't2', name: 'Sin gluten', slug: 'sin-gluten', source: 'auto' },
        ],
      }),
    );
    renderDetail();

    await screen.findByRole('heading', { name: 'Tortilla' });
    const list = screen.getByRole('list', { name: /etiquetas/i });
    const manual = within(list).getByText('Vegano');
    const auto = within(list).getByText('Sin gluten');

    expect(manual).toHaveClass('recipe-detail__tag--manual');
    expect(auto).toHaveClass('recipe-detail__tag--auto');
    expect(manual.className).not.toEqual(auto.className);
  });

  it('usa la preferencia de comensales y escala 8→2 por defecto', async () => {
    mockedFetchRecipe.mockResolvedValue(
      recipe({ servings: 8, ingredients: [{ name: 'Patata', amount: 800, unit: 'g' }] }),
    );
    mockedFetchSettings.mockResolvedValue({ default_servings: 2 });
    renderDetail();

    await screen.findByRole('heading', { name: 'Tortilla' });
    const servings = screen.getByRole('spinbutton', { name: /raciones/i });
    expect(servings).toHaveValue('2');
    expect(screen.getByText(/receta para 8 · cantidades para 2/i)).toBeInTheDocument();
    expect(screen.getByText(/200\s*g/)).toBeInTheDocument();
  });

  it('recalcula las cantidades al cambiar a 6 raciones', async () => {
    mockedFetchRecipe.mockResolvedValue(
      recipe({ servings: 8, ingredients: [{ name: 'Patata', amount: 800, unit: 'g' }] }),
    );
    mockedFetchSettings.mockResolvedValue({ default_servings: 2 });
    const user = userEvent.setup();
    renderDetail();

    await screen.findByRole('heading', { name: 'Tortilla' });
    const servings = screen.getByRole('spinbutton', { name: /raciones/i });
    await user.clear(servings);
    await user.type(servings, '6');

    expect(screen.getByText(/600\s*g/)).toBeInTheDocument();
    expect(screen.getByText(/receta para 8 · cantidades para 6/i)).toBeInTheDocument();
  });

  it('«Ver originales» muestra las cantidades sin factor', async () => {
    mockedFetchRecipe.mockResolvedValue(
      recipe({ servings: 8, ingredients: [{ name: 'Patata', amount: 800, unit: 'g' }] }),
    );
    mockedFetchSettings.mockResolvedValue({ default_servings: 2 });
    const user = userEvent.setup();
    renderDetail();

    await screen.findByRole('heading', { name: 'Tortilla' });
    expect(screen.getByText(/200\s*g/)).toBeInTheDocument();

    await user.click(screen.getByRole('checkbox', { name: /ver originales/i }));

    expect(screen.getByText(/800\s*g/)).toBeInTheDocument();
  });

  it('una receta importada no escala, muestra originales y avisa', async () => {
    mockedFetchRecipe.mockResolvedValue(
      recipe({
        servings: 8,
        ingredients: [
          { name: 'Sal', amount: 1, unit: 'ud' },
          { name: 'Pimienta', amount: 1, unit: 'ud' },
        ],
      }),
    );
    mockedFetchSettings.mockResolvedValue({ default_servings: 2 });
    renderDetail();

    await screen.findByRole('heading', { name: 'Tortilla' });
    expect(
      screen.getByText(/la importación no trae cantidades; no se escalan/i),
    ).toBeInTheDocument();
    expect(screen.getAllByText(/1\s*ud/)).toHaveLength(2);

    // El toggle de originales queda deshabilitado: siempre se muestran las
    // originales. El selector sigue visible pero no tiene efecto.
    expect(screen.getByRole('checkbox', { name: /ver originales/i })).toBeDisabled();
    expect(screen.getByRole('spinbutton', { name: /raciones/i })).toBeInTheDocument();
  });

  it('añadir a la compra envía las raciones seleccionadas', async () => {
    mockedFetchRecipe.mockResolvedValue(
      recipe({ servings: 8, ingredients: [{ name: 'Patata', amount: 800, unit: 'g' }] }),
    );
    mockedFetchSettings.mockResolvedValue({ default_servings: 2 });
    const user = userEvent.setup();
    renderDetail();

    await screen.findByRole('heading', { name: 'Tortilla' });
    const servings = screen.getByRole('spinbutton', { name: /raciones/i });
    await user.clear(servings);
    await user.type(servings, '6');
    await user.click(screen.getByRole('button', { name: /añadir a la lista/i }));

    await waitFor(() =>
      expect(mockedAddFromRecipe).toHaveBeenCalledWith({ recipe_id: 'r1', servings: 6 }),
    );
  });

  it('usa la preferencia del servidor (4) y no el fallback', async () => {
    mockedFetchRecipe.mockResolvedValue(
      recipe({ servings: 8, ingredients: [{ name: 'Patata', amount: 800, unit: 'g' }] }),
    );
    mockedFetchSettings.mockResolvedValue({ default_servings: 4 });
    renderDetail();

    await screen.findByRole('heading', { name: 'Tortilla' });
    const servings = screen.getByRole('spinbutton', { name: /raciones/i });
    expect(servings).toHaveValue('4');
    expect(screen.getByText(/receta para 8 · cantidades para 4/i)).toBeInTheDocument();
    expect(screen.getByText(/400\s*g/)).toBeInTheDocument();
  });

  it('una preferencia que llega tarde no pisa una elección manual', async () => {
    mockedFetchRecipe.mockResolvedValue(
      recipe({ servings: 8, ingredients: [{ name: 'Patata', amount: 800, unit: 'g' }] }),
    );
    let resolveSettings: (value: { default_servings: number }) => void = () => {};
    mockedFetchSettings.mockReturnValue(
      new Promise((resolve) => {
        resolveSettings = resolve;
      }),
    );
    const user = userEvent.setup();
    renderDetail();

    await screen.findByRole('heading', { name: 'Tortilla' });
    const servings = screen.getByRole('spinbutton', { name: /raciones/i });
    // Antes de resolver la preferencia se usa el fallback 2.
    expect(servings).toHaveValue('2');

    await user.clear(servings);
    await user.type(servings, '6');

    resolveSettings({ default_servings: 4 });

    await waitFor(() => expect(servings).toHaveValue('6'));
  });

  it('clampea una preferencia 0 a 1 al calcular y al añadir a la compra', async () => {
    mockedFetchRecipe.mockResolvedValue(
      recipe({ servings: 8, ingredients: [{ name: 'Patata', amount: 800, unit: 'g' }] }),
    );
    mockedFetchSettings.mockResolvedValue({ default_servings: 0 });
    const user = userEvent.setup();
    renderDetail();

    await screen.findByRole('heading', { name: 'Tortilla' });
    // 0 se acota a 1: no se muestra todo a 0.
    expect(screen.getByText(/100\s*g/)).toBeInTheDocument();
    expect(screen.queryByText(/^0\s*g$/)).not.toBeInTheDocument();

    await user.click(screen.getByRole('button', { name: /añadir a la lista/i }));
    await waitFor(() =>
      expect(mockedAddFromRecipe).toHaveBeenCalledWith({ recipe_id: 'r1', servings: 1 }),
    );
  });

  it('clampea una preferencia 21 a 20 al calcular y al añadir a la compra', async () => {
    mockedFetchRecipe.mockResolvedValue(
      recipe({ servings: 8, ingredients: [{ name: 'Patata', amount: 800, unit: 'g' }] }),
    );
    mockedFetchSettings.mockResolvedValue({ default_servings: 21 });
    const user = userEvent.setup();
    renderDetail();

    await screen.findByRole('heading', { name: 'Tortilla' });
    expect(screen.getByText(/2000\s*g/)).toBeInTheDocument();

    await user.click(screen.getByRole('button', { name: /añadir a la lista/i }));
    await waitFor(() =>
      expect(mockedAddFromRecipe).toHaveBeenCalledWith({ recipe_id: 'r1', servings: 20 }),
    );
  });

  it('al navegar a otra receta resetea raciones y toggle de originales', async () => {
    mockedFetchRecipe.mockImplementation((id: string) =>
      Promise.resolve(
        recipe({
          id,
          name: id === 'r1' ? 'Tortilla' : 'Paella',
          servings: 8,
          ingredients: [{ name: 'Patata', amount: 800, unit: 'g' }],
        }),
      ),
    );
    mockedFetchSettings.mockResolvedValue({ default_servings: 4 });
    const user = userEvent.setup();
    renderWithNavigation();

    await screen.findByRole('heading', { name: 'Tortilla' });
    const servings = screen.getByRole('spinbutton', { name: /raciones/i });
    await user.clear(servings);
    await user.type(servings, '6');
    await user.click(screen.getByRole('checkbox', { name: /ver originales/i }));
    expect(screen.getByRole('checkbox', { name: /ver originales/i })).toBeChecked();

    await user.click(screen.getByRole('link', { name: /ir a r2/i }));

    await screen.findByRole('heading', { name: 'Paella' });
    expect(screen.getByRole('spinbutton', { name: /raciones/i })).toHaveValue('4');
    expect(screen.getByRole('checkbox', { name: /ver originales/i })).not.toBeChecked();
  });
});
