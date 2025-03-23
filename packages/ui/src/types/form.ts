import type { ControllerProps, FieldValues, Path } from "react-hook-form";

export type FormFieldContextValue<
  TFieldValues extends FieldValues = FieldValues,
  TName extends Path<TFieldValues> = Path<TFieldValues>
> = {
  name: TName;
};

export type FormItemContextValue = {
  id: string;
};

export type FormFieldProps<
  TFieldValues extends FieldValues = FieldValues,
  TName extends Path<TFieldValues> = Path<TFieldValues>
> = Omit<ControllerProps<TFieldValues, TName>, "render"> & {
  name: TName;
  children?: React.ReactNode;
};

export interface UseFormReturn<TFieldValues extends FieldValues = FieldValues> {
  formState: {
    errors: any;
    isDirty: boolean;
    isSubmitting: boolean;
    isValid: boolean;
    isValidating: boolean;
    submitCount: number;
    touchedFields: Record<keyof TFieldValues, boolean>;
  };
  getValues: () => TFieldValues;
  setValue: (name: Path<TFieldValues>, value: any) => void;
  trigger: (name?: Path<TFieldValues>) => Promise<boolean>;
  reset: () => void;
  control: any;
}
