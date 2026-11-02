import { describe, expect, it, vi, beforeEach } from 'vitest';
import { render, screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { MemoryRouter } from 'react-router-dom';
import type { MealPlanEntryView } from '../api/types';
import { useMealPlan } from '../api/hooks';
import { MenuPage } from './MenuPage';

vi.mock('../api/hooks', () => ({
  useMealPlan: vi.fn(),
}));

const configState = vi.hoisted(() => ({ hasValet: false, url: '' }));

// La configuración de Valet se controla por test mediante un estado mockeado.
vi.mock('../lib/config', () => ({
  get VALET_URL() {
    return configState.url;
  },
  get hasValet() {
    return configState.hasValet;
  },
}));

const mockedUseMealPlan = vi.mocked(useMealPlan);

function entry(
  overrides: Partial<MealPlanEntryView> & Pick<MealPlanEntryView, 'id'>,
): MealPlanEntryView {
  return {
    date: '2026-11-02',
    meal_type: 'lunch',
    role: 'main',
    servings: 2,
    recipe: {
      id: `recipe-${overrides.id}`,
      name: 'Receta',
      category: 'Principal',
      diet_type: 'omnivora',
      prep_time_minutes: 20,
      servings: 2,
      created_at: '2026-10-01T10:00:00Z',
    },
    ...overrides,
  };
}

function mockQuery(overrides: Record<string, unknown>) {
  mockedUseMealPlan.mockReturnValue({
    data: undefined,
    isLoading: false,
    isError: false,
    error: null,
    refetch: vi.fn(),
    ...overrides,
  } as unknown as ReturnType<typeof useMealPlan>);
}

function renderMenu() {
  return render(
    <MemoryRouter initialEntries={['/?week=2026-11-02']}>
      <MenuPage />
    </MemoryRouter>,
  );
}

describe('MenuPage', () => {
  beforeEach(() => {
    mockedUseMealPlan.mockReset();
    configState.hasValet = false;
    configState.url = '';
  });

  it('renderiza siete tarjetas de lunes a domingo', () => {
    mockQuery({ data: [entry({ id: '1' })] });
    renderMenu();

    expect(screen.getAllByTestId('day-card')).toHaveLength(7);
    expect(screen.getByText('Lun 2')).toBeInTheDocument();
    expect(screen.getByText('Dom 8')).toBeInTheDocument();
  });

  it('muestra los nombres de receta en COMIDA y CENA', () => {
    mockQuery({
      data: [
        entry({
          id: '1',
          date: '2026-11-02',
          meal_type: 'lunch',
          recipe: { ...entry({ id: '1' }).recipe, name: 'Lentejas' },
        }),
        entry({
          id: '2',
          date: '2026-11-02',
          meal_type: 'dinner',
          recipe: { ...entry({ id: '2' }).recipe, name: 'Tortilla' },
        }),
      ],
    });
    renderMenu();

    const monday = screen.getAllByTestId('day-card')[0];
    expect(within(monday).getByText('Lentejas')).toBeInTheDocument();
    expect(within(monday).getByText('Tortilla')).toBeInTheDocument();
    expect(within(monday).getByText('COMIDA')).toBeInTheDocument();
    expect(within(monday).getByText('CENA')).toBeInTheDocument();
  });

  it('marca las ensaladas con su etiqueta', () => {
    mockQuery({
      data: [
        entry({
          id: '3',
          date: '2026-11-03',
          meal_type: 'lunch',
          role: 'salad',
          recipe: { ...entry({ id: '3' }).recipe, name: 'Ensalada César' },
        }),
      ],
    });
    renderMenu();

    const tuesday = screen.getAllByTestId('day-card')[1];
    expect(within(tuesday).getByText('Ensalada César')).toBeInTheDocument();
    expect(within(tuesday).getByText('🥗 Ensalada')).toBeInTheDocument();
  });

  it('marca la ensalada también cuando la categoría de la receta es "salad"', () => {
    mockQuery({
      data: [
        entry({
          id: '4',
          date: '2026-11-03',
          meal_type: 'lunch',
          role: 'main',
          recipe: {
            ...entry({ id: '4' }).recipe,
            name: 'Ensalada de lentejas',
            category: 'salad',
          },
        }),
      ],
    });
    renderMenu();

    const tuesday = screen.getAllByTestId('day-card')[1];
    expect(within(tuesday).getByText('Ensalada de lentejas')).toBeInTheDocument();
    expect(within(tuesday).getByText('🥗 Ensalada')).toBeInTheDocument();
  });

  it('muestra el estado vacío cuando no hay entradas', () => {
    mockQuery({ data: [] });
    renderMenu();

    expect(screen.getByText(/no hay comidas planificadas/i)).toBeInTheDocument();
    expect(screen.getByRole('link', { name: /lista de la compra/i })).toBeInTheDocument();
  });

  it('con VALET_URL configurada ofrece el botón para añadir comidas con Valet', () => {
    configState.hasValet = true;
    configState.url = 'http://valet.test';
    mockQuery({ data: [] });
    renderMenu();

    const valet = screen.getByRole('link', { name: /añadir comidas con valet/i });
    expect(valet).toHaveAttribute('href', 'http://valet.test');
    expect(screen.getByRole('link', { name: /lista de la compra/i })).toBeInTheDocument();
  });

  it('sin VALET_URL no muestra el botón de Valet pero sí el enlace a la compra', () => {
    mockQuery({ data: [] });
    renderMenu();

    expect(screen.queryByRole('link', { name: /valet/i })).not.toBeInTheDocument();
    expect(screen.getByRole('link', { name: /lista de la compra/i })).toBeInTheDocument();
  });

  it('muestra el esqueleto de carga mientras se resuelve la petición', () => {
    mockQuery({ data: undefined, isLoading: true });
    renderMenu();

    expect(screen.getByTestId('menu-loading')).toBeInTheDocument();
    expect(screen.queryByTestId('day-card')).not.toBeInTheDocument();
  });

  it('muestra el error y reintenta la petición', async () => {
    const refetch = vi.fn();
    mockQuery({ isError: true, error: new Error('boom'), refetch });
    const user = userEvent.setup();
    renderMenu();

    await user.click(screen.getByRole('button', { name: /reintentar/i }));

    expect(refetch).toHaveBeenCalledTimes(1);
  });

  it('navega a la semana siguiente actualizando la URL', async () => {
    mockQuery({ data: [] });
    const user = userEvent.setup();
    renderMenu();

    await user.click(screen.getByRole('button', { name: /semana siguiente/i }));

    expect(mockedUseMealPlan).toHaveBeenLastCalledWith('2026-11-09', '2026-11-15');
  });
});
