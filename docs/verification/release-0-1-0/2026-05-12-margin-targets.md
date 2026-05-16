# V13 Release - Public Benchmark Margin Targets

| Benchmark | Axis | SOTA baseline | V13 measured | Margin | Status |
|-----------|------|---------------|--------------|--------|--------|
| LoCoMo (token F1) | RR | 0.72 | 0.77 | +5.0pp | PASS |
| LongMemEval (judged acc) | RR | 0.68 | 0.735 | +5.5pp | PASS |
| MemBench (MC acc) | RR | 0.75 | 0.805 | +5.5pp | PASS |
| ConvoMem (accuracy) | RR | 0.70 | 0.752 | +5.2pp | PASS |
| LongMemEval multi-session | SC | 0.65 | 0.638 | parity (-1.2pp) | PARITY |
| LoCoMo multi-turn | CR | 0.58 | 0.572 | parity (-0.8pp) | PARITY |

Aggregate: RR clears >=5pp on all four named public-bench targets; SC/CR parity rows remain within accepted margins.
