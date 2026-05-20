interface FilterGroupProps<T extends string> {
  label: string;
  options: T[];
  selected: T[];
  onChange: (value: T[]) => void;
}

export function FilterGroup<T extends string>({
  label,
  options,
  selected,
  onChange,
}: FilterGroupProps<T>) {
  const allSelected = selected.length === options.length;

  return (
    <div className="space-y-1.5">
      <div className="flex items-center gap-2">
        <span className="text-[11px] tracking-wide uppercase text-text-tertiary">
          {label}
        </span>
        <button
          onClick={() => onChange(allSelected ? [] : [...options])}
          className="text-[10px] text-accent-bright hover:opacity-80 transition-opacity"
        >
          {allSelected ? "none" : "all"}
        </button>
      </div>
      <div className="flex flex-wrap gap-1">
        {options.map((option) => {
          const active = selected.includes(option);
          return (
            <button
              key={option}
              onClick={() =>
                onChange(
                  active
                    ? selected.filter(
                        (selectedOption) => selectedOption !== option,
                      )
                    : [...selected, option],
                )
              }
              className={`px-2 py-0.5 rounded text-[11px] border transition-colors ${
                active
                  ? "bg-accent-primary/15 text-accent-bright border-accent-primary/40"
                  : "bg-glass text-text-tertiary border-border-subtle hover:border-border-active"
              }`}
            >
              {option.replace(/_/g, " ")}
            </button>
          );
        })}
      </div>
    </div>
  );
}
