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
  removingId?: string | null;
  onPrevious: () => void;
  onNext: () => void;
  onToday: () => void;
}

/** Vista diaria: un único día con todas sus franjas y navegación entre días. */
export function DailyView({
  date,
  entries,
  online,
  onAdd,
  onRemove,
  onEditServings,
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
        slots={['breakfast', 'lunch', 'dinner', 'snack']}
        online={online}
        onAdd={onAdd}
        onRemove={onRemove}
        onEditServings={onEditServings}
        removingId={removingId}
      />
    </div>
  );
}
