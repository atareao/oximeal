import type { MealPlanEntryView } from '../api/types';
import { DayCard } from './DayCard';
import './WeeklyCalendar.css';

export interface WeeklyCalendarProps {
  days: readonly string[];
  entries: readonly MealPlanEntryView[];
  isDesktop: boolean;
}

/** Calendario semanal: una tarjeta por día, en columna en móvil y rejilla en desktop. */
export function WeeklyCalendar({ days, entries, isDesktop }: WeeklyCalendarProps) {
  return (
    <div
      className={`weekly-calendar${isDesktop ? ' weekly-calendar--desktop' : ''}`}
      data-testid="weekly-calendar"
    >
      {days.map((day) => (
        <DayCard key={day} date={day} entries={entries.filter((entry) => entry.date === day)} />
      ))}
    </div>
  );
}
