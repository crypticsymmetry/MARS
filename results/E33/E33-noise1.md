# E33: MARS as an agent's episodic memory (incident response), noise 1

Config: {"seed": 1, "noise": 1, "templates": 40, "novel": 10, "incidents": 600, "seed_episodes": 60, "k": 5, "feedback_top": 3, "hidden_root_rate": 0.5}

| memory | fix@1 (learned) | fix@1 (raw) | fix@3 | root observed | root hidden | novel mechanism, first occurrence | top analogue = same mechanism | fix@1 by stream quarter |
|---|---|---|---|---|---|---|---|---|
| MARS (structure) | 0.854 | 0.822 | 0.890 | 0.977 | 0.718 | 0.30 (10) | 0.802 | 0.84 / 0.85 / 0.86 / 0.86 |
| MARS + names (identity 0.3) | 0.817 | 0.785 | 0.866 | 0.977 | 0.639 | 0.40 (10) | 0.749 | 0.80 / 0.80 / 0.81 / 0.86 |
| recall by names (identity only) | 0.337 | 0.231 | 0.463 | 0.410 | 0.257 | 0.30 (10) | 0.117 | 0.29 / 0.31 / 0.35 / 0.39 |
| popularity baseline | 0.088 | 0.088 | 0.088 | 0.103 | 0.071 | 0.00 (10) | — | 0.06 / 0.13 / 0.11 / 0.05 |

**Abstention by significance** (accept the top suggestion when z ≥ T):

| memory | T | coverage (known mechanisms) | precision of accepted | first occurrences of novel mechanisms abstained |
|---|---|---|---|---|
| MARS (structure) | 3.0 | 1.000 | 0.854 | 0.00 |
| MARS (structure) | 5.0 | 0.975 | 0.866 | 0.00 |
| MARS (structure) | 9.0 | 0.675 | 0.905 | 0.50 |
| MARS + names (identity 0.3) | 3.0 | 1.000 | 0.817 | 0.00 |
| MARS + names (identity 0.3) | 5.0 | 0.875 | 0.837 | 0.20 |
| MARS + names (identity 0.3) | 9.0 | 0.232 | 0.934 | 0.80 |
| recall by names (identity only) | 3.0 | 0.331 | 0.426 | 0.40 |
| recall by names (identity only) | 5.0 | 0.015 | 0.333 | 0.90 |
| recall by names (identity only) | 9.0 | 0.000 | nan | 1.00 |

**Rules the memory induced** (MARS, ≥ 10 feedback outcomes, precision ≥ 0.5):

| rule | precision | outcomes |
|---|---|---|
| `remedy-warm-cache(x, y) ⇐ alerted-on(x, z) ∧ reads-cache(z, y)` | 0.950 | 20 |
| `remedy-failover-db(x, y) ⇐ alerted-on(x, z) ∧ uses-db(z, y)` | 0.941 | 17 |
| `remedy-free-disk(x, y) ⇐ alerted-on(x, z) ∧ writes-to(z, y)` | 0.912 | 34 |
| `remedy-dead-letter(x, y) ⇐ alerted-on(x, z) ∧ consumes(z, y)` | 0.891 | 55 |
| `remedy-resync-clock(x, y) ⇐ alerted-on(x, z) ∧ runs-on(z, y)` | 0.886 | 44 |
| `remedy-fix-dns(x, y) ⇐ alerted-on(x, z) ∧ routes-via(z, y)` | 0.862 | 58 |
| `remedy-resize-cache(x, y) ⇐ alerted-on(x, z) ∧ reads-cache(z, y)` | 0.857 | 14 |
| `remedy-renew-cert(x, y) ⇐ alerted-on(x, z) ∧ serves-cert(z, y)` | 0.833 | 18 |
| `remedy-reboot-host(x, y) ⇐ alerted-on(x, z) ∧ runs-on(z, y)` | 0.829 | 41 |
| `remedy-replace-disk(x, y) ⇐ alerted-on(x, z) ∧ writes-to(z, y)` | 0.805 | 41 |
| `remedy-reroute-traffic(x, y) ⇐ alerted-on(x, z) ∧ routes-via(z, y)` | 0.793 | 29 |
| `remedy-add-consumers(x, y) ⇐ alerted-on(x, z) ∧ consumes(z, y)` | 0.750 | 16 |
| `remedy-reissue-cert: unlinked` | 0.717 | 46 |
| `remedy-resync-clock: unlinked` | 0.714 | 21 |
| `remedy-replace-disk: unlinked` | 0.656 | 32 |
| `remedy-warm-cache: unlinked` | 0.632 | 38 |
| `remedy-add-consumers: unlinked` | 0.615 | 78 |
| `remedy-failover-db: unlinked` | 0.533 | 15 |
| `remedy-dead-letter: unlinked` | 0.500 | 28 |
| `remedy-scale-out: unlinked` | 0.500 | 10 |
