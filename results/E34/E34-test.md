# E34: dataflow front end (py2pdg) for code retrieval

5982 programs from 200 CodeNet Python800 problems (seed 1, exclude-seed 0; the program set is py2mars's sample; py2pdg re-encodes the same programs, 1 of them skipped). Relevant = other solutions of the same problem.

| method | MAP@R [95% CI] | Hits@1 [95% CI] | MRR [95% CI] |
|---|---|---|---|
| MARS fused, py2mars | 0.2110 [0.2060, 0.2159] | 0.6993 [0.6876, 0.7108] | 0.7608 [0.7510, 0.7703] |
| MARS fused, py2pdg | 0.3189 [0.3125, 0.3251] | 0.8096 [0.7996, 0.8195] | 0.8550 [0.8472, 0.8627] |
| MARS FP-literal, py2mars | 0.1663 [0.1617, 0.1708] | 0.6092 [0.5968, 0.6214] | 0.6887 [0.6784, 0.6988] |
| MARS FP-literal, py2pdg | 0.2520 [0.2463, 0.2577] | 0.7252 [0.7138, 0.7365] | 0.7862 [0.7771, 0.7953] |
| code embedding | 0.4731 [0.4659, 0.4804] | 0.8561 [0.8470, 0.8648] | 0.8915 [0.8845, 0.8983] |
| embedding top-100 re-ranked by ½cos + ½FAC (py2pdg) | 0.4975 [0.4907, 0.5043] | 0.8920 [0.8842, 0.9000] | 0.9186 [0.9125, 0.9247] |

**Paired comparisons** (MAP@R, paired over queries):

| comparison | A − B [95% CI] | p |
|---|---|---|
| A1 (P10): MARS fused, py2pdg − MARS fused, py2mars | +0.1079 [+0.1034, +0.1122] | 0.0001 |
| A1-FP: MARS FP-literal, py2pdg − MARS FP-literal, py2mars | +0.0857 [+0.0817, +0.0899] | 0.0001 |
| P11: MARS fused, py2pdg − code embedding | -0.1543 [-0.1616, -0.1471] | 0.0001 |
| A2 (P12): embedding top-100 re-ranked by ½cos + ½FAC (py2pdg) − code embedding | +0.0244 [+0.0204, +0.0284] | 0.0001 |
