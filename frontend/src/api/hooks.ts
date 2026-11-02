import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import {
  clearCompletedShoppingItems,
  consolidateShoppingList,
  createShoppingItem,
  fetchMealPlan,
  fetchRecipe,
  fetchRecipes,
  fetchShoppingList,
  patchShoppingItem,
} from './client';
import type {
  RecipeQueryParams,
  ShoppingItem,
  ShoppingItemInput,
  ShoppingItemPatch,
  ShoppingListGroup,
} from './types';

export const queryKeys = {
  recipes: {
    all: ['recipes'] as const,
    list: (params: RecipeQueryParams) => ['recipes', 'list', params] as const,
    detail: (id: string) => ['recipes', 'detail', id] as const,
  },
  mealPlan: {
    all: ['meal-plan'] as const,
    range: (startDate: string, endDate: string) =>
      ['meal-plan', startDate, endDate] as const,
  },
  shoppingList: {
    all: ['shopping-list'] as const,
  },
};

export function useRecipes(params: RecipeQueryParams = {}) {
  return useQuery({
    queryKey: queryKeys.recipes.list(params),
    queryFn: ({ signal }) => fetchRecipes(params, signal),
  });
}

export function useRecipe(id: string | undefined) {
  return useQuery({
    queryKey: queryKeys.recipes.detail(id ?? ''),
    queryFn: ({ signal }) => fetchRecipe(id as string, signal),
    enabled: Boolean(id),
  });
}

export function useMealPlan(startDate: string, endDate: string) {
  return useQuery({
    queryKey: queryKeys.mealPlan.range(startDate, endDate),
    queryFn: ({ signal }) => fetchMealPlan(startDate, endDate, signal),
    enabled: Boolean(startDate && endDate),
  });
}

export function useShoppingList() {
  return useQuery({
    queryKey: queryKeys.shoppingList.all,
    queryFn: ({ signal }) => fetchShoppingList(signal),
  });
}

interface ToggleContext {
  previous: ShoppingListGroup[] | undefined;
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
 *   cambio en la caché para feedback inmediato.
 * - `onError`: restaura el snapshot (rollback).
 * - `onSuccess`: sustituye el ítem por la versión del servidor, de modo que
 *   `updated_at` proviene del backend y no del reloj del cliente.
 * - `onSettled`: invalida para reconciliar con el servidor.
 *
 * `scope` serializa las mutaciones de toggle: dos pulsaciones rápidas se
 * ejecutan en orden y no se pisan entre sí.
 */
export function useToggleShoppingItem() {
  const queryClient = useQueryClient();
  return useMutation<ShoppingItem, Error, { id: string; patch: ShoppingItemPatch }, ToggleContext>({
    scope: { id: 'shopping-toggle' },
    mutationFn: ({ id, patch }: { id: string; patch: ShoppingItemPatch }) =>
      patchShoppingItem(id, patch),
    onMutate: async ({ id, patch }) => {
      await queryClient.cancelQueries({ queryKey: queryKeys.shoppingList.all });
      const previous = queryClient.getQueryData<ShoppingListGroup[]>(
        queryKeys.shoppingList.all,
      );
      queryClient.setQueryData<ShoppingListGroup[]>(queryKeys.shoppingList.all, (old) =>
        applyItemPatch(old, id, patch),
      );
      return { previous };
    },
    onError: (_error, _variables, context) => {
      if (context?.previous) {
        queryClient.setQueryData(queryKeys.shoppingList.all, context.previous);
      }
    },
    onSuccess: (updated) => {
      queryClient.setQueryData<ShoppingListGroup[]>(queryKeys.shoppingList.all, (old) =>
        applyItemPatch(old, updated.id, updated),
      );
    },
    onSettled: () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.shoppingList.all });
    },
  });
}

export function useAddShoppingItem() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: (input: ShoppingItemInput) => createShoppingItem(input),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.shoppingList.all });
    },
  });
}

export function useConsolidateShopping() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: () => consolidateShoppingList(),
    onSuccess: (groups: ShoppingListGroup[]) => {
      queryClient.setQueryData(queryKeys.shoppingList.all, groups);
    },
  });
}

export function useClearCompleted() {
  const queryClient = useQueryClient();
  return useMutation({
    mutationFn: () => clearCompletedShoppingItems(),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: queryKeys.shoppingList.all });
    },
  });
}
