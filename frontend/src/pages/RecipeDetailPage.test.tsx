import { describe, expect, it, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { MemoryRouter, Route, Routes } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { App as AntdApp } from 'antd';
import type { Recipe } from '../api/types';
import { ApiError, deleteRecipe, fetchRecipe, updateRecipe } from '../api/client';
import { useOnline } from '../offline/connectivity';
import { RecipeDetailPage } from './RecipeDetailPage';

vi.mock('../api/client', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../api/client')>();
  return {
    ...actual,
    fetchRecipe: vi.fn(),
    updateRecipe: vi.fn(),
    deleteRecipe: vi.fn(),
  };
});

vi.mock('../offline/connectivity', () => ({
  useOnline: vi.fn(() => true),
  isOnline: vi.fn(() => true),
}));

const mockedFetchRecipe = vi.mocked(fetchRecipe);
const mockedUpdateRecipe = vi.mocked(updateRecipe);
const mockedDeleteRecipe = vi.mocked(deleteRecipe);
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

describe('RecipeDetailPage', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    installMatchMedia();
    mockedUseOnline.mockReturnValue(true);
    mockedFetchRecipe.mockResolvedValue(recipe());
    mockedDeleteRecipe.mockResolvedValue(undefined);
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
});
