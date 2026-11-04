import { describe, expect, it, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { App as AntdApp } from 'antd';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import recipeFormCss from './RecipeForm.css?raw';
import type { Recipe, RecipeInput, TagWithCount } from '../api/types';
import { fetchTags } from '../api/client';
import { RecipeForm } from './RecipeForm';

vi.mock('../api/client', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../api/client')>();
  return {
    ...actual,
    fetchTags: vi.fn(),
  };
});

const mockedFetchTags = vi.mocked(fetchTags);

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

const TAG_VEGANO: TagWithCount = { id: 't1', name: 'Vegano', slug: 'vegano', recipe_count: 2 };
const TAG_SIN_GLUTEN: TagWithCount = {
  id: 't2',
  name: 'Sin gluten',
  slug: 'sin-gluten',
  recipe_count: 0,
};

function recipe(overrides: Partial<Recipe> = {}): Recipe {
  return {
    id: 'r1',
    name: 'Tortilla',
    category: 'main',
    diet_type: 'mediterranean',
    prep_time_minutes: 25,
    servings: 4,
    created_at: '2026-11-01T10:00:00Z',
    tags: [{ id: 't1', name: 'Vegano', slug: 'vegano' }],
    ingredients: [{ name: 'Patata', amount: 500, unit: 'g' }],
    instructions: null,
    source_url: null,
    ...overrides,
  };
}

function renderForm(overrides: Partial<Parameters<typeof RecipeForm>[0]> = {}) {
  const onSubmit = vi.fn();
  const onClose = vi.fn();
  const queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: 0 }, mutations: { retry: false } },
  });
  render(
    <QueryClientProvider client={queryClient}>
      <AntdApp>
        <RecipeForm open initial={null} onClose={onClose} onSubmit={onSubmit} {...overrides} />
      </AntdApp>
    </QueryClientProvider>,
  );
  return { onSubmit, onClose };
}

async function fillRequiredFields(user: ReturnType<typeof userEvent.setup>) {
  await user.type(screen.getByLabelText('Nombre'), 'Tortilla');
  await user.type(screen.getByLabelText('Nombre del ingrediente 1'), 'Patata');
  await user.type(screen.getByLabelText('Cantidad del ingrediente 1'), '2');
  await user.type(screen.getByLabelText('Unidad del ingrediente 1'), 'ud');
}

describe('RecipeForm', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    installMatchMedia();
    mockedFetchTags.mockResolvedValue([TAG_VEGANO, TAG_SIN_GLUTEN]);
  });

  it('no envía si el nombre está vacío y muestra el error de validación', async () => {
    const user = userEvent.setup();
    const { onSubmit } = renderForm();

    await user.click(screen.getByRole('button', { name: /guardar/i }));

    expect(await screen.findByText('Escribe el nombre de la receta')).toBeInTheDocument();
    expect(onSubmit).not.toHaveBeenCalled();
  });

  it('exige al menos un ingrediente', async () => {
    const user = userEvent.setup();
    const { onSubmit } = renderForm();

    await user.click(await screen.findByRole('button', { name: /eliminar ingrediente 1/i }));
    await user.click(screen.getByRole('button', { name: /guardar/i }));

    expect(await screen.findByText(/añade al menos un ingrediente/i)).toBeInTheDocument();
    expect(onSubmit).not.toHaveBeenCalled();
  });

  it('envía los datos válidos normalizados', async () => {
    const user = userEvent.setup();
    const { onSubmit } = renderForm();

    await fillRequiredFields(user);
    await user.click(screen.getByRole('button', { name: /guardar/i }));

    await waitFor(() => expect(onSubmit).toHaveBeenCalledTimes(1));
    expect(onSubmit).toHaveBeenCalledWith(
      expect.objectContaining<Partial<RecipeInput>>({
        name: 'Tortilla',
        category: 'main',
        ingredients: [expect.objectContaining({ name: 'Patata', amount: 2, unit: 'ud' })],
      }),
    );
  });

  it('envía las etiquetas vacías cuando no se selecciona ninguna', async () => {
    const user = userEvent.setup();
    const { onSubmit } = renderForm();

    await fillRequiredFields(user);
    await user.click(screen.getByRole('button', { name: /guardar/i }));

    await waitFor(() => expect(onSubmit).toHaveBeenCalledTimes(1));
    expect(onSubmit).toHaveBeenCalledWith(expect.objectContaining({ tags: [] }));
  });

  it('permite seleccionar una etiqueta existente y la envía por nombre', async () => {
    const user = userEvent.setup();
    const { onSubmit } = renderForm();

    await fillRequiredFields(user);
    const tags = screen.getByRole('combobox', { name: /etiquetas/i });
    await user.click(tags);
    await user.click(
      await screen.findByText('Vegano', { selector: '.ant-select-item-option-content' }),
    );
    await user.click(screen.getByRole('button', { name: /guardar/i }));

    await waitFor(() => expect(onSubmit).toHaveBeenCalledTimes(1));
    expect(onSubmit).toHaveBeenCalledWith(expect.objectContaining({ tags: ['Vegano'] }));
  });

  it('permite crear una etiqueta al vuelo', async () => {
    const user = userEvent.setup();
    const { onSubmit } = renderForm();

    await fillRequiredFields(user);
    const tags = screen.getByRole('combobox', { name: /etiquetas/i });
    await user.type(tags, 'Casero{enter}');
    await user.click(screen.getByRole('button', { name: /guardar/i }));

    await waitFor(() => expect(onSubmit).toHaveBeenCalledTimes(1));
    expect(onSubmit).toHaveBeenCalledWith(expect.objectContaining({ tags: ['Casero'] }));
  });

  it('precarga las etiquetas de la receta al editar y las conserva en el payload', async () => {
    const user = userEvent.setup();
    const { onSubmit } = renderForm({ initial: recipe() });

    await user.click(screen.getByRole('button', { name: /guardar/i }));

    await waitFor(() => expect(onSubmit).toHaveBeenCalledTimes(1));
    expect(onSubmit).toHaveBeenCalledWith(expect.objectContaining({ tags: ['Vegano'] }));
  });

  it('el selector de etiquetas cumple el objetivo táctil de 48px y tiene rol accesible', async () => {
    renderForm();

    const tags = await screen.findByRole('combobox', { name: /etiquetas/i });
    expect(tags).toBeInTheDocument();
    expect(screen.getByText('Etiquetas')).toBeInTheDocument();
    expect(recipeFormCss).toMatch(/\.recipe-form__tags[\s\S]*?min-height:\s*48px/);
  });

  it('el botón de guardar cumple el objetivo táctil de 48px', async () => {
    renderForm();

    const submit = await screen.findByRole('button', { name: /guardar/i });
    expect(submit).toHaveClass('recipe-form__submit');

    expect(recipeFormCss).toMatch(/\.recipe-form__submit\s*\{[^}]*min-height:\s*48px/);
  });
});
