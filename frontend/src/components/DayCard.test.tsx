import { describe, expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { MemoryRouter } from 'react-router-dom';
import type { MealPlanEntryView, RecipeSummary } from '../api/types';
import { DayCard } from './DayCard';

function summary(overrides: Partial<RecipeSummary> = {}): RecipeSummary {
  return {
    id: 'r1',
    name: 'Tortilla',
    category: 'main',
    diet_type: 'mediterranean',
    prep_time_minutes: 20,
    servings: 2,
    created_at: '2026-11-01T10:00:00Z',
    tags: [],
    ...overrides,
  };
}

function entry(overrides: Partial<MealPlanEntryView> = {}): MealPlanEntryView {
  return {
    id: 'e1',
    date: '2026-11-02',
    meal_type: 'lunch',
    role: 'main',
    servings: 2,
    recipe: summary(),
    ...overrides,
  };
}

function renderCard(props: Partial<Parameters<typeof DayCard>[0]> = {}) {
  const onAddToShoppingList = vi.fn();
  render(
    <MemoryRouter>
      <DayCard
        date="2026-11-02"
        entries={[entry()]}
        onAddToShoppingList={onAddToShoppingList}
        {...props}
      />
    </MemoryRouter>,
  );
  return { onAddToShoppingList };
}

describe('DayCard', () => {
  it('enlaza la receta a su detalle', () => {
    renderCard({ entries: [entry({ recipe: summary({ id: 'r-tortilla', name: 'Tortilla' }) })] });

    expect(screen.getByRole('link', { name: 'Tortilla' })).toHaveAttribute(
      'href',
      '/recipes/r-tortilla',
    );
  });

  it('añade el día a la lista de la compra', async () => {
    const user = userEvent.setup();
    const { onAddToShoppingList } = renderCard();

    await user.click(
      screen.getByRole('button', { name: 'Añadir los ingredientes de Lun 2 a la lista' }),
    );

    expect(onAddToShoppingList).toHaveBeenCalledWith('2026-11-02');
  });

  it('sin conexión deshabilita el botón de añadir', () => {
    renderCard({ online: false });

    expect(
      screen.getByRole('button', { name: 'Añadir los ingredientes de Lun 2 a la lista' }),
    ).toBeDisabled();
  });

  it('muestra el spinner mientras la petición está en curso', () => {
    renderCard({ addingToShoppingList: true, addingToShoppingListDate: '2026-11-02' });

    const button = screen.getByRole('button', {
      name: /añadir los ingredientes de lun 2 a la lista/i,
    });
    expect(button).toHaveClass('ant-btn-loading');
    expect(button).toBeDisabled();
  });

  it('no renderiza el botón si no hay handler', () => {
    render(
      <MemoryRouter>
        <DayCard date="2026-11-02" entries={[entry()]} />
      </MemoryRouter>,
    );

    expect(
      screen.queryByRole('button', { name: 'Añadir los ingredientes de Lun 2 a la lista' }),
    ).not.toBeInTheDocument();
  });
});
