import * as SwitchPrimitive from '@radix-ui/react-switch';
import { cn } from '@/utils/cn';

interface SwitchProps extends SwitchPrimitive.SwitchProps {
  className?: string;
}

export function Switch({ className, ...props }: SwitchProps) {
  return (
    <SwitchPrimitive.Root
      className={cn(
        'group relative inline-flex h-6 w-12 cursor-pointer items-center rounded-full bg-(--meta-text) transition-colors duration-200 ease-in-out focus-visible:outline-none focus-visible:ring-(--accent) focus-visible:ring-2 focus-visible:ring-offset-(--background) focus-visible:ring-offset-2',
        'data-[state=checked]:bg-(--accent)',
        className,
      )}
      {...props}
    >
      <SwitchPrimitive.Thumb
        className={cn(
          'pointer-events-none block h-5 w-5 rounded-full bg-white shadow-lg ring-0 transition-transform duration-200 ease-in-out',
          'translate-x-0.5 data-[state=checked]:translate-x-6.5',
        )}
      />
    </SwitchPrimitive.Root>
  );
}
