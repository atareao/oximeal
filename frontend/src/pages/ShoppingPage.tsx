import { useMemo, useState } from 'react';
import {
  Alert,
  App as AntdApp,
  Button,
  Empty,
  Popconfirm,
  Result,
  Skeleton,
  Typography,
} from 'antd';
import { useQueryClient } from '@tanstack/react-query';
import {
  useClearCompleted,
  useConsolidateShopping,
  useShoppingList,
  useToggleShoppingItem,
} from '../api/hooks';
import type { ShoppingItem } from '../api/types';
import { AddItemForm } from '../components/AddItemForm';
import { ShoppingList } from '../components/ShoppingList';
import { useHaptics } from '../hooks/useHaptics';
import { useOnline } from '../offline/connectivity';
import { useQueuedMutations } from '../offline/mutationQueue';
import { syncShoppingQueue } from '../offline/syncShoppingQueue';
import './ShoppingPage.css';

const { Title, Paragraph } = Typography;

/** Vista de la compra: alta rápida, agrupación por pasillo y marcado táctil. */
export function ShoppingPage() {
  const { message } = AntdApp.useApp();
  const queryClient = useQueryClient();
  const { data, isLoading, isError, error, refetch } = useShoppingList();
  const toggleItem = useToggleShoppingItem();
  const clearCompleted = useClearCompleted();
  const consolidate = useConsolidateShopping();
  const haptics = useHaptics();
  const online = useOnline();
  const queuedMutations = useQueuedMutations();
  const [isSyncing, setIsSyncing] = useState(false);

  const groups = useMemo(() => data ?? [], [data]);

  const checkedCount = useMemo(
    () => groups.reduce((total, group) => total + group.items.filter((i) => i.checked).length, 0),
    [groups],
  );

  const errorMessage =
    error instanceof Error ? error.message : 'Ha ocurrido un error al contactar con el servidor.';

  const handleToggle = (item: ShoppingItem) => {
    haptics(10);
    toggleItem.mutate({ id: item.id, patch: { checked: !item.checked } });
  };

  const handleCategoryChange = (item: ShoppingItem, category: string) => {
    haptics(10);
    toggleItem.mutate({ id: item.id, patch: { category } });
  };

  const handleSync = () => {
    setIsSyncing(true);
    void syncShoppingQueue(queryClient).finally(() => {
      setIsSyncing(false);
    });
  };

  const handleClearCompleted = () => {
    haptics(10);
    clearCompleted.mutate(undefined, {
      onSuccess: () => {
        message.success('Ítems comprados eliminados');
      },
      onError: (mutationError: unknown) => {
        message.error(
          mutationError instanceof Error
            ? mutationError.message
            : 'No se pudieron eliminar los ítems comprados',
        );
      },
    });
  };

  const handleConsolidate = () => {
    consolidate.mutate(undefined, {
      onSuccess: () => {
        message.success('Lista consolidada desde el menú');
      },
      onError: (mutationError: unknown) => {
        message.error(
          mutationError instanceof Error ? mutationError.message : 'No se pudo consolidar la lista',
        );
      },
    });
  };

  return (
    <section aria-labelledby="shopping-title" className="shopping-page">
      <header className="shopping-page__header">
        <Title id="shopping-title" level={2} className="visually-hidden">
          Lista de la compra
        </Title>
        <div className="shopping-page__actions">
          <Popconfirm
            title="¿Consolidar la lista?"
            description="Reemplaza los ítems derivados del plan del menú con los pasillos canónicos."
            okText="Consolidar"
            cancelText="Cancelar"
            onConfirm={handleConsolidate}
          >
            <Button loading={consolidate.isPending}>Consolidar desde el menú</Button>
          </Popconfirm>
          <Popconfirm
            title="¿Eliminar los ítems comprados?"
            okText="Eliminar"
            cancelText="Cancelar"
            okButtonProps={{ danger: true }}
            onConfirm={handleClearCompleted}
            disabled={checkedCount === 0}
          >
            <Button danger disabled={checkedCount === 0} loading={clearCompleted.isPending}>
              Limpiar comprados
            </Button>
          </Popconfirm>
        </div>
      </header>

      <AddItemForm />

      {queuedMutations.length > 0 && (
        <Alert
          type="info"
          showIcon
          data-testid="pending-mutations"
          message={`${queuedMutations.length} cambio(s) pendiente(s) de sincronizar`}
          action={
            <Button size="small" onClick={handleSync} disabled={!online} loading={isSyncing}>
              Sincronizar ahora
            </Button>
          }
        />
      )}

      {isLoading && (
        <div className="shopping-page__loading" data-testid="shopping-loading">
          <Skeleton active paragraph={{ rows: 6 }} />
        </div>
      )}

      {!isLoading && isError && !data && (
        <Result
          status="error"
          title="No se pudo cargar la lista"
          subTitle={errorMessage}
          extra={
            <Button type="primary" onClick={() => void refetch()}>
              Reintentar
            </Button>
          }
        />
      )}

      {!isLoading && isError && data && (
        <Alert
          type="error"
          showIcon
          message="No se pudo actualizar la lista"
          description={errorMessage}
          action={
            <Button size="small" onClick={() => void refetch()}>
              Reintentar
            </Button>
          }
        />
      )}

      {!isLoading && !isError && groups.length === 0 && (
        <Empty description="La lista está vacía" image={Empty.PRESENTED_IMAGE_SIMPLE}>
          <Paragraph type="secondary" className="shopping-page__empty-hint">
            Añade productos con el formulario de arriba o consolídalos desde el menú.
          </Paragraph>
        </Empty>
      )}

      {!isLoading && groups.length > 0 && (
        <ShoppingList
          groups={groups}
          onToggle={handleToggle}
          onCategoryChange={handleCategoryChange}
        />
      )}
    </section>
  );
}
