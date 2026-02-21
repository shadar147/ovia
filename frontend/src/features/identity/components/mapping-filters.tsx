"use client";

import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Slider } from "@/components/ui/slider";
import { Button } from "@/components/ui/button";
import type { LinkStatus } from "@/lib/api/types";

export interface FilterValues {
  status: LinkStatus | "all";
  minConfidence: number;
  maxConfidence: number;
}

interface MappingFiltersProps {
  value: FilterValues;
  onChange: (v: FilterValues) => void;
  onReset: () => void;
}

export const defaultFilters: FilterValues = {
  status: "all",
  minConfidence: 0,
  maxConfidence: 100,
};

export function MappingFilters({ value, onChange, onReset }: MappingFiltersProps) {
  return (
    <div className="flex flex-wrap items-end gap-4">
      <div className="space-y-1">
        <label className="text-xs font-medium text-muted-foreground">Status</label>
        <Select
          value={value.status}
          onValueChange={(v) => onChange({ ...value, status: v as LinkStatus | "all" })}
        >
          <SelectTrigger className="w-36">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="all">All</SelectItem>
            <SelectItem value="auto">Auto</SelectItem>
            <SelectItem value="verified">Verified</SelectItem>
            <SelectItem value="conflict">Conflict</SelectItem>
            <SelectItem value="rejected">Rejected</SelectItem>
            <SelectItem value="split">Split</SelectItem>
          </SelectContent>
        </Select>
      </div>

      <div className="space-y-1 w-48">
        <label className="text-xs font-medium text-muted-foreground">
          Confidence: {value.minConfidence}% – {value.maxConfidence}%
        </label>
        <Slider
          min={0}
          max={100}
          step={5}
          value={[value.minConfidence, value.maxConfidence]}
          onValueChange={([min, max]) => {
            if (min !== undefined && max !== undefined) {
              onChange({ ...value, minConfidence: min, maxConfidence: max });
            }
          }}
        />
      </div>

      <Button variant="ghost" size="sm" onClick={onReset}>
        Reset
      </Button>
    </div>
  );
}
