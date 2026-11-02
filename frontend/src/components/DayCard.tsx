import { useId } from 'react';
import type { MealPlanEntryView } from '../api/types';
import { formatDayLabel, isToday } from '../lib/dates';
import './DayCard.css';

export interface DayCardProps {
  date: string;
  entries: readonly MealPlanEntryView[];
}

interface MealSlotProps {
  label: string;
  entries: readonly MealPlanEntryView[];
}

function MealSlot({ label, entries }: MealSlotProps) {
  return (
    <div className="day-card__slot">
      <h3 className="day-card__slot-label">{label}</h3>
      {entries.length === 0 ? (
        <p className="day-card__empty">Sin planificar</p>
      ) : (
        <ul className="day-card__meals">
          {entries.map((entry) => {
            const isSalad = entry.role === 'salad' || entry.recipe.category === 'salad';
            return (
              <li key={entry.id} className="day-card__meal">
                <span className="day-card__meal-name">{entry.recipe.name}</span>
                {isSalad && <span className="day-card__salad">🥗 Ensalada</span>}
              </li>
            );
          })}
        </ul>
      )}
    </div>
  );
}

/** Tarjeta de un día con las franjas COMIDA y CENA. */
export function DayCard({ date, entries }: DayCardProps) {
  const headingId = useId();
  const today = isToday(date);
  const lunch = entries.filter((entry) => entry.meal_type === 'lunch');
  const dinner = entries.filter((entry) => entry.meal_type === 'dinner');

  return (
    <article
      className={`day-card${today ? ' day-card--today' : ''}`}
      data-testid="day-card"
      aria-labelledby={headingId}
    >
      <header className="day-card__header">
        <span id={headingId} className="day-card__day">
          {formatDayLabel(date)}
        </span>
        {today && <span className="day-card__today-badge">Hoy</span>}
      </header>
      <MealSlot label="COMIDA" entries={lunch} />
      <MealSlot label="CENA" entries={dinner} />
    </article>
  );
}
