import { useState, type FormEvent } from 'react';
import { App as AntdApp, Button, Input } from 'antd';
import { useAddShoppingItem } from '../api/hooks';
import './AddItemForm.css';

const UNITS = ['ud', 'g', 'kg', 'ml', 'l', 'cda', 'cdta'] as const;

/** Formulario de alta rápida de ítems de la compra. */
export function AddItemForm() {
  const { message } = AntdApp.useApp();
  const [name, setName] = useState('');
  const [amount, setAmount] = useState('1');
  const [unit, setUnit] = useState<string>('ud');
  const addItem = useAddShoppingItem();

  const handleSubmit = (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const trimmedName = name.trim();
    const parsedAmount = Number(amount.replace(',', '.'));

    if (!trimmedName) {
      message.warning('Escribe el nombre del producto');
      return;
    }
    if (!Number.isFinite(parsedAmount) || parsedAmount <= 0) {
      message.warning('La cantidad debe ser mayor que cero');
      return;
    }

    // Capturamos lo enviado para no borrar lo que el usuario escriba mientras
    // la petición está en vuelo.
    const submittedName = name;
    const submittedAmount = amount;
    const submittedUnit = unit;

    addItem.mutate(
      { name: trimmedName, amount: parsedAmount, unit },
      {
        onSuccess: () => {
          setName((current) => (current === submittedName ? '' : current));
          setAmount((current) => (current === submittedAmount ? '1' : current));
          setUnit((current) => (current === submittedUnit ? 'ud' : current));
        },
        onError: (error: unknown) => {
          message.error(
            error instanceof Error ? error.message : 'No se pudo añadir el ítem',
          );
        },
      },
    );
  };

  return (
    <form className="add-item-form" onSubmit={handleSubmit} aria-label="Añadir ítem">
      <div className="add-item-form__field add-item-form__field--name">
        <label className="add-item-form__label" htmlFor="add-item-name">
          Nombre del producto
        </label>
        <Input
          id="add-item-name"
          value={name}
          onChange={(event) => setName(event.target.value)}
          placeholder="p. ej. Leche"
          autoComplete="off"
          required
          aria-required="true"
        />
      </div>
      <div className="add-item-form__field add-item-form__field--amount">
        <label className="add-item-form__label" htmlFor="add-item-amount">
          Cantidad
        </label>
        <Input
          id="add-item-amount"
          type="number"
          min={0.01}
          step="any"
          inputMode="decimal"
          value={amount}
          onChange={(event) => setAmount(event.target.value)}
          required
          aria-required="true"
        />
      </div>
      <div className="add-item-form__field add-item-form__field--unit">
        <label className="add-item-form__label" htmlFor="add-item-unit">
          Unidad
        </label>
        <select
          id="add-item-unit"
          className="add-item-form__select"
          value={unit}
          onChange={(event) => setUnit(event.target.value)}
        >
          {UNITS.map((option) => (
            <option key={option} value={option}>
              {option}
            </option>
          ))}
        </select>
      </div>
      <Button
        type="primary"
        htmlType="submit"
        className="add-item-form__submit"
        loading={addItem.isPending}
      >
        Añadir
      </Button>
    </form>
  );
}
