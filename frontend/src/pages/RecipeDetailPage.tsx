import { useState } from 'react';
import { Link, useNavigate, useParams } from 'react-router-dom';
import {
  Alert,
  App as AntdApp,
  Button,
  Checkbox,
  InputNumber,
  Popconfirm,
  Result,
  Skeleton,
  Tabs,
  Tag,
  Typography,
} from 'antd';
import {
  useAddRecipeToShoppingList,
  useDeleteRecipe,
  useRecipe,
  useSettings,
  useUpdateRecipe,
} from '../api/hooks';
import { ApiError } from '../api/client';
import type { IngredientInput, RecipeInput } from '../api/types';
import { IngredientEditor } from '../components/IngredientEditor';
import { RecipeForm } from '../components/RecipeForm';
import { hasRealQuantities } from '../lib/scaling';
import { categoryLabel, formatRecipeMeta } from '../lib/recipes';
import { useOnline } from '../offline/connectivity';
import './RecipeDetailPage.css';

const { Title, Paragraph } = Typography;

const MIN_SERVINGS = 1;
const MAX_SERVINGS = 20;
const FALLBACK_SERVINGS = 2;

/** Acota las raciones al rango válido (1..20) y cae en 2 si no es finito. */
function clampServings(value: number): number {
  if (!Number.isFinite(value)) {
    return FALLBACK_SERVINGS;
  }
  return Math.min(MAX_SERVINGS, Math.max(MIN_SERVINGS, value));
}

/**
 * Detalle de una receta con acciones de edición y borrado.
 *
 * El contenido se monta con `key={id}`: React Router reutiliza la instancia de
 * la ruta al navegar entre recetas, así que sin la key el estado local
 * (raciones, «Ver originales», edición) se filtraría de una receta a otra.
 */
export function RecipeDetailPage() {
  const { id } = useParams<{ id: string }>();
  return <RecipeDetailContent key={id ?? ''} id={id} />;
}

interface RecipeDetailContentProps {
  id: string | undefined;
}

function RecipeDetailContent({ id }: RecipeDetailContentProps) {
  const { message } = AntdApp.useApp();
  const navigate = useNavigate();
  const online = useOnline();
  const { data, isLoading, isError, error, refetch } = useRecipe(id);
  const { data: settings } = useSettings();
  const updateRecipe = useUpdateRecipe();
  const deleteRecipe = useDeleteRecipe();
  const addToList = useAddRecipeToShoppingList();
  const [editOpen, setEditOpen] = useState(false);
  // Edición en línea de la lista de ingredientes (sin abrir el modal).
  const [editingIngredients, setEditingIngredients] = useState(false);
  // `null` mientras no se toque el selector: así el valor cae en la preferencia
  // de comensales (o en 2) aunque la consulta resuelva después del primer render.
  const [servings, setServings] = useState<number | null>(null);
  const [showOriginal, setShowOriginal] = useState(false);

  const defaultServings = clampServings(settings?.default_servings ?? FALLBACK_SERVINGS);
  // Se acota también el valor elegido: `InputNumber` solo clampea al perder el
  // foco, así que el factor y el alta en la compra nunca deben usar 0 ni > 20.
  const selectedServings = clampServings(servings ?? defaultServings);
  const realQuantities = data ? hasRealQuantities(data.ingredients) : true;
  const factor = data && data.servings > 0 ? selectedServings / data.servings : 1;
  const showScaled = realQuantities && !showOriginal;

  const handleUpdate = (input: RecipeInput) => {
    if (!id) {
      return;
    }
    updateRecipe.mutate(
      { id, input },
      {
        onSuccess: () => {
          message.success('Receta actualizada');
          setEditOpen(false);
        },
        onError: (mutationError: unknown) => {
          message.error(
            mutationError instanceof Error
              ? mutationError.message
              : 'No se pudo actualizar la receta',
          );
        },
      },
    );
  };

  /**
   * Persiste una edición en línea de los ingredientes reutilizando el mismo
   * `PUT /api/recipes/{id}` que el modal: se envía la receta completa con la
   * nueva lista y el resto de campos (incluidas etiquetas manuales) intactos.
   */
  const handleSaveIngredients = (ingredients: IngredientInput[]) => {
    if (!id || !data) {
      return;
    }
    const input: RecipeInput = {
      name: data.name,
      category: data.category,
      diet_type: data.diet_type,
      prep_time_minutes: data.prep_time_minutes,
      servings: data.servings,
      instructions: data.instructions,
      source_url: data.source_url,
      ingredients,
      // Se preservan las etiquetas manuales; las automáticas se recalculan en
      // el backend, igual que hace el modal.
      tags: data.tags.filter((tag) => tag.source !== 'auto').map((tag) => tag.name),
      auto_tags: true,
      suppressed_auto_tags: [],
    };
    updateRecipe.mutate(
      { id, input },
      {
        onSuccess: () => {
          message.success('Ingredientes actualizados');
          setEditingIngredients(false);
        },
        onError: (mutationError: unknown) => {
          message.error(
            mutationError instanceof Error
              ? mutationError.message
              : 'No se pudieron actualizar los ingredientes',
          );
        },
      },
    );
  };

  const handleDelete = () => {
    if (!id) {
      return;
    }
    deleteRecipe.mutate(id, {
      onSuccess: () => {
        message.success('Receta eliminada');
        navigate('/recipes');
      },
      onError: (mutationError: unknown) => {
        message.error(
          mutationError instanceof Error ? mutationError.message : 'No se pudo eliminar la receta',
        );
      },
    });
  };

  const handleAddToList = () => {
    if (!id) {
      return;
    }
    addToList.mutate(
      { recipe_id: id, servings: selectedServings },
      {
        onSuccess: () => {
          message.success({
            content: (
              <>
                Receta añadida a la lista.{' '}
                {/* Ancla plana: el mensaje de antd se renderiza fuera del
                    Router, donde un <Link> no tendría contexto de navegación. */}
                <a href="/shopping">Ver la lista</a>
              </>
            ),
          });
        },
        onError: (mutationError: unknown) => {
          message.error(
            mutationError instanceof Error
              ? mutationError.message
              : 'No se pudo añadir la receta a la lista',
          );
        },
      },
    );
  };

  if (isLoading) {
    return (
      <div className="recipe-detail__loading" data-testid="recipe-loading">
        <Skeleton active paragraph={{ rows: 6 }} />
      </div>
    );
  }

  if (isError || !data) {
    const notFound = error instanceof ApiError && error.status === 404;

    return (
      <Result
        status={notFound ? '404' : 'error'}
        title={notFound ? 'Receta no encontrada' : 'No se pudo cargar la receta'}
        subTitle={
          notFound
            ? 'La receta que buscas no existe o fue eliminada.'
            : error instanceof Error
              ? error.message
              : undefined
        }
        extra={
          <div className="recipe-detail__result-actions">
            {!notFound && <Button onClick={() => void refetch()}>Reintentar</Button>}
            <Link to="/recipes">Volver a recetas</Link>
          </div>
        }
      />
    );
  }

  return (
    <article aria-labelledby="recipe-title" className="recipe-detail">
      <header className="recipe-detail__header">
        <Title id="recipe-title" level={2} className="recipe-detail__title">
          {data.name}
        </Title>
        <Tag color="green" className="recipe-detail__category">
          {categoryLabel(data.category)}
        </Tag>
      </header>

      <Paragraph className="recipe-detail__meta" type="secondary">
        {formatRecipeMeta(data.prep_time_minutes, data.servings)}
      </Paragraph>

      <div className="recipe-detail__scaling">
        <InputNumber
          className="recipe-detail__servings"
          min={1}
          max={20}
          value={selectedServings}
          onChange={(value) => setServings(value)}
          disabled={!online}
          aria-label="Raciones"
        />
        <Checkbox
          className="recipe-detail__original-toggle"
          checked={showOriginal}
          onChange={(event) => setShowOriginal(event.target.checked)}
          disabled={!realQuantities}
        >
          Ver originales
        </Checkbox>
      </div>

      {!realQuantities ? (
        <Alert
          type="info"
          showIcon
          data-testid="recipe-detail-imported"
          message="La importación no trae cantidades; no se escalan"
        />
      ) : (
        selectedServings !== data.servings && (
          <Alert
            type="info"
            showIcon
            data-testid="recipe-detail-scaling-notice"
            message={`Receta para ${data.servings} · cantidades para ${selectedServings}`}
          />
        )
      )}

      {!online && (
        <Alert
          type="warning"
          showIcon
          data-testid="recipe-detail-offline"
          message="Sin conexión: no puedes editar, borrar ni volcar esta receta a la lista."
        />
      )}

      <div className="recipe-detail__actions">
        <Button
          className="recipe-detail__action"
          disabled={!online}
          onClick={() => setEditOpen(true)}
        >
          Editar
        </Button>
        <Popconfirm
          title="¿Borrar esta receta?"
          okText="Borrar"
          cancelText="Cancelar"
          okButtonProps={{ danger: true }}
          onConfirm={handleDelete}
        >
          <Button
            danger
            className="recipe-detail__action"
            aria-label="Borrar receta"
            disabled={!online}
          >
            Borrar
          </Button>
        </Popconfirm>
        <Button
          className="recipe-detail__add-to-list"
          type="primary"
          disabled={!online}
          loading={addToList.isPending}
          onClick={handleAddToList}
        >
          Añadir a la lista
        </Button>
      </div>

      {data.diet_type && (
        <Paragraph className="recipe-detail__diet">Dieta: {data.diet_type}</Paragraph>
      )}

      {data.tags.length > 0 && (
        <ul className="recipe-detail__tags" aria-label="Etiquetas">
          {data.tags.map((tag) => (
            <li
              key={tag.id}
              className={`recipe-detail__tag recipe-detail__tag--${
                tag.source === 'auto' ? 'auto' : 'manual'
              }`}
            >
              {tag.name}
            </li>
          ))}
        </ul>
      )}

      <Tabs
        className="recipe-detail__tabs"
        defaultActiveKey="instructions"
        items={[
          {
            key: 'instructions',
            label: 'Instrucciones',
            children: data.instructions ? (
              <section
                aria-labelledby="recipe-instructions-title"
                className="recipe-detail__section"
              >
                <h3 id="recipe-instructions-title" className="recipe-detail__section-title">
                  Instrucciones
                </h3>
                <Paragraph className="recipe-detail__instructions">{data.instructions}</Paragraph>
              </section>
            ) : (
              <Paragraph className="recipe-detail__instructions-empty">
                Esta receta no tiene instrucciones.
              </Paragraph>
            ),
          },
          {
            key: 'ingredients',
            label: 'Ingredientes',
            children: (
              <IngredientEditor
                ingredients={data.ingredients}
                factor={factor}
                showScaled={showScaled}
                editable={online}
                saving={updateRecipe.isPending}
                editing={editingIngredients}
                onStartEdit={() => setEditingIngredients(true)}
                onCancel={() => setEditingIngredients(false)}
                onSave={handleSaveIngredients}
              />
            ),
          },
        ]}
      />

      {data.source_url && (
        <a
          className="recipe-detail__source"
          href={data.source_url}
          target="_blank"
          rel="noopener noreferrer"
        >
          Fuente original ↗
        </a>
      )}

      <RecipeForm
        open={editOpen}
        initial={data}
        submitting={updateRecipe.isPending}
        onClose={() => setEditOpen(false)}
        onSubmit={handleUpdate}
      />
    </article>
  );
}
