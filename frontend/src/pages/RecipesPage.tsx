import { useEffect, useMemo, useState } from 'react';
import { Alert, App as AntdApp, Button, Empty, Input, Result, Skeleton, Typography } from 'antd';
import { useCreateRecipe, useRecipes } from '../api/hooks';
import type { RecipeInput, RecipeQueryParams } from '../api/types';
import { RecipeCard } from '../components/RecipeCard';
import { RecipeForm } from '../components/RecipeForm';
import { useOnline } from '../offline/connectivity';
import './RecipesPage.css';

const { Title } = Typography;

/** Tamaño de página del catálogo (coincide con el valor por defecto del backend). */
const PAGE_SIZE = 20;

/** Retardo del debounce de búsqueda: evita un refetch por cada tecla. */
const SEARCH_DEBOUNCE_MS = 300;

/** Catálogo de recetas: listado con búsqueda, paginación y alta. */
export function RecipesPage() {
  const { message } = AntdApp.useApp();
  const online = useOnline();
  const [search, setSearch] = useState('');
  const [debouncedSearch, setDebouncedSearch] = useState('');
  const [page, setPage] = useState(1);
  const [formOpen, setFormOpen] = useState(false);

  useEffect(() => {
    const timer = setTimeout(() => setDebouncedSearch(search), SEARCH_DEBOUNCE_MS);
    return () => clearTimeout(timer);
  }, [search]);

  const params = useMemo<RecipeQueryParams>(
    () => ({ q: debouncedSearch.trim() || undefined, page, page_size: PAGE_SIZE }),
    [debouncedSearch, page],
  );

  const { data, isLoading, isError, error, refetch } = useRecipes(params);
  const createRecipe = useCreateRecipe();

  const items = data?.items ?? [];
  const total = data?.total ?? 0;
  const totalPages = Math.max(1, Math.ceil(total / PAGE_SIZE));

  const errorMessage =
    error instanceof Error ? error.message : 'Ha ocurrido un error al contactar con el servidor.';

  const handleSearch = (value: string) => {
    setSearch(value);
    setPage(1);
  };

  const handleCreate = (input: RecipeInput) => {
    createRecipe.mutate(input, {
      onSuccess: () => {
        message.success('Receta creada');
        setFormOpen(false);
      },
      onError: (mutationError: unknown) => {
        message.error(
          mutationError instanceof Error ? mutationError.message : 'No se pudo crear la receta',
        );
      },
    });
  };

  return (
    <section aria-labelledby="recipes-title" className="recipes-page">
      <header className="recipes-page__header">
        <Title id="recipes-title" level={2} className="recipes-page__title">
          Recetas
        </Title>
        <Button
          type="primary"
          className="recipes-page__new"
          disabled={!online}
          onClick={() => setFormOpen(true)}
        >
          Nueva receta
        </Button>
      </header>

      {!online && (
        <Alert
          type="warning"
          showIcon
          data-testid="recipes-offline"
          message="Sin conexión: no puedes crear, editar ni borrar recetas."
        />
      )}

      <Input
        type="search"
        aria-label="Buscar recetas"
        placeholder="Buscar recetas…"
        value={search}
        onChange={(event) => handleSearch(event.target.value)}
        allowClear
        className="recipes-page__search"
      />

      {isLoading && (
        <div className="recipes-page__loading" data-testid="recipes-loading">
          <Skeleton active paragraph={{ rows: 5 }} />
        </div>
      )}

      {!isLoading && isError && !data && (
        <Result
          status="error"
          title="No se pudieron cargar las recetas"
          subTitle={errorMessage}
          extra={
            <Button type="primary" onClick={() => void refetch()}>
              Reintentar
            </Button>
          }
        />
      )}

      {!isLoading && !isError && items.length === 0 && (
        <Empty description="No hay recetas" image={Empty.PRESENTED_IMAGE_SIMPLE} />
      )}

      {!isLoading && items.length > 0 && (
        <ul className="recipes-page__list">
          {items.map((recipe) => (
            <RecipeCard key={recipe.id} recipe={recipe} />
          ))}
        </ul>
      )}

      {!isLoading && !isError && total > PAGE_SIZE && (
        <nav className="recipes-page__pagination" aria-label="Paginación de recetas">
          <Button
            className="recipes-page__page-button"
            disabled={page <= 1}
            onClick={() => setPage((current) => Math.max(1, current - 1))}
          >
            Anterior
          </Button>
          <span className="recipes-page__page-info">
            Página {page} de {totalPages}
          </span>
          <Button
            className="recipes-page__page-button"
            disabled={page >= totalPages}
            onClick={() => setPage((current) => current + 1)}
          >
            Siguiente
          </Button>
        </nav>
      )}

      <RecipeForm
        open={formOpen}
        initial={null}
        submitting={createRecipe.isPending}
        onClose={() => setFormOpen(false)}
        onSubmit={handleCreate}
      />
    </section>
  );
}
