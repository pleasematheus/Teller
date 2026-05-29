import { useState } from "react";
import type { DateRange } from "react-day-picker";
import { HugeiconsIcon } from "@hugeicons/react";
import { Calendar03Icon } from "@hugeicons/core-free-icons";
import { Calendar } from "@/components/ui/calendar";
import {
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@/components/ui/popover";

const fmt = new Intl.DateTimeFormat("pt-BR", {
  day: "2-digit",
  month: "short",
  year: "numeric",
});

function formatRange(range: DateRange): string {
  if (!range.from) return "Selecionar período";
  if (!range.to) return fmt.format(range.from);
  return `${fmt.format(range.from)} — ${fmt.format(range.to)}`;
}

type Props = {
  value: DateRange;
  onChange: (range: DateRange) => void;
};

export function DateRangePicker({ value, onChange }: Props) {
  const [open, setOpen] = useState(false);

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <button className="flex items-center gap-2 rounded-md border border-teller-border bg-teller-card px-3 py-1.5 text-sm text-teller-text hover:bg-teller-hover transition-colors">
          <HugeiconsIcon
            icon={Calendar03Icon}
            strokeWidth={1.5}
            className="size-4 text-teller-muted"
          />
          <span className="tabular-nums">{formatRange(value)}</span>
        </button>
      </PopoverTrigger>
      <PopoverContent className="w-auto p-0" align="end">
        <Calendar
          mode="range"
          selected={value}
          onSelect={(range) => {
            if (range) {
              onChange(range);
              if (range.from && range.to) setOpen(false);
            }
          }}
          numberOfMonths={2}
          captionLayout="label"
        />
      </PopoverContent>
    </Popover>
  );
}
