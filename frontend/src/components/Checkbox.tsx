import * as CheckboxPrimitive from '@radix-ui/react-checkbox';
import { Check } from 'lucide-react';

export function Checkbox(
  props: React.ComponentProps<typeof CheckboxPrimitive.Root>,
) {
  return (
    <CheckboxPrimitive.Root
      className="peer size-5 shrink-0 cursor-pointer rounded border border-(--border) disabled:cursor-not-allowed disabled:opacity-50 data-[state=checked]:border-(--accent) data-[state=checked]:bg-(--accent)"
      {...props}
    >
      <CheckboxPrimitive.Indicator className="flex items-center justify-center">
        <Check size={14} strokeWidth={4} />
      </CheckboxPrimitive.Indicator>
    </CheckboxPrimitive.Root>
  );
}
