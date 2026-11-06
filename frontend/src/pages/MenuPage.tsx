import { useEffect, useMemo, useRef, useState } from 'react';
import { Link, useSearchParams } from 'react-router-dom';
import { Alert, Button, Empty, Result, Segmented, Skeleton, Typography } from 'antd';
import {
  useAddMealPlanEntry,
  useMealPlan,
  useRemoveMealPlanEntry,
  useUpdateMealPlanEntry,
} from '../api/hooks';
import type { MealPlanEntryView, MealPlanInput, MealType } from '../api/types';
import { AddMealPlanDialog } from '../components/AddMealPlanDialog';
import { DailyView } from '../components/DailyView';
import { EditServingsDialog } from '../components/EditServingsDialog';
import { MonthlyCalendar } from '../components/MonthlyCalendar';
import { WeeklyCalendar } from '../components/WeeklyCalendar';
import { useMediaQuery } from '../hooks/useMediaQuery';
import { useOnline } from '../offline/connectivity';
import { hasValet, VALET_URL } from '../lib/config';
import {
  addDays,
  formatWeekRange,
  getWeekDays,
  isValidDateString,
  parseDateString,
  startOfWeek,
  toDateString,
} from '../lib/dates';
import './MenuPage.css';

const { Title, Paragraph } = Typography;

const DESKTOP_QUERY = '(min-width: 768px)';

type MenuView = 'day' | 'week' | 'month';

const VIEW_OPTIONS: { label: string; value: MenuView }[] = [
  { label: 'Diaria', value: 'day' },
  { label: 'Semanal', value: 'week' },
  { label: 'Mensual', value: 'month' },
];

interface AddSlot {
  date: string;
  /** `undefined` cuando se planifica fuera de un slot (estado vacío). */
  mealType?: MealType;
}

function resolveView(value: string | null): MenuView {
  return value === 'day' || value === 'month' ? value : 'week';
}

function resolveWeekParam(value: string | null): string {
  if (value && isValidDateString(value)) {
    return startOfWeek(value);
  }
  return startOfWeek(toDateString(new Date()));
}

function resolveDateParam(value: string | null): string {
  if (value && isValidDateString(value)) {
    return value;
  }
  return toDateString(new Date());
}

function monthStartOf(date: string): string {
  return `${date.slice(0, 7)}-01`;
}

function formatMonthLabel(monthStart: string): string {
  return new Intl.DateTimeFormat('es-ES', { month: 'long', year: 'numeric' }).format(
    parseDateString(monthStart),
  );
}

function formatDayLabel(date: string): string {
  return new Intl.DateTimeFormat('es-ES', {
    weekday: 'long',
    day: 'numeric',
    month: 'long',
  }).format(parseDateString(date));
}

function monthEndOf(monthStart: string): string {
  const base = parseDateString(monthStart);
  return toDateString(new Date(base.getFullYear(), base.getMonth() + 1, 0));
}

function shiftMonth(monthStart: string, delta: number): string {
  const base = parseDateString(monthStart);
  return toDateString(new Date(base.getFullYear(), base.getMonth() + delta, 1));
}

function describeActionError(error: unknown, fallback: string): string {
  return error instanceof Error && error.message ? error.message : fallback;
}

/** Vista del menú editable: diaria, semanal o mensual, con alta, baja y edición. */
export function MenuPage() {
  const [searchParams, setSearchParams] = useSearchParams();
  const isDesktop = useMediaQuery(DESKTOP_QUERY);
  const online = useOnline();

  const view = resolveView(searchParams.get('view'));
  const referenceDate =
    view === 'week'
      ? resolveWeekParam(searchParams.get('week'))
      : resolveDateParam(searchParams.get('date'));

  const weekStart = view === 'week' ? referenceDate : startOfWeek(referenceDate);
  const weekEnd = addDays(weekStart, 6);
  const days = useMemo(() => getWeekDays(weekStart), [weekStart]);

  const monthStart = monthStartOf(referenceDate);
  const monthEnd = monthEndOf(monthStart);

  const rangeStart = view === 'day' ? referenceDate : view === 'week' ? weekStart : monthStart;
  const rangeEnd = view === 'day' ? referenceDate : view === 'week' ? weekEnd : monthEnd;

  const { data, isLoading, isError, error, refetch } = useMealPlan(rangeStart, rangeEnd);

  // Al recuperar la conexión, refresca el plan desde el servidor
  // (scenario «Recuperación de la conexión» de offline-writes).
  const wasOffline = useRef(!online);
  useEffect(() => {
    if (online && wasOffline.current) {
      void refetch();
    }
    wasOffline.current = !online;
  }, [online, refetch]);

  const addEntry = useAddMealPlanEntry();
  const removeEntry = useRemoveMealPlanEntry();
  const updateEntry = useUpdateMealPlanEntry();

  const [addSlot, setAddSlot] = useState<AddSlot | null>(null);
  const [editEntry, setEditEntry] = useState<MealPlanEntryView | null>(null);
  const [addError, setAddError] = useState<string | null>(null);
  const [editError, setEditError] = useState<string | null>(null);
  const [removeError, setRemoveError] = useState<string | null>(null);

  const errorMessage =
    error instanceof Error ? error.message : 'Ha ocurrido un error al contactar con el servidor.';

  const changeView = (next: MenuView) => {
    if (next === view) {
      return;
    }
    if (next === 'week') {
      setSearchParams({ view: 'week', week: startOfWeek(referenceDate) });
    } else if (next === 'day') {
      let date = referenceDate;
      if (view === 'week') {
        const today = toDateString(new Date());
        date = today >= weekStart && today <= weekEnd ? today : weekStart;
      }
      setSearchParams({ view: 'day', date });
    } else {
      setSearchParams({ view: 'month', date: monthStartOf(referenceDate) });
    }
  };

  const goToWeek = (start: string) => {
    setSearchParams({ view: 'week', week: start });
  };

  const goToDay = (date: string) => {
    setSearchParams({ view: 'day', date });
  };

  const goToMonth = (start: string) => {
    setSearchParams({ view: 'month', date: start });
  };

  const openAdd = (date: string, mealType?: MealType) => {
    setAddError(null);
    setAddSlot({ date, mealType });
  };

  const handleAddSubmit = (input: MealPlanInput) => {
    addEntry.mutate(input, {
      onSuccess: () => {
        setAddSlot(null);
      },
      onError: () => {
        setAddError('No se pudo añadir la receta. Inténtalo de nuevo.');
      },
    });
  };

  const handleRemove = (entry: MealPlanEntryView) => {
    setRemoveError(null);
    removeEntry.mutate(entry.id, {
      onError: (mutationError: unknown) => {
        setRemoveError(
          describeActionError(mutationError, 'No se pudo quitar la receta. Inténtalo de nuevo.'),
        );
      },
    });
  };

  const openEditServings = (entry: MealPlanEntryView) => {
    setEditError(null);
    setEditEntry(entry);
  };

  const handleEditSubmit = (servings: number) => {
    if (!editEntry) {
      return;
    }
    updateEntry.mutate(
      { id: editEntry.id, input: { servings } },
      {
        onSuccess: () => {
          setEditEntry(null);
        },
        onError: () => {
          setEditError('No se pudieron actualizar las raciones. Inténtalo de nuevo.');
        },
      },
    );
  };

  const rangeLabel =
    view === 'week'
      ? formatWeekRange(weekStart, weekEnd)
      : view === 'month'
        ? formatMonthLabel(monthStart)
        : formatDayLabel(referenceDate);

  const header = (
    <header className="menu-page__header">
      <div className="menu-page__heading">
        <Title id="menu-title" level={2} className="menu-page__title">
          Menú
        </Title>
        <Paragraph className="menu-page__range" type="secondary">
          {rangeLabel}
        </Paragraph>
      </div>
      <Segmented
        className="menu-page__view"
        options={VIEW_OPTIONS}
        value={view}
        onChange={(value) => changeView(value as MenuView)}
      />
      <div className="menu-page__nav">
        {view === 'week' && (
          <>
            <Button aria-label="Semana anterior" onClick={() => goToWeek(addDays(weekStart, -7))}>
              ‹
            </Button>
            <Button onClick={() => goToWeek(startOfWeek(toDateString(new Date())))}>Hoy</Button>
            <Button aria-label="Semana siguiente" onClick={() => goToWeek(addDays(weekStart, 7))}>
              ›
            </Button>
          </>
        )}
        {view === 'month' && (
          <>
            <Button aria-label="Mes anterior" onClick={() => goToMonth(shiftMonth(monthStart, -1))}>
              ‹
            </Button>
            <Button aria-label="Mes siguiente" onClick={() => goToMonth(shiftMonth(monthStart, 1))}>
              ›
            </Button>
          </>
        )}
        {hasValet && (
          <Button href={VALET_URL} target="_blank" rel="noopener noreferrer">
            Añadir comidas con Valet
          </Button>
        )}
      </div>
    </header>
  );

  const emptyState = (
    <Empty description="No hay comidas planificadas" image={Empty.PRESENTED_IMAGE_SIMPLE}>
      <div className="menu-page__empty-actions">
        <Paragraph type="secondary" className="menu-page__empty-hint">
          Planifica aquí mismo una receta del recetario o añade comidas con Valet.
        </Paragraph>
        <Button
          type="primary"
          className="menu-page__empty-plan"
          disabled={!online}
          onClick={() => openAdd(referenceDate)}
        >
          Planificar una comida
        </Button>
        <Link to="/shopping">Ir a la lista de la compra</Link>
      </div>
    </Empty>
  );

  return (
    <section aria-labelledby="menu-title" className="menu-page">
      {header}

      {!online && (
        <Alert
          type="warning"
          showIcon
          data-testid="menu-offline"
          message="Sin conexión: la planificación de comidas requiere conexión."
        />
      )}

      {removeError && (
        <Alert
          type="error"
          showIcon
          data-testid="menu-remove-error"
          message={removeError}
          closable
          onClose={() => setRemoveError(null)}
        />
      )}

      {isLoading && (
        <div className="menu-page__loading" data-testid="menu-loading">
          <Skeleton active paragraph={{ rows: 6 }} />
        </div>
      )}

      {!isLoading && isError && !data && (
        <Result
          status="error"
          title="No se pudo cargar el menú"
          subTitle={errorMessage}
          extra={
            <Button type="primary" onClick={() => void refetch()}>
              Reintentar
            </Button>
          }
        />
      )}

      {!isLoading && isError && data && (
        <Alert
          type="error"
          showIcon
          message="No se pudo actualizar el menú"
          description={errorMessage}
          action={
            <Button size="small" onClick={() => void refetch()}>
              Reintentar
            </Button>
          }
        />
      )}

      {!isLoading &&
        !isError &&
        data &&
        view === 'week' &&
        (data.length === 0 ? (
          emptyState
        ) : (
          <WeeklyCalendar
            days={days}
            entries={data}
            isDesktop={isDesktop}
            online={online}
            onAdd={(date, mealType) => openAdd(date, mealType)}
            onRemove={handleRemove}
            onEditServings={openEditServings}
            removingId={removeEntry.isPending ? removeEntry.variables : null}
          />
        ))}

      {!isLoading && !isError && data && view === 'day' && (
        <DailyView
          date={referenceDate}
          entries={data}
          online={online}
          onAdd={(date, mealType) => openAdd(date, mealType)}
          onRemove={handleRemove}
          onEditServings={openEditServings}
          removingId={removeEntry.isPending ? removeEntry.variables : null}
          onPrevious={() => goToDay(addDays(referenceDate, -1))}
          onNext={() => goToDay(addDays(referenceDate, 1))}
          onToday={() => goToDay(toDateString(new Date()))}
        />
      )}

      {!isLoading && !isError && data && view === 'month' && (
        <MonthlyCalendar
          monthStart={monthStart}
          entries={data}
          onSelectDay={(date) => goToDay(date)}
        />
      )}

      <AddMealPlanDialog
        open={addSlot !== null}
        date={addSlot?.date ?? referenceDate}
        mealType={addSlot?.mealType}
        online={online}
        submitting={addEntry.isPending}
        error={addError}
        onClose={() => setAddSlot(null)}
        onSubmit={handleAddSubmit}
      />

      <EditServingsDialog
        open={editEntry !== null}
        entry={editEntry}
        submitting={updateEntry.isPending}
        error={editError}
        onClose={() => setEditEntry(null)}
        onSubmit={handleEditSubmit}
      />
    </section>
  );
}
