import { useRef } from 'react';
import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import {
  clearCompletedShoppingItems,
  consolidateShoppingList,
  createMealPlanEntry,
  createRecipe,
  createShoppingItem,
  createTag,
  deleteMealPlanEntry,
  deleteRecipe,
  deleteTag,
  fetchMealPlan,
  fetchRecipe,
  fetchRecipes,
  fetchShoppingList,
  fetchTags,
  importRecipe,
  patchMealPlanEntry,
  patchShoppingItem,
  updateRecipe,
} from './client';
import type {
  MealPlanInput,
  MealPlanPatchInput,
  RecipeImportInput,
  RecipeImportResult,
  RecipeInput,
  RecipeQueryParams,
  ShoppingItem,
  ShoppingItemInput,
  ShoppingItemPatch,
  ShoppingListGroup,
} from './types';
import { isOnline } from '../offline/connectivity';
import {
  enqueueCreate,
  enqueueUpdate,
  isLocalItemId,
  isNetworkError,
} from '../offline/mutationQueue';

export const queryKeys = {
  recipes: {
    all: ['recipes'] as const,
    list: (params: RecipeQueryParams) => ['recipes', 'list', params] as const,
    detail: (id: string) => ['recipes', 'detail', id] as const,
  },
  mealPlan: {
    all: ['meal-plan'] as const,
    range: (startDate: string, endDate: string) => ['meal-plan', startDate, endDate] as const,
  },
  shoppingList: {
    all: ['shopping-list'] as const,
  },
  tags: {
    all: ['tags'] as const,
  },
};

export function useRecipes(params: RecipeQueryParams = {}) {
  return useQuery({
    queryKey: queryKeys.recipes.list(params),
    queryFn: ({ signal }) => fetchRecipes(params, signal),
    // Sin conexión se sirven los datos cacheados sin entrar en estado de error.
    networkMode: 'offlineFirst',
  });
}

export function useRecipe(id: string | undefined) {
  return useQuery({
    queryKey: queryKeys.recipes.detail(id ?? ''),
    queryFn: ({ signal }) => fetchRecipe(id as string, signal),
    enabled: Boolean(id),
    // Sin conexión se sirve el detalle cacheado si lo hubiera.
    networkMode: 'offlineFirst',
  });
}

/**
 * Alta de receta. Invalida el catálogo completo para que el listado refleje la
 * nueva receta independientemente de la página o el filtro activos.
 */
export function useCreateRecipe() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: RecipeInput) => createRecipe(input),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.recipes.all });
    },
  });
}

/**
 * Importación de receta desde una URL (Schema.org). Invalida el catálogo
 * completo para que el listado refleje la receta importada.
 *
 * Usa el `networkMode` por defecto (`'online'`): importar exige que el
 * servidor alcance el origen remoto, de modo que sin conexión la mutación
 * queda **pausada** en lugar de aparentar un éxito. La UI deshabilita el
 * botón offline (como el alta), así que en la práctica no se dispara.
 */
export function useImportRecipe() {
  const queryClient = useQueryClient();
  return useMutation<RecipeImportResult, Error, RecipeImportInput>({
    mutationFn: (input: RecipeImportInput) => importRecipe(input),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.recipes.all });
    },
  });
}

/** Edición de receta. Invalida el catálogo y el detalle concreto. */
export function useUpdateRecipe() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ id, input }: { id: string; input: RecipeInput }) => updateRecipe(id, input),
    onSuccess: (_recipe, { id }) => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.recipes.all });
      void queryClient.invalidateQueries({ queryKey: queryKeys.recipes.detail(id) });
    },
  });
}

/** Borrado de receta. Invalida el catálogo completo. */
export function useDeleteRecipe() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => deleteRecipe(id),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.recipes.all });
    },
  });
}

/**
 * Catálogo de etiquetas con su recuento de recetas. Sin conexión se sirven las
 * etiquetas cacheadas (mismo patrón `offlineFirst` que el recetario).
 */
export function useTags() {
  return useQuery({
    queryKey: queryKeys.tags.all,
    queryFn: ({ signal }) => fetchTags(signal),
    networkMode: 'offlineFirst',
  });
}

/**
 * Alta de etiqueta (idempotente por `slug`). Invalida el catálogo de etiquetas.
 *
 * API preparada para la futura gestión de etiquetas; todavía sin UI.
 */
export function useCreateTag() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (name: string) => createTag(name),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.tags.all });
    },
  });
}

/**
 * Borrado de etiqueta. Invalida el catálogo de etiquetas y las recetas, cuyas
 * tarjetas y detalles pueden mostrar la etiqueta eliminada.
 *
 * API preparada para la futura gestión de etiquetas; todavía sin UI.
 */
export function useDeleteTag() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => deleteTag(id),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.tags.all });
      void queryClient.invalidateQueries({ queryKey: queryKeys.recipes.all });
    },
  });
}

export function useMealPlan(startDate: string, endDate: string) {
  return useQuery({
    queryKey: queryKeys.mealPlan.range(startDate, endDate),
    queryFn: ({ signal }) => fetchMealPlan(startDate, endDate, signal),
    enabled: Boolean(startDate && endDate),
  });
}

/**
 * Invalida el plan y la lista de la compra (que puede derivarse del menú).
 */
function invalidateMealPlanQueries(queryClient: ReturnType<typeof useQueryClient>): void {
  void queryClient.invalidateQueries({ queryKey: queryKeys.mealPlan.all });
  void queryClient.invalidateQueries({ queryKey: queryKeys.shoppingList.all });
}

/**
 * Alta de una entrada del plan.
 *
 * Usa el `networkMode` por defecto (`'online'`): la planificación **requiere
 * conexión** y no se encola en IndexedDB (a diferencia de la lista de la
 * compra). Sin conexión la mutación queda pausada y la UI deshabilita los
 * controles.
 */
export function useAddMealPlanEntry() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: MealPlanInput) => createMealPlanEntry(input),
    onSuccess: () => {
      invalidateMealPlanQueries(queryClient);
    },
  });
}

/** Baja de una entrada del plan. Requiere conexión (no se encola). */
export function useRemoveMealPlanEntry() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (id: string) => deleteMealPlanEntry(id),
    onSuccess: () => {
      invalidateMealPlanQueries(queryClient);
    },
  });
}

/** Edición de las raciones de una entrada. Requiere conexión (no se encola). */
export function useUpdateMealPlanEntry() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: ({ id, input }: { id: string; input: MealPlanPatchInput }) =>
      patchMealPlanEntry(id, input),
    onSuccess: () => {
      invalidateMealPlanQueries(queryClient);
    },
  });
}

export function useShoppingList() {
  return useQuery({
    queryKey: queryKeys.shoppingList.all,
    queryFn: ({ signal }) => fetchShoppingList(signal),
    // Sin conexión se sirven los datos cacheados sin entrar en estado de error.
    networkMode: 'offlineFirst',
  });
}

/** Busca un ítem por id en la lista agrupada. */
export function findShoppingItem(
  groups: ShoppingListGroup[] | undefined,
  id: string,
): ShoppingItem | undefined {
  if (!groups) {
    return undefined;
  }
  for (const group of groups) {
    const found = group.items.find((item) => item.id === id);
    if (found) {
      return found;
    }
  }
  return undefined;
}

/**
 * Inserta un ítem en el grupo de su categoría (o en `Otros`), creando el grupo
 * si no existe.
 */
function optimisticInsert(
  groups: ShoppingListGroup[] | undefined,
  localItem: ShoppingItem,
): ShoppingListGroup[] {
  const category = localItem.category || 'Otros';
  const base = groups ?? [];
  const index = base.findIndex((group) => group.category === category);

  if (index === -1) {
    return [...base, { category, items: [localItem] }];
  }

  return base.map((group, groupIndex) =>
    groupIndex === index ? { ...group, items: [...group.items, localItem] } : group,
  );
}

/** Encola un alta y la aplica de forma optimista a la caché. */
async function queueOfflineCreate(
  queryClient: ReturnType<typeof useQueryClient>,
  input: ShoppingItemInput,
): Promise<ShoppingItem> {
  const { clientId, localId } = await enqueueCreate(input);
  const optimistic: ShoppingItem = {
    id: localId,
    name: input.name,
    amount: input.amount,
    unit: input.unit,
    category: input.category ?? 'Otros',
    checked: false,
    updated_at: new Date().toISOString(),
    client_id: clientId,
    pending: true,
  };

  queryClient.setQueryData<ShoppingListGroup[]>(queryKeys.shoppingList.all, (old) =>
    optimisticInsert(old, optimistic),
  );

  return optimistic;
}

interface ToggleContext {
  previous: ShoppingListGroup[] | undefined;
  /**
   * `true` si el cambio no se enviará al servidor en este momento (id
   * provisional o sin conexión), es decir, se encola. Capturado en `onMutate`
   * para que el rollback no dependa de un `isOnline()` posterior.
   */
  deferred: boolean;
  /** `true` cuando el cambio llegó a encolarse. */
  queued: boolean;
}

function applyItemPatch(
  groups: ShoppingListGroup[] | undefined,
  id: string,
  patch: Partial<ShoppingItem>,
): ShoppingListGroup[] | undefined {
  if (!groups) {
    return groups;
  }
  return groups.map((group) => ({
    ...group,
    items: group.items.map((item) => (item.id === id ? { ...item, ...patch } : item)),
  }));
}

/**
 * Alterna el estado `checked` de un ítem con actualización optimista.
 *
 * - `onMutate`: cancela refetches en vuelo, guarda un snapshot y aplica el
 *   cambio en la caché para feedback inmediato. Si el ítem es local o no hay
 *   conexión, se marca además como `pending`.
 * - Sin conexión o con id provisional: el cambio se encola y **no** se llama al
 *   servidor; no hay rollback porque no hay petición que pueda fallar.
 * - `onError`: restaura el snapshot (rollback) solo si el cambio se dirigió al
 *   servidor (`deferred === false`); un cambio encolado nunca se revierte.
 * - `onSuccess`: sustituye el ítem por la versión del servidor, de modo que
 *   `updated_at` proviene del backend y no del reloj del cliente.
 * - `onSettled`: invalida para reconciliar con el servidor (solo online).
 *
 * `scope` serializa las mutaciones de toggle: dos pulsaciones rápidas se
 * ejecutan en orden y no se pisan entre sí.
 */
export function useToggleShoppingItem() {
  const queryClient = useQueryClient();
  // Comparte la decisión capturada en `onMutate` con `mutationFn`, que no
  // recibe el `onMutateResult`. `scope` serializa los toggles, así que solo hay
  // un contexto vivo a la vez.
  const contextRef = useRef<ToggleContext | undefined>(undefined);

  return useMutation<ShoppingItem, Error, { id: string; patch: ShoppingItemPatch }, ToggleContext>({
    scope: { id: 'shopping-toggle' },
    networkMode: 'always',
    mutationFn: async ({ id, patch }: { id: string; patch: ShoppingItemPatch }) => {
      const context = contextRef.current;
      const deferred = context?.deferred ?? (isLocalItemId(id) || !isOnline());

      if (deferred) {
        await enqueueUpdate(id, patch);
        if (context) {
          context.queued = true;
        }
        const current = findShoppingItem(
          queryClient.getQueryData<ShoppingListGroup[]>(queryKeys.shoppingList.all),
          id,
        );
        if (current) {
          return current;
        }
        return {
          id,
          name: '',
          amount: 0,
          unit: '',
          category: 'Otros',
          checked: false,
          updated_at: new Date().toISOString(),
          pending: true,
        };
      }
      return patchShoppingItem(id, patch);
    },
    onMutate: async ({ id, patch }) => {
      await queryClient.cancelQueries({ queryKey: queryKeys.shoppingList.all });
      const previous = queryClient.getQueryData<ShoppingListGroup[]>(queryKeys.shoppingList.all);
      const deferred = isLocalItemId(id) || !isOnline();
      const context: ToggleContext = { previous, deferred, queued: false };
      contextRef.current = context;
      queryClient.setQueryData<ShoppingListGroup[]>(queryKeys.shoppingList.all, (old) =>
        applyItemPatch(old, id, deferred ? { ...patch, pending: true } : patch),
      );
      return context;
    },
    onError: (_error, _variables, context) => {
      // Solo se revierte un cambio que iba al servidor; si se difirió (encoló),
      // no hubo petición que fallar.
      if (context?.deferred !== false) {
        return;
      }
      if (context.previous) {
        queryClient.setQueryData(queryKeys.shoppingList.all, context.previous);
      }
    },
    onSuccess: (updated, _variables, context) => {
      if (context?.deferred !== false) {
        return;
      }
      queryClient.setQueryData<ShoppingListGroup[]>(queryKeys.shoppingList.all, (old) =>
        applyItemPatch(old, updated.id, updated),
      );
    },
    onSettled: (_data, _error, _variables, context) => {
      if (context?.deferred !== false) {
        return;
      }
      void queryClient.invalidateQueries({ queryKey: queryKeys.shoppingList.all });
    },
  });
}

/**
 * Alta de un ítem.
 *
 * Sin conexión (o si la petición falla por red) el alta se encola y se aplica
 * de forma optimista con un id provisional, sin propagar el error al
 * formulario. Con conexión se envía al servidor y se invalida la lista.
 */
export function useAddShoppingItem() {
  const queryClient = useQueryClient();
  return useMutation<ShoppingItem, Error, ShoppingItemInput>({
    networkMode: 'always',
    mutationFn: async (input: ShoppingItemInput) => {
      if (!isOnline()) {
        return queueOfflineCreate(queryClient, input);
      }
      try {
        return await createShoppingItem(input);
      } catch (error) {
        if (isNetworkError(error)) {
          return queueOfflineCreate(queryClient, input);
        }
        throw error;
      }
    },
    onSuccess: () => {
      if (isOnline()) {
        void queryClient.invalidateQueries({ queryKey: queryKeys.shoppingList.all });
      }
    },
  });
}

export function useConsolidateShopping() {
  const queryClient = useQueryClient();
  return useMutation({
    networkMode: 'offlineFirst',
    mutationFn: () => consolidateShoppingList(),
    onSuccess: (groups: ShoppingListGroup[]) => {
      queryClient.setQueryData(queryKeys.shoppingList.all, groups);
    },
  });
}

export function useClearCompleted() {
  const queryClient = useQueryClient();
  return useMutation({
    networkMode: 'offlineFirst',
    mutationFn: () => clearCompletedShoppingItems(),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.shoppingList.all });
    },
  });
}
