import type { MealPlanEntryView } from '../api/types';
import { parseDateString, toDateString } from '../lib/dates';
import { PLANNED_MEAL_TYPES } from '../lib/mealPlan';
import './MonthlyCalendar.css';

export interface MonthlyCalendarProps {
  /** Primer día del mes visible (`YYYY-MM-01`). */
  monthStart: string;
  entries: readonly MealPlanEntryView[];
  onSelectDay: (date: string) => void;
}

const WEEKDAY_LABELS = ['L', 'M', 'X', 'J', 'V', 'S', 'D'] as const;

const LONG_DATE_FORMAT = new Intl.DateTimeFormat('es-ES', {
  day: 'numeric',
  month: 'long',
  year: 'numeric',
});

function formatLongDate(date: string): string {
  return LONG_DATE_FORMAT.format(parseDateString(date));
}

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
  // getDay(): 0 = domingo … 6 = sábado. Desplazamos para que lunes = 0.
  const leadingBlanks = (parseDateString(monthStart).getDay() + 6) % 7;
  const trailingBlanks = (7 - ((leadingBlanks + days.length) % 7)) % 7;

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
        {Array.from({ length: leadingBlanks }, (_, index) => (
          <span
            key={`blank-start-${index}`}
            aria-hidden="true"
            className="monthly-calendar__blank"
          />
        ))}
        {days.map((date) => {
          const count = entries.filter(
            (entry) => entry.date === date && PLANNED_MEAL_TYPES.includes(entry.meal_type),
          ).length;
          const dayNumber = parseDateString(date).getDate();
          const summary = count === 0 ? '—' : `${count} ${count === 1 ? 'comida' : 'comidas'}`;
          return (
            <button
              key={date}
              type="button"
              className="monthly-calendar__day"
              data-testid={`month-day-${date}`}
              aria-label={`${formatLongDate(date)}, ${summary}`}
              onClick={() => onSelectDay(date)}
            >
              <span className="monthly-calendar__day-number">{dayNumber}</span>
              <span className="monthly-calendar__day-summary">{summary}</span>
            </button>
          );
        })}
        {Array.from({ length: trailingBlanks }, (_, index) => (
          <span key={`blank-end-${index}`} aria-hidden="true" className="monthly-calendar__blank" />
        ))}
      </div>
    </div>
  );
}
