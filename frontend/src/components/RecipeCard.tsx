import { Link } from 'react-router-dom';
import type { RecipeSummary } from '../api/types';
import { categoryLabel, formatRecipeMeta } from '../lib/recipes';
import './RecipeCard.css';

export interface RecipeCardProps {
  recipe: RecipeSummary;
  /** Inicia el filtrado del recetario por la etiqueta indicada (su `slug`). */
  onSelectTag?: (slug: string) => void;
}

/** Tarjeta-enlace de una receta del listado, con sus etiquetas filtrables. */
export function RecipeCard({ recipe, onSelectTag }: RecipeCardProps) {
  return (
    <li className="recipe-card__item">
      <Link to={`/recipes/${recipe.id}`} className="recipe-card">
        <span className="recipe-card__name">{recipe.name}</span>
        <span className="recipe-card__category">{categoryLabel(recipe.category)}</span>
        <span className="recipe-card__meta">
          {formatRecipeMeta(recipe.prep_time_minutes, recipe.servings)}
        </span>
      </Link>
      {recipe.tags.length > 0 && (
        <ul className="recipe-card__tags" aria-label={`Etiquetas de ${recipe.name}`}>
          {recipe.tags.map((tag) => (
            <li key={tag.id}>
              <button
                type="button"
                className={`recipe-card__tag recipe-card__tag--${
                  tag.source === 'auto' ? 'auto' : 'manual'
                }`}
                aria-label={`Filtrar por etiqueta ${tag.name}`}
                onClick={() => onSelectTag?.(tag.slug)}
              >
                {tag.name}
              </button>
            </li>
          ))}
        </ul>
      )}
    </li>
  );
}
