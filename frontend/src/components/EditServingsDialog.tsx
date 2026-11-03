import { useEffect, useRef, useState } from 'react';
import { Alert, Button, Input, Modal } from 'antd';
import type { InputRef } from 'antd';
import type { MealPlanEntryView } from '../api/types';
import './EditServingsDialog.css';

export interface EditServingsDialogProps {
  open: boolean;
  entry: MealPlanEntryView | null;
  submitting: boolean;
  /** Mensaje de error del servidor a mostrar dentro del diálogo. */
  error?: string | null;
  onClose: () => void;
  onSubmit: (servings: number) => void;
}

/** Diálogo para editar las raciones de una entrada del plan. */
export function EditServingsDialog({
  open,
  entry,
  submitting,
  error = null,
  onClose,
  onSubmit,
}: EditServingsDialogProps) {
  const [servings, setServings] = useState('');
  const [validationError, setValidationError] = useState<string | null>(null);
  const [prevOpenKey, setPrevOpenKey] = useState<string | null>(null);

  const openerRef = useRef<HTMLElement | null>(null);
  const inputRef = useRef<InputRef | null>(null);

  // Reinicia el formulario al abrir (o al cambiar de entrada) comparando con el
  // valor previo durante el render, en lugar de con un efecto.
  const openKey = open ? (entry?.id ?? 'entry') : null;
  if (openKey !== prevOpenKey) {
    setPrevOpenKey(openKey);
    if (openKey !== null) {
      setServings(entry ? String(entry.servings) : '');
      setValidationError(null);
    }
  }

  // Gestión de foco: al abrir, el foco entra en el input; al cerrar, vuelve al
  // control que lo abrió. Se hace con efectos para funcionar también sin
  // animaciones, como en los tests.
  useEffect(() => {
    if (!open) {
      openerRef.current?.focus();
      return undefined;
    }
    openerRef.current = document.activeElement as HTMLElement | null;
    const timer = window.setTimeout(() => inputRef.current?.focus(), 0);
    return () => window.clearTimeout(timer);
  }, [open]);

  const handleConfirm = () => {
    const parsed = Number(servings);
    if (!Number.isFinite(parsed) || parsed < 1) {
      setValidationError('Las raciones deben ser al menos 1');
      return;
    }
    setValidationError(null);
    onSubmit(parsed);
  };

  return (
    <Modal
      open={open}
      title={entry ? `Editar raciones de ${entry.recipe.name}` : 'Editar raciones'}
      onCancel={onClose}
      footer={
        <div className="edit-servings__actions">
          <Button className="edit-servings__cancel" onClick={onClose}>
            Cancelar
          </Button>
          <Button
            type="primary"
            className="edit-servings__confirm"
            loading={submitting}
            onClick={handleConfirm}
          >
            Guardar
          </Button>
        </div>
      }
    >
      {error && (
        <Alert
          type="error"
          showIcon
          data-testid="edit-servings-error"
          className="edit-servings__error"
          message={error}
        />
      )}

      <div className="edit-servings__field">
        <label htmlFor="edit-servings-input" className="edit-servings__label">
          Raciones
        </label>
        <Input
          id="edit-servings-input"
          ref={inputRef}
          type="number"
          min={1}
          inputMode="numeric"
          value={servings}
          onChange={(event) => setServings(event.target.value)}
        />
      </div>

      {validationError && (
        <p role="alert" className="edit-servings__validation">
          {validationError}
        </p>
      )}
    </Modal>
  );
}
