# E34: dataflow front end (py2pdg) for code retrieval

5970 programs from 200 CodeNet Python800 problems (seed 2, exclude-seed 1; the program set is py2mars's sample; py2pdg re-encodes the same programs, 1 of them skipped). Relevant = other solutions of the same problem.

| method | MAP@R [95% CI] | Hits@1 [95% CI] | MRR [95% CI] |
|---|---|---|---|
| MARS fused, py2mars | 0.2403 [0.2349, 0.2458] | 0.7298 [0.7184, 0.7410] | 0.7893 [0.7801, 0.7985] |
| MARS fused, py2pdg | 0.3608 [0.3542, 0.3672] | 0.8444 [0.8350, 0.8538] | 0.8834 [0.8761, 0.8907] |
| MARS FP-literal, py2mars | 0.1955 [0.1904, 0.2006] | 0.6395 [0.6271, 0.6516] | 0.7200 [0.7100, 0.7297] |
| MARS FP-literal, py2pdg | 0.2928 [0.2867, 0.2987] | 0.7628 [0.7521, 0.7735] | 0.8194 [0.8108, 0.8279] |

**Paired comparisons** (MAP@R, paired over queries):

| comparison | A − B [95% CI] | p |
|---|---|---|
| A1 (P10): MARS fused, py2pdg − MARS fused, py2mars | +0.1205 [+0.1160, +0.1249] | 0.0001 |
| A1-FP: MARS FP-literal, py2pdg − MARS FP-literal, py2mars | +0.0973 [+0.0931, +0.1015] | 0.0001 |
