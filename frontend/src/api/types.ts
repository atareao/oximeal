/**
 * Tipos espejo de los DTOs del backend (Axum).
 * Mantener sincronizados con `backend/src/models/*`.
 */

/** Etiqueta libre asociada a recetas (espejo del DTO del backend). */
export interface Tag {
  id: string;
  name: string;
  slug: string;
}

/** Etiqueta tal y como viaja dentro de `Recipe`/`RecipeSummary`. */
export type RecipeTag = Tag;

/** Etiqueta del catálogo con su recuento de recetas (`GET /api/tags`). */
export interface TagWithCount extends Tag {
  recipe_count: number;
}

export interface RecipeSummary {
  id: string;
  name: string;
  category: string;
  diet_type: string;
  prep_time_minutes: number | null;
  servings: number;
  created_at: string;
  tags: RecipeTag[];
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

/** Cuerpo de alta/edición de receta (espejo de `RecipeInput` del backend). */
export interface RecipeInput {
  name: string;
  category: string;
  diet_type?: string;
  prep_time_minutes?: number | null;
  servings?: number | null;
  ingredients: IngredientInput[];
  instructions?: string | null;
  source_url?: string | null;
  /** Nombres de las etiquetas; el backend crea al vuelo las que no existan. */
  tags: string[];
}

/** Cuerpo de `POST /api/recipes/import` (espejo de `ImportRecipeRequest`). */
export interface RecipeImportInput {
  url: string;
}

/**
 * Resultado de `POST /api/recipes/import`: la receta importada y si fue creada
 * (`201`) o ya existía por `source_url` (`200`, idempotente).
 */
export interface RecipeImportResult {
  recipe: Recipe;
  created: boolean;
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

/** Cuerpo de `POST /api/meal-plan` (espejo de `MealPlanInput` del backend). */
export interface MealPlanInput {
  date: string;
  meal_type: MealType;
  recipe_id: string;
  role: MealRole;
  /** Raciones (por defecto 2 en el backend). */
  servings?: number;
}

/** Cuerpo de `PATCH /api/meal-plan/{id}`: solo se editan las raciones. */
export interface MealPlanPatchInput {
  servings: number;
}

export interface ShoppingItem {
  id: string;
  name: string;
  amount: number;
  unit: string;
  category: string;
  checked: boolean;
  updated_at: string;
  /** Identificador de cliente del alta idempotente, si lo hubo. */
  client_id?: string | null;
  /** Solo en cliente: ítem creado sin conexión y aún no confirmado por el servidor. */
  pending?: boolean;
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
  /** Filtro por etiqueta: `id` o `slug`. */
  tag?: string;
  page?: number;
  page_size?: number;
}

export interface ShoppingItemInput {
  name: string;
  amount: number;
  unit: string;
  category?: string;
  client_id?: string;
}

export interface ShoppingItemPatch {
  checked?: boolean;
  amount?: number;
  category?: string;
}

/** Cuerpo de `POST /api/shopping-list/from-recipe` (espejo de `RecipeToShoppingListInput`). */
export interface RecipeToShoppingListInput {
  recipe_id: string;
  /** Raciones a volcar; si se omite, el backend usa las de la receta. */
  servings?: number;
}

/** Sobre de error estándar del backend. */
export interface ApiErrorBody {
  error: string;
  message: string;
}

/**
 * Respuesta de `GET /auth/me` (contrato BFF).
 *
 * En modo OIDC desactivado el backend no monta `/auth/*` y la petición
 * responde 404. En sesión anónima `authenticated` es `false` y el resto de
 * campos llega ausente.
 */
export interface AuthMeResponse {
  authenticated: boolean;
  subject?: string;
  name?: string;
  permissions?: string[];
  csrf_token?: string;
}
