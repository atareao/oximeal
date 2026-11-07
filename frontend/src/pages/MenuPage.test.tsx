import { describe, expect, it, vi, beforeEach } from 'vitest';
import { render, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { MemoryRouter, useLocation } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { App as AntdApp } from 'antd';
import type {
  MealPlanEntryView,
  MealPlanInput,
  RecipeListResponse,
  RecipeSummary,
} from '../api/types';
import {
  createMealPlanEntry,
  deleteMealPlanEntry,
  fetchMealPlan,
  fetchRecipes,
  patchMealPlanEntry,
} from '../api/client';
import { useOnline } from '../offline/connectivity';
import { toDateString } from '../lib/dates';
import { MenuPage } from './MenuPage';
import dayCardCss from '../components/DayCard.css?raw';
import addMealPlanCss from '../components/AddMealPlanDialog.css?raw';

// Se mockea el cliente HTTP (no los hooks) para ejercer la integración real de
// React Query: alta → invalidación → refetch de `fetchMealPlan`.
vi.mock('../api/client', async (importOriginal) => {
  const actual = await importOriginal<typeof import('../api/client')>();
  return {
    ...actual,
    fetchMealPlan: vi.fn(),
    fetchRecipes: vi.fn(),
    createMealPlanEntry: vi.fn(),
    deleteMealPlanEntry: vi.fn(),
    patchMealPlanEntry: vi.fn(),
  };
});

vi.mock('../offline/connectivity', () => ({
  useOnline: vi.fn(() => true),
  isOnline: vi.fn(() => true),
}));

const configState = vi.hoisted(() => ({ hasValet: false, url: '' }));

vi.mock('../lib/config', () => ({
  get VALET_URL() {
    return configState.url;
  },
  get hasValet() {
    return configState.hasValet;
  },
}));

const mockedFetchMealPlan = vi.mocked(fetchMealPlan);
const mockedFetchRecipes = vi.mocked(fetchRecipes);
const mockedCreateMealPlan = vi.mocked(createMealPlanEntry);
const mockedDeleteMealPlan = vi.mocked(deleteMealPlanEntry);
const mockedPatchMealPlan = vi.mocked(patchMealPlanEntry);
const mockedUseOnline = vi.mocked(useOnline);

const PAGE_SIZE = 20;

function installMatchMedia(matches = false) {
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

function summary(overrides: Partial<RecipeSummary> & Pick<RecipeSummary, 'id' | 'name'>) {
  return {
    category: 'main',
    diet_type: 'omnivora',
    prep_time_minutes: 20,
    servings: 2,
    created_at: '2026-10-01T10:00:00Z',
    tags: [],
    ...overrides,
  } satisfies RecipeSummary;
}

let recipes: RecipeSummary[] = [];
let entries: MealPlanEntryView[] = [];
let idSeq = 0;

function view(
  overrides: Partial<MealPlanEntryView> & Pick<MealPlanEntryView, 'id'>,
): MealPlanEntryView {
  return {
    date: '2026-11-02',
    meal_type: 'lunch',
    role: 'main',
    servings: 2,
    recipe: summary({ id: `recipe-${overrides.id}`, name: 'Receta' }),
    ...overrides,
  };
}

function recipesPage(): RecipeListResponse {
  return {
    items: recipes.slice(0, PAGE_SIZE),
    total: recipes.length,
    page: 1,
    page_size: PAGE_SIZE,
  };
}

function LocationProbe() {
  const location = useLocation();
  return (
    <div data-testid="location">
      {location.pathname}
      {location.search}
    </div>
  );
}

function tree(
  queryClient = new QueryClient({
    defaultOptions: { queries: { retry: false, gcTime: 0 }, mutations: { retry: false } },
  }),
  initialEntry = '/?week=2026-11-02',
) {
  return (
    <QueryClientProvider client={queryClient}>
      <AntdApp>
        <MemoryRouter initialEntries={[initialEntry]}>
          <LocationProbe />
          <MenuPage />
        </MemoryRouter>
      </AntdApp>
    </QueryClientProvider>
  );
}

function renderMenu() {
  return render(tree());
}

function renderMenuAt(initialEntry: string) {
  return render(tree(undefined, initialEntry));
}

function cardForDay(index: number): HTMLElement {
  return screen.getAllByTestId('day-card')[index];
}

describe('MenuPage', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    installMatchMedia();
    configState.hasValet = false;
    configState.url = '';
    recipes = [
      summary({ id: 'r-lentejas', name: 'Lentejas' }),
      summary({ id: 'r-tortilla', name: 'Tortilla' }),
    ];
    entries = [];
    idSeq = 0;
    mockedUseOnline.mockReturnValue(true);

    mockedFetchRecipes.mockImplementation(() => Promise.resolve(recipesPage()));
    mockedFetchMealPlan.mockImplementation((start, end) =>
      Promise.resolve(entries.filter((e) => e.date >= start && e.date <= end)),
    );
    mockedCreateMealPlan.mockImplementation((input: MealPlanInput) => {
      const recipe = recipes.find((r) => r.id === input.recipe_id);
      const created = view({
        id: `m${++idSeq}`,
        date: input.date,
        meal_type: input.meal_type,
        role: input.role,
        servings: input.servings ?? 2,
        recipe: recipe ?? summary({ id: input.recipe_id, name: 'Receta' }),
      });
      entries.push(created);
      return Promise.resolve(created);
    });
    mockedDeleteMealPlan.mockImplementation((id: string) => {
      entries = entries.filter((e) => e.id !== id);
      return Promise.resolve();
    });
    mockedPatchMealPlan.mockImplementation((id: string, input: { servings: number }) => {
      const index = entries.findIndex((e) => e.id === id);
      const updated = { ...entries[index], servings: input.servings };
      entries[index] = updated;
      return Promise.resolve(updated);
    });
  });

  it('renderiza siete tarjetas de lunes a domingo', async () => {
    entries = [view({ id: '1' })];
    renderMenu();

    await waitFor(() => expect(screen.getAllByTestId('day-card')).toHaveLength(7));
    expect(screen.getByText('Lun 2')).toBeInTheDocument();
    expect(screen.getByText('Dom 8')).toBeInTheDocument();
  });

  it('muestra los nombres de receta en COMIDA y CENA', async () => {
    entries = [
      view({ id: '1', meal_type: 'lunch', recipe: summary({ id: 'a', name: 'Lentejas' }) }),
      view({ id: '2', meal_type: 'dinner', recipe: summary({ id: 'b', name: 'Tortilla' }) }),
    ];
    renderMenu();

    await screen.findByTestId('weekly-calendar');
    const monday = cardForDay(0);
    await waitFor(() => expect(within(monday).getByText('Lentejas')).toBeInTheDocument());
    expect(within(monday).getByText('Tortilla')).toBeInTheDocument();
    expect(within(monday).getByText('COMIDA')).toBeInTheDocument();
    expect(within(monday).getByText('CENA')).toBeInTheDocument();
  });

  it('marca las ensaladas con su etiqueta', async () => {
    entries = [
      view({
        id: '3',
        date: '2026-11-03',
        role: 'salad',
        recipe: summary({ id: 'c', name: 'Ensalada César' }),
      }),
    ];
    renderMenu();

    await screen.findByTestId('weekly-calendar');
    const tuesday = cardForDay(1);
    await waitFor(() => expect(within(tuesday).getByText('Ensalada César')).toBeInTheDocument());
    expect(within(tuesday).getByText('🥗 Ensalada')).toBeInTheDocument();
  });

  it('añade una receta a un slot usando el meal_type del slot', async () => {
    entries = [view({ id: 'seed', date: '2026-11-02' })];
    const user = userEvent.setup();
    renderMenu();

    const monday = await screen.findByTestId('weekly-calendar').then(() => cardForDay(0));
    await user.click(within(monday).getByRole('button', { name: 'Añadir receta a COMIDA' }));

    const dialog = await screen.findByRole('dialog');
    // El slot fija el tipo de comida: no se ofrece volver a elegirlo.
    expect(within(dialog).queryByLabelText(/tipo de comida/i)).not.toBeInTheDocument();

    await user.type(within(dialog).getByLabelText(/buscar recetas/i), 'len');
    await waitFor(() =>
      expect(mockedFetchRecipes).toHaveBeenLastCalledWith(
        expect.objectContaining({ q: 'len' }),
        expect.anything(),
      ),
    );
    await user.click(await within(dialog).findByRole('button', { name: 'Lentejas' }));

    await user.selectOptions(within(dialog).getByLabelText(/^rol$/i), 'salad');

    await user.click(within(dialog).getByRole('button', { name: 'Añadir' }));

    await waitFor(() =>
      expect(mockedCreateMealPlan).toHaveBeenCalledWith({
        date: '2026-11-02',
        meal_type: 'lunch',
        recipe_id: 'r-lentejas',
        role: 'salad',
        servings: 2,
      }),
    );

    const updatedMonday = cardForDay(0);
    expect(await within(updatedMonday).findByText('Lentejas')).toBeInTheDocument();
  });

  it('usa el meal_type CENA al añadir desde el slot de cena', async () => {
    entries = [view({ id: 'seed' })];
    const user = userEvent.setup();
    renderMenu();

    await screen.findByTestId('weekly-calendar');
    const monday = cardForDay(0);
    await user.click(within(monday).getByRole('button', { name: 'Añadir receta a CENA' }));

    const dialog = await screen.findByRole('dialog');
    await user.click(await within(dialog).findByRole('button', { name: 'Tortilla' }));
    await user.click(within(dialog).getByRole('button', { name: 'Añadir' }));

    await waitFor(() =>
      expect(mockedCreateMealPlan).toHaveBeenCalledWith(
        expect.objectContaining({ meal_type: 'dinner', recipe_id: 'r-tortilla' }),
      ),
    );
  });

  it('valida que haya una receta seleccionada y no envía', async () => {
    entries = [view({ id: 'seed' })];
    const user = userEvent.setup();
    renderMenu();

    await screen.findByTestId('weekly-calendar');
    await user.click(within(cardForDay(0)).getByRole('button', { name: 'Añadir receta a COMIDA' }));

    const dialog = await screen.findByRole('dialog');
    await user.click(within(dialog).getByRole('button', { name: 'Añadir' }));

    expect(await within(dialog).findByText(/selecciona una receta/i)).toBeInTheDocument();
    expect(mockedCreateMealPlan).not.toHaveBeenCalled();
  });

  it('valida que las raciones sean al menos 1 y no envía', async () => {
    entries = [view({ id: 'seed' })];
    const user = userEvent.setup();
    renderMenu();

    await screen.findByTestId('weekly-calendar');
    await user.click(within(cardForDay(0)).getByRole('button', { name: 'Añadir receta a COMIDA' }));

    const dialog = await screen.findByRole('dialog');
    await user.click(await within(dialog).findByRole('button', { name: 'Lentejas' }));

    const servings = within(dialog).getByLabelText(/raciones/i);
    await user.clear(servings);
    await user.type(servings, '0');
    await user.click(within(dialog).getByRole('button', { name: 'Añadir' }));

    expect(await within(dialog).findByText(/al menos 1/i)).toBeInTheDocument();
    expect(mockedCreateMealPlan).not.toHaveBeenCalled();
  });

  it('muestra estado de carga y evita el doble envío durante el alta', async () => {
    entries = [view({ id: 'seed' })];
    let resolveCreate: (value: MealPlanEntryView) => void = () => undefined;
    mockedCreateMealPlan.mockImplementation(
      () =>
        new Promise<MealPlanEntryView>((resolve) => {
          resolveCreate = resolve;
        }),
    );
    const user = userEvent.setup();
    renderMenu();

    await screen.findByTestId('weekly-calendar');
    await user.click(within(cardForDay(0)).getByRole('button', { name: 'Añadir receta a COMIDA' }));

    const dialog = await screen.findByRole('dialog');
    await user.click(await within(dialog).findByRole('button', { name: 'Lentejas' }));
    await user.click(within(dialog).getByRole('button', { name: 'Añadir' }));

    await waitFor(() => expect(mockedCreateMealPlan).toHaveBeenCalledTimes(1));
    const confirm = within(dialog).getByRole('button', { name: /Añadir/ });
    expect(confirm).toHaveClass('ant-btn-loading');

    // El botón en carga ignora el segundo clic: no hay doble envío.
    await user.click(confirm);
    expect(mockedCreateMealPlan).toHaveBeenCalledTimes(1);

    resolveCreate(
      view({
        id: 'm-new',
        date: '2026-11-02',
        recipe: summary({ id: 'r-lentejas', name: 'Lentejas' }),
      }),
    );
  });

  it('muestra el error del servidor y conserva el estado previo del calendario', async () => {
    entries = [view({ id: 'seed', recipe: summary({ id: 'x', name: 'Receta previa' }) })];
    mockedCreateMealPlan.mockRejectedValueOnce(new Error('boom'));
    const user = userEvent.setup();
    renderMenu();

    await screen.findByTestId('weekly-calendar');
    const before = cardForDay(0);
    expect(within(before).getByText('Receta previa')).toBeInTheDocument();

    await user.click(within(before).getByRole('button', { name: 'Añadir receta a COMIDA' }));
    const dialog = await screen.findByRole('dialog');
    await user.click(await within(dialog).findByRole('button', { name: 'Lentejas' }));
    await user.click(within(dialog).getByRole('button', { name: 'Añadir' }));

    expect(await within(dialog).findByText(/no se pudo añadir/i)).toBeInTheDocument();
    expect(within(cardForDay(0)).getByText('Receta previa')).toBeInTheDocument();
  });

  it('muestra el error de carga del catálogo en el selector con reintento', async () => {
    entries = [view({ id: 'seed' })];
    mockedFetchRecipes.mockRejectedValueOnce(new Error('boom'));
    const user = userEvent.setup();
    renderMenu();

    await screen.findByTestId('weekly-calendar');
    await user.click(within(cardForDay(0)).getByRole('button', { name: 'Añadir receta a COMIDA' }));

    const dialog = await screen.findByRole('dialog');
    expect(
      await within(dialog).findByText(/no se pudieron cargar las recetas/i),
    ).toBeInTheDocument();

    await user.click(within(dialog).getByRole('button', { name: /reintentar/i }));

    expect(await within(dialog).findByRole('button', { name: 'Lentejas' })).toBeInTheDocument();
  });

  it('quita una entrada tras confirmar en el diálogo', async () => {
    entries = [view({ id: 'e1', recipe: summary({ id: 'x', name: 'Lentejas' }) })];
    const user = userEvent.setup();
    renderMenu();

    await screen.findByTestId('weekly-calendar');
    await user.click(within(cardForDay(0)).getByRole('button', { name: 'Quitar Lentejas' }));
    await user.click(await screen.findByRole('button', { name: 'Quitar' }));

    await waitFor(() => expect(mockedDeleteMealPlan).toHaveBeenCalledWith('e1'));
    await waitFor(() => expect(screen.queryByText('Lentejas')).not.toBeInTheDocument());
  });

  it('cancelar la baja no elimina la entrada', async () => {
    entries = [view({ id: 'e1', recipe: summary({ id: 'x', name: 'Lentejas' }) })];
    const user = userEvent.setup();
    renderMenu();

    await screen.findByTestId('weekly-calendar');
    await user.click(within(cardForDay(0)).getByRole('button', { name: 'Quitar Lentejas' }));
    await user.click(await screen.findByRole('button', { name: 'Cancelar' }));

    expect(mockedDeleteMealPlan).not.toHaveBeenCalled();
    expect(within(cardForDay(0)).getByText('Lentejas')).toBeInTheDocument();
  });

  it('edita las raciones y persiste con PATCH conservando la entrada', async () => {
    entries = [view({ id: 'e1', servings: 2, recipe: summary({ id: 'x', name: 'Lentejas' }) })];
    const user = userEvent.setup();
    renderMenu();

    await screen.findByTestId('weekly-calendar');
    await user.click(
      within(cardForDay(0)).getByRole('button', { name: 'Editar raciones de Lentejas' }),
    );

    const dialog = await screen.findByRole('dialog');
    const servings = within(dialog).getByLabelText(/raciones/i);
    await user.clear(servings);
    await user.type(servings, '4');
    await user.click(within(dialog).getByRole('button', { name: /guardar/i }));

    await waitFor(() => expect(mockedPatchMealPlan).toHaveBeenCalledWith('e1', { servings: 4 }));
    await waitFor(() => expect(within(cardForDay(0)).getByText('Lentejas')).toBeInTheDocument());
    expect(within(cardForDay(0)).getByText(/4/)).toBeInTheDocument();
  });

  it('valida las raciones al editar y no envía si son menores que 1', async () => {
    entries = [view({ id: 'e1', servings: 2, recipe: summary({ id: 'x', name: 'Lentejas' }) })];
    const user = userEvent.setup();
    renderMenu();

    await screen.findByTestId('weekly-calendar');
    await user.click(
      within(cardForDay(0)).getByRole('button', { name: 'Editar raciones de Lentejas' }),
    );

    const dialog = await screen.findByRole('dialog');
    const servings = within(dialog).getByLabelText(/raciones/i);
    await user.clear(servings);
    await user.type(servings, '0');
    await user.click(within(dialog).getByRole('button', { name: /guardar/i }));

    expect(await within(dialog).findByText(/al menos 1/i)).toBeInTheDocument();
    expect(mockedPatchMealPlan).not.toHaveBeenCalled();
  });

  it('muestra el estado vacío con ambas vías: nativa y Valet', async () => {
    configState.hasValet = true;
    configState.url = 'http://valet.test';
    entries = [];
    renderMenu();

    expect(await screen.findByText(/no hay comidas planificadas/i)).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /planificar/i })).toBeInTheDocument();
    expect(screen.getByRole('link', { name: /valet/i })).toHaveAttribute(
      'href',
      'http://valet.test',
    );
    expect(screen.getByRole('link', { name: /lista de la compra/i })).toBeInTheDocument();
  });

  it('mantiene la acción de Valet disponible con la vista con datos', async () => {
    configState.hasValet = true;
    configState.url = 'http://valet.test';
    entries = [view({ id: '1' })];
    renderMenu();

    await screen.findByTestId('weekly-calendar');
    expect(screen.getByRole('link', { name: /valet/i })).toHaveAttribute(
      'href',
      'http://valet.test',
    );
  });

  it('sin conexión deshabilita los controles, avisa y no llama al servidor', async () => {
    mockedUseOnline.mockReturnValue(false);
    entries = [view({ id: 'e1', recipe: summary({ id: 'x', name: 'Lentejas' }) })];
    renderMenu();

    await screen.findByTestId('weekly-calendar');
    expect(screen.getByTestId('menu-offline')).toBeInTheDocument();
    expect(
      within(cardForDay(0)).getByRole('button', { name: 'Añadir receta a COMIDA' }),
    ).toBeDisabled();
    expect(within(cardForDay(0)).getByRole('button', { name: 'Quitar Lentejas' })).toBeDisabled();
    expect(
      within(cardForDay(0)).getByRole('button', { name: 'Editar raciones de Lentejas' }),
    ).toBeDisabled();

    expect(mockedCreateMealPlan).not.toHaveBeenCalled();
    expect(mockedDeleteMealPlan).not.toHaveBeenCalled();
    expect(mockedPatchMealPlan).not.toHaveBeenCalled();
  });

  it('rehabilita los controles y retira el aviso al recuperar la conexión', async () => {
    const queryClient = new QueryClient({
      defaultOptions: { queries: { retry: false, gcTime: 0 }, mutations: { retry: false } },
    });
    mockedUseOnline.mockReturnValue(false);
    entries = [view({ id: 'e1', recipe: summary({ id: 'x', name: 'Lentejas' }) })];
    const { rerender } = render(tree(queryClient));

    await screen.findByTestId('weekly-calendar');
    expect(screen.getByTestId('menu-offline')).toBeInTheDocument();
    expect(
      within(cardForDay(0)).getByRole('button', { name: 'Añadir receta a COMIDA' }),
    ).toBeDisabled();

    // La conexión se recupera: el mock pasa a `true` y se re-renderiza el mismo
    // árbol (mismo QueryClient, sin remontar el subárbol).
    const callsWhileOffline = mockedFetchMealPlan.mock.calls.length;
    mockedUseOnline.mockReturnValue(true);
    rerender(tree(queryClient));

    await waitFor(() => expect(screen.queryByTestId('menu-offline')).not.toBeInTheDocument());
    // El plan se refresca desde el servidor al recuperar la conexión.
    await waitFor(() =>
      expect(mockedFetchMealPlan.mock.calls.length).toBeGreaterThan(callsWhileOffline),
    );
    expect(
      within(cardForDay(0)).getByRole('button', { name: 'Añadir receta a COMIDA' }),
    ).toBeEnabled();
    expect(within(cardForDay(0)).getByRole('button', { name: 'Quitar Lentejas' })).toBeEnabled();
    expect(
      within(cardForDay(0)).getByRole('button', { name: 'Editar raciones de Lentejas' }),
    ).toBeEnabled();
  });

  it('expone foco y nombres accesibles al abrir y cerrar el selector', async () => {
    entries = [view({ id: 'seed' })];
    const user = userEvent.setup();
    renderMenu();

    await screen.findByTestId('weekly-calendar');
    const addButton = within(cardForDay(0)).getByRole('button', { name: 'Añadir receta a COMIDA' });
    await user.click(addButton);

    const dialog = await screen.findByRole('dialog');
    await waitFor(() => expect(dialog.contains(document.activeElement)).toBe(true));

    await user.click(within(dialog).getByRole('button', { name: 'Cancelar' }));
    await waitFor(() => expect(addButton).toHaveFocus());
  });

  it('define objetivos táctiles de al menos 48px para los controles de planificación', async () => {
    entries = [view({ id: 'seed' })];
    renderMenu();

    await screen.findByTestId('weekly-calendar');
    expect(dayCardCss).toMatch(/\.day-card__add\s*\{[^}]*min-height:\s*48px/);
    expect(dayCardCss).toMatch(/\.day-card__remove\s*\{[^}]*min-height:\s*48px/);
    expect(dayCardCss).toMatch(/\.day-card__edit\s*\{[^}]*min-height:\s*48px/);
    expect(addMealPlanCss).toMatch(/\.add-meal-plan__recipe\s*\{[^}]*min-height:\s*48px/);
    expect(addMealPlanCss).toMatch(/\.add-meal-plan__confirm\s*\{[^}]*min-height:\s*48px/);
  });

  it('navega a la semana siguiente actualizando la URL', async () => {
    entries = [];
    const user = userEvent.setup();
    renderMenu();

    await user.click(screen.getByRole('button', { name: /semana siguiente/i }));

    await waitFor(() =>
      expect(mockedFetchMealPlan).toHaveBeenLastCalledWith(
        '2026-11-09',
        '2026-11-15',
        expect.anything(),
      ),
    );
  });

  it('muestra el esqueleto de carga mientras se resuelve la petición', () => {
    mockedFetchMealPlan.mockImplementation(() => new Promise(() => undefined));
    renderMenu();

    expect(screen.getByTestId('menu-loading')).toBeInTheDocument();
    expect(screen.queryByTestId('day-card')).not.toBeInTheDocument();
  });

  it('muestra el error de carga del menú y reintenta', async () => {
    mockedFetchMealPlan.mockRejectedValueOnce(new Error('boom'));
    const user = userEvent.setup();
    renderMenu();

    expect(await screen.findByText(/no se pudo cargar el menú/i)).toBeInTheDocument();

    mockedFetchMealPlan.mockImplementation(() => Promise.resolve([]));
    await user.click(screen.getByRole('button', { name: /reintentar/i }));

    expect(await screen.findByText(/no hay comidas planificadas/i)).toBeInTheDocument();
  });

  it('sin view en la URL muestra la vista semanal por defecto', async () => {
    entries = [view({ id: 'seed' })];
    renderMenuAt('/?week=2026-11-02');

    expect(await screen.findByTestId('weekly-calendar')).toBeInTheDocument();
    expect(screen.getByTestId('location')).not.toHaveTextContent('view=day');
    expect(screen.getByTestId('location')).not.toHaveTextContent('view=month');
  });

  it('cambiar a Diaria muestra un solo día y actualiza view=day', async () => {
    entries = [view({ id: 'seed' })];
    const user = userEvent.setup();
    renderMenuAt('/?week=2026-11-02');

    await screen.findByTestId('weekly-calendar');
    await user.click(screen.getByText('Diaria'));

    await waitFor(() => expect(screen.getAllByTestId('day-card')).toHaveLength(1));
    expect(screen.getByTestId('location')).toHaveTextContent('view=day');
  });

  it('cambiar a Mensual muestra la rejilla del mes y actualiza view=month', async () => {
    entries = [view({ id: 'seed' })];
    const user = userEvent.setup();
    renderMenuAt('/?week=2026-11-02');

    await screen.findByTestId('weekly-calendar');
    await user.click(screen.getByText('Mensual'));

    expect(await screen.findByTestId('monthly-calendar')).toBeInTheDocument();
    expect(screen.getByTestId('location')).toHaveTextContent('view=month');
  });

  it('en la vista mensual tocar un día navega a view=day con la fecha', async () => {
    entries = [view({ id: 'seed' })];
    const user = userEvent.setup();
    renderMenuAt('/?view=month&date=2026-11-01');

    await screen.findByTestId('monthly-calendar');
    await user.click(screen.getByTestId('month-day-2026-11-15'));

    await waitFor(() => expect(screen.getByTestId('location')).toHaveTextContent('view=day'));
    expect(screen.getByTestId('location')).toHaveTextContent('date=2026-11-15');
  });

  it('en la vista diaria los botones anterior, siguiente y Hoy actualizan la fecha', async () => {
    entries = [view({ id: 'seed' })];
    const user = userEvent.setup();
    renderMenuAt('/?view=day&date=2026-11-10');

    await screen.findByTestId('daily-view');

    await user.click(screen.getByRole('button', { name: /día siguiente/i }));
    await waitFor(() =>
      expect(screen.getByTestId('location')).toHaveTextContent('date=2026-11-11'),
    );

    await user.click(screen.getByRole('button', { name: /día anterior/i }));
    await waitFor(() =>
      expect(screen.getByTestId('location')).toHaveTextContent('date=2026-11-10'),
    );

    await user.click(screen.getByRole('button', { name: /^hoy$/i }));
    await waitFor(() =>
      expect(screen.getByTestId('location')).toHaveTextContent(`date=${toDateString(new Date())}`),
    );
  });

  it('la vista mensual pide a useMealPlan el rango del mes visible', async () => {
    entries = [view({ id: 'seed' })];
    renderMenuAt('/?view=month&date=2026-11-01');

    await screen.findByTestId('monthly-calendar');

    await waitFor(() =>
      expect(mockedFetchMealPlan).toHaveBeenLastCalledWith(
        '2026-11-01',
        '2026-11-30',
        expect.anything(),
      ),
    );
  });

  it('un view inválido muestra la vista semanal y normaliza la URL', async () => {
    entries = [view({ id: 'seed' })];
    renderMenuAt('/?view=xxx&week=2026-11-02');

    expect(await screen.findByTestId('weekly-calendar')).toBeInTheDocument();
    await waitFor(() => expect(screen.getByTestId('location')).toHaveTextContent('view=week'));
    expect(screen.getByTestId('location')).not.toHaveTextContent('view=xxx');
  });

  it('una semana que cruza de año pide el rango correcto', async () => {
    entries = [view({ id: 'seed', date: '2026-12-28' })];
    renderMenuAt('/?week=2026-12-28');

    await screen.findByTestId('weekly-calendar');

    await waitFor(() =>
      expect(mockedFetchMealPlan).toHaveBeenLastCalledWith(
        '2026-12-28',
        '2027-01-03',
        expect.anything(),
      ),
    );
  });

  it('la vista diaria muestra solo las franjas COMIDA y CENA y descarta desayuno y merienda', async () => {
    entries = [
      view({
        id: 'b',
        date: '2026-11-02',
        meal_type: 'breakfast',
        recipe: summary({ id: 'rb', name: 'Tostadas' }),
      }),
      view({
        id: 'l',
        date: '2026-11-02',
        meal_type: 'lunch',
        recipe: summary({ id: 'rl', name: 'Lentejas' }),
      }),
      view({
        id: 'd',
        date: '2026-11-02',
        meal_type: 'dinner',
        recipe: summary({ id: 'rd', name: 'Tortilla' }),
      }),
      view({
        id: 's',
        date: '2026-11-02',
        meal_type: 'snack',
        recipe: summary({ id: 'rs', name: 'Fruta' }),
      }),
    ];
    renderMenuAt('/?view=day&date=2026-11-02');

    const card = await screen.findByTestId('day-card');
    expect(within(card).getByText('COMIDA')).toBeInTheDocument();
    expect(within(card).getByText('CENA')).toBeInTheDocument();
    expect(within(card).queryByText('DESAYUNO')).not.toBeInTheDocument();
    expect(within(card).queryByText('MERIENDA')).not.toBeInTheDocument();
    expect(within(card).getByText('Lentejas')).toBeInTheDocument();
    expect(within(card).getByText('Tortilla')).toBeInTheDocument();
    expect(within(card).queryByText('Tostadas')).not.toBeInTheDocument();
    expect(within(card).queryByText('Fruta')).not.toBeInTheDocument();
  });

  it('el selector de tipo de comida ofrece solo Comida y Cena', async () => {
    entries = [];
    const user = userEvent.setup();
    renderMenu();

    await user.click(await screen.findByRole('button', { name: 'Planificar una comida' }));

    const dialog = await screen.findByRole('dialog');
    const select = within(dialog).getByLabelText(/tipo de comida/i);
    const options = within(select)
      .getAllByRole('option')
      .map((option) => option.textContent);
    expect(options).toEqual(['Comida', 'Cena']);
    expect(within(select).queryByRole('option', { name: 'Desayuno' })).not.toBeInTheDocument();
    expect(within(select).queryByRole('option', { name: 'Merienda' })).not.toBeInTheDocument();
  });

  it('expone el encabezado de la vista oculto para lectores de pantalla', () => {
    renderMenu();

    expect(screen.getByRole('heading', { name: 'Menú' })).toHaveClass('visually-hidden');
  });

  it('muestra la fecha/rango debajo del selector de vistas en el DOM', () => {
    renderMenu();

    const controls = document.querySelector('.menu-page__controls');
    const range = document.querySelector('.menu-page__range');
    const segmented = document.querySelector('.ant-segmented');
    expect(controls).toBeInTheDocument();
    expect(range).toBeInTheDocument();
    expect(segmented).toBeInTheDocument();

    // El rango es hijo directo de los controles y va después del Segmented.
    expect(controls?.contains(range)).toBe(true);
    expect(range?.parentElement).toBe(controls);
    const children = Array.from(controls?.children ?? []);
    expect(children.indexOf(segmented as Element)).toBeLessThan(children.indexOf(range as Element));
    expect(
      (segmented as Node).compareDocumentPosition(range as Node) & Node.DOCUMENT_POSITION_FOLLOWING,
    ).toBeTruthy();
  });
});
