import * as Tooltip from '@radix-ui/react-tooltip';
import { Info } from 'lucide-react';
import type { ReactNode } from 'react';

interface HintIconProps {
  text: ReactNode;
  size?: number;
}

export function HintIcon({ text, size = 14 }: HintIconProps) {
  return (
    <Tooltip.Provider delayDuration={100}>
      <Tooltip.Root>
        <Tooltip.Trigger asChild>
          <Info
            size={size}
            className="mr-1 cursor-help text-(--meta-text) hover:text-(--text)"
          />
        </Tooltip.Trigger>
        <Tooltip.Portal>
          <Tooltip.Content
            side="right"
            className="z-50 max-w-[350px] rounded-lg border border-(--border) bg-(--tooltip-bg) px-2 py-1 text-(--text) text-sm shadow-md"
          >
            {text}
            <Tooltip.Arrow className="fill-(--tooltip-bg)" />
          </Tooltip.Content>
        </Tooltip.Portal>
      </Tooltip.Root>
    </Tooltip.Provider>
  );
}
