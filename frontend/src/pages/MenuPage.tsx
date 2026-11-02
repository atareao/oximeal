import { useMemo } from 'react';
import { Link, useSearchParams } from 'react-router-dom';
import { Alert, Button, Empty, Result, Skeleton, Typography } from 'antd';
import { useMealPlan } from '../api/hooks';
import { WeeklyCalendar } from '../components/WeeklyCalendar';
import { useMediaQuery } from '../hooks/useMediaQuery';
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

function resolveWeekParam(value: string | null): string {
  if (value && isValidDateString(value)) {
    return startOfWeek(value);
  }
  return startOfWeek(toDateString(new Date()));
}

/** Vista del menú semanal con navegación por semanas en la URL. */
export function MenuPage() {
  const [searchParams, setSearchParams] = useSearchParams();
  const isDesktop = useMediaQuery(DESKTOP_QUERY);

  const weekStart = resolveWeekParam(searchParams.get('week'));
  const weekEnd = addDays(weekStart, 6);
  const days = useMemo(() => getWeekDays(weekStart), [weekStart]);

  const { data, isLoading, isError, error, refetch } = useMealPlan(weekStart, weekEnd);

  const errorMessage =
    error instanceof Error
      ? error.message
      : 'Ha ocurrido un error al contactar con el servidor.';

  const goToWeek = (start: string) => {
    setSearchParams({ week: start });
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
      </div>
    </header>
  );

  return (
    <section aria-labelledby="menu-title" className="menu-page">
      {header}

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
              La planificación de comidas se hace desde el asistente Valet.
            </Paragraph>
            {hasValet && (
              <Button
                type="primary"
                href={VALET_URL}
                target="_blank"
                rel="noopener noreferrer"
              >
                Añadir comidas con Valet
              </Button>
            )}
            <Link to="/shopping">Ir a la lista de la compra</Link>
          </div>
        </Empty>
      )}

      {!isLoading && !isError && data && data.length > 0 && (
        <WeeklyCalendar days={days} entries={data} isDesktop={isDesktop} />
      )}
    </section>
  );
}
