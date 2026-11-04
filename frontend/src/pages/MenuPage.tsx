import { useEffect, useMemo, useRef, useState } from 'react';
import { Link, useSearchParams } from 'react-router-dom';
import { Alert, Button, Empty, Result, Skeleton, Typography } from 'antd';
import {
  useAddMealPlanEntry,
  useMealPlan,
  useRemoveMealPlanEntry,
  useUpdateMealPlanEntry,
} from '../api/hooks';
import type { MealPlanEntryView, MealPlanInput, MealType } from '../api/types';
import { AddMealPlanDialog } from '../components/AddMealPlanDialog';
import { EditServingsDialog } from '../components/EditServingsDialog';
import { WeeklyCalendar } from '../components/WeeklyCalendar';
import { useMediaQuery } from '../hooks/useMediaQuery';
import { useOnline } from '../offline/connectivity';
import { hasValet, VALET_URL } from '../lib/config';
import {
  addDays,
  formatWeekRange,
  getWeekDays,
  isValidDateString,
  startOfWeek,
  toDateString,
} from '../lib/dates';
import './MenuPage.css';

const { Title, Paragraph } = Typography;

const DESKTOP_QUERY = '(min-width: 768px)';

interface AddSlot {
  date: string;
  /** `undefined` cuando se planifica fuera de un slot (estado vacío). */
  mealType?: MealType;
}

function resolveWeekParam(value: string | null): string {
  if (value && isValidDateString(value)) {
    return startOfWeek(value);
  }
  return startOfWeek(toDateString(new Date()));
}

function describeActionError(error: unknown, fallback: string): string {
  return error instanceof Error && error.message ? error.message : fallback;
}

/** Vista del menú semanal editable: alta en slot, baja y edición de raciones. */
export function MenuPage() {
  const [searchParams, setSearchParams] = useSearchParams();
  const isDesktop = useMediaQuery(DESKTOP_QUERY);
  const online = useOnline();

  const weekStart = resolveWeekParam(searchParams.get('week'));
  const weekEnd = addDays(weekStart, 6);
  const days = useMemo(() => getWeekDays(weekStart), [weekStart]);

  const { data, isLoading, isError, error, refetch } = useMealPlan(weekStart, weekEnd);

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

  const goToWeek = (start: string) => {
    setSearchParams({ week: start });
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

  const header = (
    <header className="menu-page__header">
      <div className="menu-page__heading">
        <Title id="menu-title" level={2} className="menu-page__title">
          Menú semanal
        </Title>
        <Paragraph className="menu-page__range" type="secondary">
          {formatWeekRange(weekStart, weekEnd)}
        </Paragraph>
      </div>
      <div className="menu-page__nav">
        <Button aria-label="Semana anterior" onClick={() => goToWeek(addDays(weekStart, -7))}>
          ‹
        </Button>
        <Button onClick={() => goToWeek(startOfWeek(toDateString(new Date())))}>Hoy</Button>
        <Button aria-label="Semana siguiente" onClick={() => goToWeek(addDays(weekStart, 7))}>
          ›
        </Button>
        {hasValet && (
          <Button href={VALET_URL} target="_blank" rel="noopener noreferrer">
            Añadir comidas con Valet
          </Button>
        )}
      </div>
    </header>
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

      {!isLoading && !isError && data && data.length === 0 && (
        <Empty
          description="No hay comidas planificadas esta semana"
          image={Empty.PRESENTED_IMAGE_SIMPLE}
        >
          <div className="menu-page__empty-actions">
            <Paragraph type="secondary" className="menu-page__empty-hint">
              Planifica aquí mismo una receta del recetario o añade comidas con Valet.
            </Paragraph>
            <Button
              type="primary"
              className="menu-page__empty-plan"
              disabled={!online}
              onClick={() => openAdd(weekStart)}
            >
              Planificar una comida
            </Button>
            <Link to="/shopping">Ir a la lista de la compra</Link>
          </div>
        </Empty>
      )}

      {!isLoading && !isError && data && data.length > 0 && (
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
      )}

      <AddMealPlanDialog
        open={addSlot !== null}
        date={addSlot?.date ?? weekStart}
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
