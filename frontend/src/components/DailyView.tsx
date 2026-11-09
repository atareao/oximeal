import type { MealPlanEntryView, MealType } from '../api/types';
import { DayCard } from './DayCard';
import './DailyView.css';

export interface DailyViewProps {
  date: string;
  entries: readonly MealPlanEntryView[];
  online?: boolean;
  onAdd?: (date: string, mealType: MealType) => void;
  onRemove?: (entry: MealPlanEntryView) => void;
  onEditServings?: (entry: MealPlanEntryView) => void;
  onAddToShoppingList?: (date: string) => void;
  addingToShoppingList?: boolean;
  addingToShoppingListDate?: string | null;
  removingId?: string | null;
  onPrevious: () => void;
  onNext: () => void;
  onToday: () => void;
}

/** Vista diaria: un único día con COMIDA y CENA y navegación entre días. */
export function DailyView({
  date,
  entries,
  online,
  onAdd,
  onRemove,
  onEditServings,
  onAddToShoppingList,
  addingToShoppingList,
  addingToShoppingListDate,
  removingId,
  onPrevious,
  onNext,
  onToday,
}: DailyViewProps) {
  return (
    <div className="daily-view" data-testid="daily-view">
      <div className="daily-view__nav">
        <button type="button" className="daily-view__button" onClick={onPrevious}>
          ‹ Día anterior
        </button>
        <button type="button" className="daily-view__button" onClick={onToday}>
          Hoy
        </button>
        <button type="button" className="daily-view__button" onClick={onNext}>
          Día siguiente ›
        </button>
      </div>
      <DayCard
        date={date}
        entries={entries}
        online={online}
        onAdd={onAdd}
        onRemove={onRemove}
        onEditServings={onEditServings}
        onAddToShoppingList={onAddToShoppingList}
        addingToShoppingList={addingToShoppingList}
        addingToShoppingListDate={addingToShoppingListDate}
        removingId={removingId}
      />
    </div>
  );
}
