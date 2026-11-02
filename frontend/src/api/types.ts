/**
 * Tipos espejo de los DTOs del backend (Axum).
 * Mantener sincronizados con `backend/src/models/*`.
 */

export interface RecipeSummary {
  id: string;
  name: string;
  category: string;
  diet_type: string;
  prep_time_minutes: number | null;
  servings: number;
  created_at: string;
}

export interface IngredientInput {
  name: string;
  amount: number;
  unit: string;
  category?: string;
}

export interface Recipe extends RecipeSummary {
  ingredients: IngredientInput[];
  instructions: string | null;
  source_url: string | null;
}

export type MealType = 'lunch' | 'dinner' | 'breakfast' | 'snack';
export type MealRole = 'main' | 'salad' | 'side';

export interface MealPlanEntryView {
  id: string;
  date: string;
  meal_type: MealType;
  role: MealRole;
  servings: number;
  recipe: RecipeSummary;
}

export interface ShoppingItem {
  id: string;
  name: string;
  amount: number;
  unit: string;
  category: string;
  checked: boolean;
  updated_at: string;
}

export interface ShoppingListGroup {
  category: string;
  items: ShoppingItem[];
}

export interface RecipeListResponse {
  items: RecipeSummary[];
  total: number;
  page: number;
  page_size: number;
}

export interface RecipeQueryParams {
  q?: string;
  category?: string;
  page?: number;
  page_size?: number;
}

export interface ShoppingItemInput {
  name: string;
  amount: number;
  unit: string;
  category?: string;
}

export interface ShoppingItemPatch {
  checked?: boolean;
  amount?: number;
}

/** Sobre de error estándar del backend. */
export interface ApiErrorBody {
  error: string;
  message: string;
}
