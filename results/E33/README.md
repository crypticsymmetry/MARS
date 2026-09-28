# E33: MARS as an agent's episodic memory (incident response)

Script: [`tools/e33_agent_memory.py`](../../tools/e33_agent_memory.py), which uses the Python bindings. Tables: `E33-noise{0,1,2,3}.md` / `.json`; LLM comparison: `E33-llm-noise2.json`. Every file records its config and seed.

Reproduce:
```
python3 tools/e33_agent_memory.py results/E33 --noise 2          # --noise 0..3
python3 tools/e33_agent_memory.py results/E33 --llm 150 --noise 2  # LLM agent baselines (GLM-5.3-Flash via OpenRouter; $OPENROUTER_API_KEY)
python3 tools/e33_agent_memory.py --demo                          # narrated walk-through
```

**Question:** Does MARS work as the episodic memory of an agent, end to end through the Python API? The agent stores experiences, recalls analogous ones for a new situation, infers what to do, abstains when nothing is analogous, and learns from outcomes. How does it compare with memory recalled by names and with an LLM agent given retrieved memories?

**Task: incident response.**
- **Episodes.** 40 failure mechanisms, each a root cause on a resource propagating through a chain of service symptoms. Resources: disk, database, certificate, queue, cache, network, host. Example: `disk-full(disk) → write-error(svc) → crash(svc)`, possibly spreading to a dependent service.
- **Fix.** Set by the root cause and applied to the root entity, e.g. `(remedy-free-disk INC disk-3)`. 20 remedies in total. Getting it right requires the right kind *and* the right target entity.
- **Surface overlap.** Service, host and resource names recur across *different* mechanisms, so recall by names retrieves incidents on the same services that failed differently.
- **Hidden root cause.** In half the incidents the root-cause event and its causal link are unobserved.
- **Noise level s.** s concurrent unrelated anomalies are added (a root event and first symptom on other entities, not alerted), and s observed facts are dropped.
- **Stream.** Memory starts with 2 resolved episodes of each of 30 mechanisms. 10 mechanisms are *novel*: they first appear in the stream, with no precedent. 600 incidents follow, seed 1. For each incident the agent:
  1. queries the memory (significance z);
  2. asks for suggestions (`suggest`, top-5 analogues);
  3. applies the top remedy;
  4. gives feedback on the top-3 remedy suggestions (`feedback`);
  5. stores the resolved episode (`add_case`).

**Memories compared** (same stream; all but popularity use the same mapper and feedback):
- **MARS:** structural fingerprints (analogy profile) + FAC.
- **MARS + names:** identity channel at λ = 0.3.
- **Recall by names:** identity channel only, λ = 1, so retrieval is by entity and predicate names.
- **Popularity:** the most frequent remedy kind, on the incident's first entity of that resource type.

## Results (fix accuracy, known mechanisms)

| noise | MARS: fix@1 (learned / raw) | fix@3 | root observed / hidden | MARS + names | recall by names | popularity |
|---|---|---|---|---|---|---|
| 0 | **0.995** / 0.995 | 0.997 | 1.000 / 0.990 | 0.990 | 0.475 | 0.086 |
| 1 | **0.854** / 0.822 | 0.890 | 0.977 / 0.718 | 0.817 | 0.337 | 0.088 |
| 2 | **0.761** / 0.732 | 0.836 | 0.954 / 0.587 | 0.722 | 0.315 | 0.078 |
| 3 | **0.646** / 0.585 | 0.731 | 0.914 / 0.422 | 0.553 | 0.231 | 0.068 |

- **MARS retrieves the right mechanism far more often than recall by names.** The top analogue is an incident of the same mechanism 100% of the time at noise 0 and 65% at noise 2, against 17% and 10% for recall by names.
- **Novel mechanisms, first occurrence** (10 per run): MARS still gets the fix right 30–40% of the time at noise 0–2, by transferring from mechanisms that share the root cause. Recall by names manages 10%.

**Abstention by significance** (MARS; accept the top suggestion when z ≥ T):

| noise | T = 5: coverage / precision / novel abstained | T = 9: coverage / precision / novel abstained |
|---|---|---|
| 0 | 1.00 / 0.995 / 0.10 | 0.97 / 0.995 / **0.80** |
| 1 | 0.97 / 0.866 / 0.00 | 0.67 / 0.905 / 0.50 |
| 2 | 0.96 / 0.762 / 0.00 | 0.46 / 0.822 / 0.60 |
| 3 | 0.96 / 0.653 / 0.10 | 0.41 / 0.711 / 0.80 |

**Rules the memory induced** from feedback (noise 0, selected):

| rule | precision | outcomes |
|---|---|---|
| `remedy-dead-letter(x, y) ⇐ alerted-on(x, z) ∧ consumes(z, y)` | 0.98 | 41 |
| `remedy-reroute-traffic(x, y) ⇐ alerted-on(x, z) ∧ routes-via(z, y)` | 0.98 | 42 |
| `remedy-resync-clock(x, y) ⇐ alerted-on(x, z) ∧ runs-on(z, y)` | 0.91 | 57 |
| `remedy-replace-disk(x, y) ⇐ alerted-on(x, z) ∧ writes-to(z, y)` | 0.89 | 37 |

That is, "apply the fix to the resource the alerted service uses".

**LLM agents with retrieved memory** (noise 2; every 4th incident, 150 in total; GLM-5.3-Flash, reasoning low, temperature 0). The LLM sees the top-5 retrieved past episodes (facts plus remedy) and the new incident, and must answer `(remedy-KIND INC TARGET)`. 300 calls, $0.30, no failures.

| agent (same 150 incidents) | fix@1 |
|---|---|
| LLM + episodes recalled by names | 0.347 |
| LLM + episodes retrieved by MARS | 0.660 |
| **MARS alone** (mapping + candidate inference + learned reliability) | **0.780** |

## Findings

1. **MARS works as an agent's episodic memory, end to end through the Python API.** On clean incidents it picks the right remedy and target 99.5% of the time, including 99% when the root cause is unobserved: the analogy supplies the missing cause and the fix together, as the demo shows. It degrades gracefully with concurrent anomalies and missing facts (0.85 / 0.76 / 0.65 at noise 1–3).
2. **Structure, not names, is what finds the right past incident.** Recall by names reaches 0.23–0.48, because the same services fail in different ways. Adding names to structure (λ = 0.3) *hurts* here (0.72 vs 0.76 at noise 2), the opposite of the KG entities in E32, where shared names are informative. Whether names help depends on whether they identify the *kind* of case or just the *place*.
3. **Learning from outcomes matters more as the task gets harder.** Learned vs raw ranking: +0.000, +0.032, +0.029 and +0.061 at noise 0–3. The induced rules are readable remediation policies.
4. **Abstention needs care under noise.**
   - On clean data, z ≥ 9 abstains on 80% of first occurrences of novel mechanisms while keeping 97% coverage at 0.995 precision.
   - Under noise, fewer known incidents reach the threshold: coverage falls to 41–67%, though precision of accepted answers rises (0.76 → 0.82 at noise 2).
   - Some novel first occurrences are *solvable* by cross-mechanism transfer (30–40%), so abstaining on them has a cost.
   - E16's threshold was calibrated for synthetic templates at scale. An agent should tune T to its cost of a wrong action.
5. **MARS is a better memory for an LLM agent, and on structural tasks a better reasoner too.** Given episodes retrieved by MARS instead of by names, the LLM agent nearly doubles its accuracy (0.35 → 0.66): retrieval quality dominates. MARS's own mapping and inference, with learned reliability, is better still (0.78) at a fraction of the cost and latency (milliseconds per incident, no API). This is a small, cheap model at low reasoning effort; a stronger LLM would likely close part of the gap. Either way, MARS is the right *retriever* for such an agent, and its explicit inferences come with provenance.
6. **Honest scope.** The mechanisms, noise and names are synthetic; real incident data needs a front end (logs and postmortems to facts, e.g. with `llm2mars.py`). The task is designed to be structural, which is where MARS is strong.
