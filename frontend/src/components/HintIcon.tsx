import * as Tooltip from '@radix-ui/react-tooltip';
import { Info } from 'lucide-react';
import type { ReactNode } from 'react';

interface HintIconProps {
  text: ReactNode;
}

export function HintIcon({ text }: HintIconProps) {
  return (
    <Tooltip.Provider delayDuration={100}>
      <Tooltip.Root>
        <Tooltip.Trigger asChild>
          <Info
            size={14}
            className="mr-1 cursor-help text-(--meta-text) hover:text-(--text)"
          />
        </Tooltip.Trigger>
        <Tooltip.Portal>
          <Tooltip.Content
            side="right"
            className="z-50 max-w-[350px] rounded-lg bg-(--tooltip-bg) px-2 py-1 text-sm"
          >
            {text}
            <Tooltip.Arrow className="fill-(--tooltip-bg)" />
          </Tooltip.Content>
        </Tooltip.Portal>
      </Tooltip.Root>
    </Tooltip.Provider>
  );
}
