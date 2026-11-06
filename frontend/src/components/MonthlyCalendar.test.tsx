import { describe, expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import type { MealPlanEntryView, MealType } from '../api/types';
import { MonthlyCalendar } from './MonthlyCalendar';

function entry(date: string, meal_type: MealType, id: string): MealPlanEntryView {
  return {
    id,
    date,
    meal_type,
    role: 'main',
    servings: 2,
    recipe: {
      id: `recipe-${id}`,
      name: `Receta ${id}`,
      category: 'otro',
      diet_type: 'omnivora',
      prep_time_minutes: null,
      servings: 2,
      created_at: '2026-01-01T00:00:00Z',
      tags: [],
    },
  };
}

describe('MonthlyCalendar', () => {
  it('renderiza una celda por cada día del mes', () => {
    render(<MonthlyCalendar monthStart="2026-12-01" entries={[]} onSelectDay={vi.fn()} />);

    expect(screen.getAllByTestId(/^month-day-/)).toHaveLength(31);
  });

  it('alinea el día 1 con su weekday', () => {
    const { container } = render(
      <MonthlyCalendar monthStart="2026-12-01" entries={[]} onSelectDay={vi.fn()} />,
    );

    const grid = container.querySelector('.monthly-calendar__grid');
    expect(grid).not.toBeNull();
    const cells = Array.from(grid!.children);
    // El 1 de diciembre de 2026 es martes: un hueco inicial (lunes).
    expect(cells[0]).toHaveClass('monthly-calendar__blank');
    expect(cells[1]).toHaveAttribute('data-testid', 'month-day-2026-12-01');
  });

  it('el resumen cuenta solo COMIDA y CENA, ignorando franjas legacy', () => {
    render(
      <MonthlyCalendar
        monthStart="2026-12-01"
        entries={[
          entry('2026-12-15', 'lunch', '1'),
          entry('2026-12-15', 'dinner', '2'),
          entry('2026-12-15', 'breakfast', '3'),
          entry('2026-12-15', 'snack', '4'),
        ]}
        onSelectDay={vi.fn()}
      />,
    );

    const cell = screen.getByTestId('month-day-2026-12-15');
    expect(cell).toHaveAttribute('aria-label', '15 de diciembre de 2026, 2 comidas');
  });

  it('cada celda tiene un aria-label con la fecha completa y el resumen', () => {
    render(<MonthlyCalendar monthStart="2026-12-01" entries={[]} onSelectDay={vi.fn()} />);

    expect(screen.getByTestId('month-day-2026-12-15')).toHaveAttribute(
      'aria-label',
      '15 de diciembre de 2026, —',
    );
  });
});
