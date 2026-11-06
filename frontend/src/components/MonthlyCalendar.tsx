import type { MealPlanEntryView } from '../api/types';
import { parseDateString, toDateString } from '../lib/dates';
import './MonthlyCalendar.css';

export interface MonthlyCalendarProps {
  /** Primer día del mes visible (`YYYY-MM-01`). */
  monthStart: string;
  entries: readonly MealPlanEntryView[];
  onSelectDay: (date: string) => void;
}

const WEEKDAY_LABELS = ['L', 'M', 'X', 'J', 'V', 'S', 'D'] as const;

function daysInMonth(monthStart: string): string[] {
  const base = parseDateString(monthStart);
  const year = base.getFullYear();
  const month = base.getMonth();
  const total = new Date(year, month + 1, 0).getDate();
  return Array.from({ length: total }, (_, index) =>
    toDateString(new Date(year, month, index + 1)),
  );
}

/** Rejilla mensual: una celda por día con el resumen de comidas planificadas. */
export function MonthlyCalendar({ monthStart, entries, onSelectDay }: MonthlyCalendarProps) {
  const days = daysInMonth(monthStart);

  return (
    <div className="monthly-calendar" data-testid="monthly-calendar">
      <div className="monthly-calendar__weekdays" aria-hidden="true">
        {WEEKDAY_LABELS.map((label) => (
          <span key={label} className="monthly-calendar__weekday">
            {label}
          </span>
        ))}
      </div>
      <div className="monthly-calendar__grid">
        {days.map((date) => {
          const count = entries.filter((entry) => entry.date === date).length;
          const dayNumber = parseDateString(date).getDate();
          return (
            <button
              key={date}
              type="button"
              className="monthly-calendar__day"
              data-testid={`month-day-${date}`}
              onClick={() => onSelectDay(date)}
            >
              <span className="monthly-calendar__day-number">{dayNumber}</span>
              <span className="monthly-calendar__day-summary">
                {count === 0 ? '—' : `${count} ${count === 1 ? 'comida' : 'comidas'}`}
              </span>
            </button>
          );
        })}
      </div>
    </div>
  );
}
