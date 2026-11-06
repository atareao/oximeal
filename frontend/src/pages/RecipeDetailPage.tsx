import { useState } from 'react';
import { Link, useNavigate, useParams } from 'react-router-dom';
import {
  Alert,
  App as AntdApp,
  Button,
  InputNumber,
  Popconfirm,
  Result,
  Skeleton,
  Tag,
  Typography,
} from 'antd';
import {
  useAddRecipeToShoppingList,
  useDeleteRecipe,
  useRecipe,
  useUpdateRecipe,
} from '../api/hooks';
import { ApiError } from '../api/client';
import type { RecipeInput } from '../api/types';
import { RecipeForm } from '../components/RecipeForm';
import { formatAmount } from '../lib/shopping';
import { categoryLabel, formatRecipeMeta } from '../lib/recipes';
import { useOnline } from '../offline/connectivity';
import './RecipeDetailPage.css';

const { Title, Paragraph } = Typography;

/** Detalle de una receta con acciones de edición y borrado. */
export function RecipeDetailPage() {
  const { id } = useParams<{ id: string }>();
  const { message } = AntdApp.useApp();
  const navigate = useNavigate();
  const online = useOnline();
  const { data, isLoading, isError, error, refetch } = useRecipe(id);
  const updateRecipe = useUpdateRecipe();
  const deleteRecipe = useDeleteRecipe();
  const addToList = useAddRecipeToShoppingList();
  const [editOpen, setEditOpen] = useState(false);
  // `null` mientras no se toque el selector: así el valor cae en las raciones
  // de la receta aunque estas lleguen después del primer render.
  const [servings, setServings] = useState<number | null>(null);

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
      { recipe_id: id, servings: servings ?? data?.servings ?? 2 },
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
        <InputNumber
          className="recipe-detail__servings"
          min={1}
          value={servings ?? data.servings}
          onChange={(value) => setServings(value)}
          disabled={!online}
          aria-label="Raciones para la lista"
        />
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

      <section aria-labelledby="recipe-ingredients-title" className="recipe-detail__section">
        <h3 id="recipe-ingredients-title" className="recipe-detail__section-title">
          Ingredientes
        </h3>
        <ul className="recipe-detail__ingredients">
          {data.ingredients.map((ingredient, index) => (
            <li key={`${ingredient.name}-${index}`} className="recipe-detail__ingredient">
              <span className="recipe-detail__ingredient-name">{ingredient.name}</span>
              <span className="recipe-detail__ingredient-amount">
                {`${formatAmount(ingredient.amount)} ${ingredient.unit}`}
              </span>
            </li>
          ))}
        </ul>
      </section>

      {data.instructions && (
        <section aria-labelledby="recipe-instructions-title" className="recipe-detail__section">
          <h3 id="recipe-instructions-title" className="recipe-detail__section-title">
            Instrucciones
          </h3>
          <Paragraph className="recipe-detail__instructions">{data.instructions}</Paragraph>
        </section>
      )}

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
