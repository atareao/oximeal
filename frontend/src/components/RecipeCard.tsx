import { Link } from 'react-router-dom';
import type { RecipeSummary } from '../api/types';
import { categoryLabel, formatRecipeMeta } from '../lib/recipes';
import './RecipeCard.css';

export interface RecipeCardProps {
  recipe: RecipeSummary;
}

/** Tarjeta-enlace de una receta del listado. */
export function RecipeCard({ recipe }: RecipeCardProps) {
  return (
    <li className="recipe-card__item">
      <Link to={`/recipes/${recipe.id}`} className="recipe-card">
        <span className="recipe-card__name">{recipe.name}</span>
        <span className="recipe-card__category">{categoryLabel(recipe.category)}</span>
        <span className="recipe-card__meta">
          {formatRecipeMeta(recipe.prep_time_minutes, recipe.servings)}
        </span>
      </Link>
    </li>
  );
}
