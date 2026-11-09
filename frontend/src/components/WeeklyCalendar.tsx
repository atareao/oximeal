import type { MealPlanEntryView, MealType } from '../api/types';
import { DayCard } from './DayCard';
import './WeeklyCalendar.css';

export interface WeeklyCalendarProps {
  days: readonly string[];
  entries: readonly MealPlanEntryView[];
  isDesktop: boolean;
  online?: boolean;
  onAdd?: (date: string, mealType: MealType) => void;
  onRemove?: (entry: MealPlanEntryView) => void;
  onEditServings?: (entry: MealPlanEntryView) => void;
  onAddToShoppingList?: (date: string) => void;
  addingToShoppingList?: boolean;
  addingToShoppingListDate?: string | null;
  removingId?: string | null;
}

/** Calendario semanal: una tarjeta por día, en columna en móvil y rejilla en desktop. */
export function WeeklyCalendar({
  days,
  entries,
  isDesktop,
  online,
  onAdd,
  onRemove,
  onEditServings,
  onAddToShoppingList,
  addingToShoppingList,
  addingToShoppingListDate,
  removingId,
}: WeeklyCalendarProps) {
  return (
    <div
      className={`weekly-calendar${isDesktop ? ' weekly-calendar--desktop' : ''}`}
      data-testid="weekly-calendar"
    >
      {days.map((day) => (
        <DayCard
          key={day}
          date={day}
          entries={entries.filter((entry) => entry.date === day)}
          online={online}
          onAdd={onAdd}
          onRemove={onRemove}
          onEditServings={onEditServings}
          onAddToShoppingList={onAddToShoppingList}
          addingToShoppingList={addingToShoppingList}
          addingToShoppingListDate={addingToShoppingListDate}
          removingId={removingId}
        />
      ))}
    </div>
  );
}
