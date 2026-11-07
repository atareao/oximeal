import { useEffect, useMemo, useState } from 'react';
import {
  Alert,
  App as AntdApp,
  Button,
  Empty,
  Input,
  Modal,
  Result,
  Select,
  Skeleton,
  Typography,
} from 'antd';
import { useSearchParams } from 'react-router-dom';
import { useCreateRecipe, useImportRecipe, useRecipes, useTags } from '../api/hooks';
import { ApiError } from '../api/client';
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

/** Comprueba que la cadena es una URL absoluta `http(s)` válida. */
function isHttpUrl(value: string): boolean {
  try {
    const parsed = new URL(value);
    return parsed.protocol === 'http:' || parsed.protocol === 'https:';
  } catch {
    return false;
  }
}

/**
 * Mensaje amigable para los fallos de importación del backend, según el código
 * de estado, sin filtrar el detalle técnico. Los `400` se muestran dentro del
 * diálogo (ver {@link RecipesPage}); el resto como aviso global.
 */
function describeImportError(error: unknown): string {
  if (error instanceof ApiError) {
    if (error.status === 422) {
      return 'La página no contiene una receta estructurada.';
    }
    if (error.status === 502) {
      return 'No se pudo contactar con el origen. Inténtalo de nuevo.';
    }
  }
  return error instanceof Error && error.message ? error.message : 'No se pudo importar la receta.';
}

/** Catálogo de recetas: listado con búsqueda, paginación y alta. */
export function RecipesPage() {
  const { message } = AntdApp.useApp();
  const online = useOnline();
  const [searchParams, setSearchParams] = useSearchParams();
  const tag = searchParams.get('tag') ?? undefined;
  const [search, setSearch] = useState('');
  const [debouncedSearch, setDebouncedSearch] = useState('');
  const [page, setPage] = useState(1);
  const [formOpen, setFormOpen] = useState(false);
  const [importOpen, setImportOpen] = useState(false);
  const [importUrl, setImportUrl] = useState('');
  const [importError, setImportError] = useState<string | null>(null);

  useEffect(() => {
    const timer = setTimeout(() => setDebouncedSearch(search), SEARCH_DEBOUNCE_MS);
    return () => clearTimeout(timer);
  }, [search]);

  const params = useMemo<RecipeQueryParams>(
    () => ({ q: debouncedSearch.trim() || undefined, tag, page, page_size: PAGE_SIZE }),
    [debouncedSearch, tag, page],
  );

  const { data, isLoading, isError, error, refetch } = useRecipes(params);
  const tagsQuery = useTags();
  const createRecipe = useCreateRecipe();
  const importRecipe = useImportRecipe();

  const tagOptions = (tagsQuery.data ?? []).map((item) => ({
    value: item.slug,
    label: item.name,
  }));

  const items = data?.items ?? [];
  const total = data?.total ?? 0;
  const totalPages = Math.max(1, Math.ceil(total / PAGE_SIZE));

  const errorMessage =
    error instanceof Error ? error.message : 'Ha ocurrido un error al contactar con el servidor.';

  const handleSearch = (value: string) => {
    setSearch(value);
    setPage(1);
  };

  /** Aplica/quita el filtro por etiqueta reflejándolo en la URL (`?tag=`). */
  const applyTag = (slug: string | undefined) => {
    setSearchParams((previous) => {
      const next = new URLSearchParams(previous);
      if (slug) {
        next.set('tag', slug);
      } else {
        next.delete('tag');
      }
      return next;
    });
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

  const openImport = () => {
    setImportUrl('');
    setImportError(null);
    setImportOpen(true);
  };

  const handleImport = () => {
    if (!online) {
      return;
    }
    const url = importUrl.trim();
    if (!isHttpUrl(url)) {
      setImportError('Introduce una URL http(s) válida');
      return;
    }
    setImportError(null);
    importRecipe.mutate(
      { url },
      {
        onSuccess: (result) => {
          message.success(result.created ? 'Receta importada' : 'La receta ya existía');
          setImportOpen(false);
        },
        onError: (mutationError: unknown) => {
          // El 400 (URL/SSRF rechazada) se muestra dentro del diálogo para que
          // el usuario corrija la URL sin perder el contexto; el resto, global.
          if (mutationError instanceof ApiError && mutationError.status === 400) {
            setImportError('Revisa la URL: introduce una página pública http(s).');
            return;
          }
          message.error(describeImportError(mutationError));
        },
      },
    );
  };

  return (
    <section aria-labelledby="recipes-title" className="recipes-page">
      <header className="recipes-page__header">
        <Title id="recipes-title" level={2} className="visually-hidden">
          Recetas
        </Title>
        <div className="recipes-page__actions">
          <Button className="recipes-page__import" disabled={!online} onClick={openImport}>
            Importar desde URL
          </Button>
          <Button
            type="primary"
            className="recipes-page__new"
            disabled={!online}
            onClick={() => setFormOpen(true)}
          >
            Nueva receta
          </Button>
        </div>
        <Select
          className="recipes-page__tag-filter"
          aria-label="Filtrar por etiqueta"
          placeholder="Filtrar por etiqueta"
          allowClear
          disabled={!online}
          value={tag}
          options={tagOptions}
          onChange={(value) => applyTag(value ? String(value) : undefined)}
        />
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
            <RecipeCard key={recipe.id} recipe={recipe} onSelectTag={applyTag} />
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

      <Modal
        open={importOpen}
        title="Importar desde URL"
        okText="Importar"
        cancelText="Cancelar"
        confirmLoading={importRecipe.isPending}
        okButtonProps={{
          disabled: !online,
          className: 'recipes-page__import-confirm',
        }}
        cancelButtonProps={{ className: 'recipes-page__import-cancel' }}
        onOk={handleImport}
        onCancel={() => setImportOpen(false)}
      >
        {!online && (
          <Alert
            type="warning"
            showIcon
            data-testid="recipes-import-offline"
            className="recipes-page__import-offline"
            message="Sin conexión: no puedes importar recetas."
          />
        )}
        <form
          onSubmit={(event) => {
            event.preventDefault();
            handleImport();
          }}
        >
          <label htmlFor="recipe-import-url" className="recipes-page__import-label">
            URL de la receta
          </label>
          <Input
            id="recipe-import-url"
            type="url"
            placeholder="https://…"
            value={importUrl}
            status={importError ? 'error' : undefined}
            onChange={(event) => {
              setImportUrl(event.target.value);
              if (importError) {
                setImportError(null);
              }
            }}
          />
          {importError && (
            <p role="alert" className="recipes-page__import-error">
              {importError}
            </p>
          )}
        </form>
      </Modal>
    </section>
  );
}
