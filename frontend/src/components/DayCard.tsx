import { useId } from 'react';
import { Button, Popconfirm } from 'antd';
import type { MealPlanEntryView, MealType } from '../api/types';
import { formatDayLabel, isToday } from '../lib/dates';
import { mealTypeLabel } from '../lib/mealPlan';
import './DayCard.css';

export interface DayCardProps {
  date: string;
  entries: readonly MealPlanEntryView[];
  /** Habilita los controles de planificación (requieren conexión). */
  online?: boolean;
  onAdd?: (date: string, mealType: MealType) => void;
  onRemove?: (entry: MealPlanEntryView) => void;
  onEditServings?: (entry: MealPlanEntryView) => void;
  /** Id de la entrada cuya baja está en curso, para el estado de carga. */
  removingId?: string | null;
}

interface MealSlotProps {
  label: string;
  mealType: MealType;
  date: string;
  entries: readonly MealPlanEntryView[];
  online: boolean;
  onAdd?: (date: string, mealType: MealType) => void;
  onRemove?: (entry: MealPlanEntryView) => void;
  onEditServings?: (entry: MealPlanEntryView) => void;
  removingId?: string | null;
}

function MealSlot({
  label,
  mealType,
  date,
  entries,
  online,
  onAdd,
  onRemove,
  onEditServings,
  removingId,
}: MealSlotProps) {
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
                <span className="day-card__meal-servings">{entry.servings} rac.</span>
                <span className="day-card__meal-actions">
                  <Button
                    size="small"
                    className="day-card__edit"
                    disabled={!online}
                    aria-label={`Editar raciones de ${entry.recipe.name}`}
                    onClick={() => onEditServings?.(entry)}
                  >
                    Raciones
                  </Button>
                  <Popconfirm
                    title="¿Quitar esta receta del menú?"
                    okText="Quitar"
                    cancelText="Cancelar"
                    okButtonProps={{ danger: true }}
                    disabled={!online}
                    onConfirm={() => onRemove?.(entry)}
                  >
                    <Button
                      size="small"
                      danger
                      className="day-card__remove"
                      disabled={!online}
                      loading={removingId === entry.id}
                      aria-label={`Quitar ${entry.recipe.name}`}
                    >
                      Quitar
                    </Button>
                  </Popconfirm>
                </span>
              </li>
            );
          })}
        </ul>
      )}
      <Button
        size="small"
        className="day-card__add"
        disabled={!online}
        aria-label={`Añadir receta a ${label}`}
        onClick={() => onAdd?.(date, mealType)}
      >
        Añadir
      </Button>
    </div>
  );
}

/** Tarjeta de un día con las franjas COMIDA y CENA y acciones de planificación. */
export function DayCard({
  date,
  entries,
  online = true,
  onAdd,
  onRemove,
  onEditServings,
  removingId,
}: DayCardProps) {
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
      <MealSlot
        label={mealTypeLabel('lunch')}
        mealType="lunch"
        date={date}
        entries={lunch}
        online={online}
        onAdd={onAdd}
        onRemove={onRemove}
        onEditServings={onEditServings}
        removingId={removingId}
      />
      <MealSlot
        label={mealTypeLabel('dinner')}
        mealType="dinner"
        date={date}
        entries={dinner}
        online={online}
        onAdd={onAdd}
        onRemove={onRemove}
        onEditServings={onEditServings}
        removingId={removingId}
      />
    </article>
  );
}
