import { describe, expect, it, vi } from 'vitest';
import { render, screen } from '@testing-library/react';
import { MonthlyCalendar } from './MonthlyCalendar';

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

  it('cada celda tiene un aria-label con la fecha completa y el resumen', () => {
    render(<MonthlyCalendar monthStart="2026-12-01" entries={[]} onSelectDay={vi.fn()} />);

    expect(screen.getByTestId('month-day-2026-12-15')).toHaveAttribute(
      'aria-label',
      '15 de diciembre de 2026, —',
    );
  });
});
