# Challenges, corruptions & campaign

Challenges are self-imposed restrictions that unlock features and (for 11–15) feed **ascension score**.
**Corruptions** are score-multiplying difficulty modifiers; **campaign** picks corruption loadouts and
grants bonus multipliers; **constant upgrades** are a small flat boost layer. Source: `Challenges.ts`,
`Corruptions.ts`, `Campaign.ts`, score arrays in `Calculate.ts:1174-1195`.

## Diagram

```mermaid
flowchart LR
  classDef ported fill:#2e7d32,color:#fff,stroke:#1b5e20;
  classDef partial fill:#f9a825,color:#000,stroke:#f57f17;
  classDef bug fill:#f9a825,color:#000,stroke:#d50000,stroke-width:3px;
  classDef ext fill:#eceff1,color:#37474f,stroke:#90a4ae,stroke-dasharray:4 3;

  chPT["Challenges 1–5 · transcend"]:::ported
  chReinc["Challenges 6–10 · reincarnation"]:::ported
  chAsc["Challenges 11–14 · ascension"]:::ported
  ch15["Challenge 15 exponent"]:::ported
  autoChal["Auto-challenge sweep"]:::ported
  corruptions["Corruptions ·8 · loadouts"]:::ported
  campaign["Campaign · tokens"]:::partial
  constUpg["Constant upgrades"]:::ported

  reinc["Reincarnation ↗ reset-cascade"]:::ext
  asc["Ascension ↗ ascension-cubes"]:::ext
  score["Ascension score ↗ ascension-cubes"]:::ext
  hept["Hepteracts ↗ ascension-cubes"]:::ext
  obt["Obtainium ↗ core-economy"]:::ext
  cubes["Cubes/offerings ↗ ascension-cubes"]:::ext
  antProd["Ant production ↗ ants"]:::ext

  reinc -->|"unlock"| chPT
  reinc -->|"unlock"| chReinc
  asc -->|"unlock"| chAsc
  chReinc -->|"c10 unlocks"| asc
  chAsc -->|"ECC base"| score
  ch15 -->|"+ score"| score
  ch15 -->|"unlock"| hept
  autoChal -->|"run"| chPT
  autoChal -->|"run"| chReinc
  corruptions -->|"× multiplier"| score
  corruptions -->|"DR"| obt
  corruptions -->|"deflation"| antProd
  campaign -->|"set loadout"| corruptions
  campaign -->|"bonus"| cubes
  campaign -->|"× score"| score
  constUpg -->|"boost"| obt
```

## How it connects

- **Out:** challenges 11–15 and corruptions are the dominant inputs to **ascension score**
  ([ascension-cubes](ascension-cubes.md)); C15 also unlocks hepteracts; the c10 completion is what
  unlocks ascension itself.
- **In:** reincarnation/ascension resets unlock the challenge ladders.

## Port status

| System | Status | Rust |
|---|---|---|
| Challenges 1–14 | 🟩 Ported | `mechanics/challenges.rs`, `tick/mod.rs:5186-5302` |
| Challenge 15 exponent | 🟩 Ported | `mechanics/challenge_15_rewards.rs`; accrual at `tick/mod.rs:5396` (PR #265) |
| Corruptions (8 effects) | 🟩 Ported | `state/corruptions.rs`, `mechanics/corruptions.rs` |
| Auto-challenge sweep | 🟩 Ported | `tick/challenge_sweep.rs` |
| Campaign tokens / constant upgrades | 🟩 Mostly | `state/campaigns.rs` (50 campaigns), `mechanics/campaign_token_rewards.rs`, `compute_campaign_tokens` (tick) — the campaign *runner* (UI) remains |

## Porting notes / open bugs

- **P1.4 — C15 exponent accrual: fixed (PR #265).** The exponent is now written at `tick/mod.rs:5396`
  (commit `ff7683e9`), so the C15 reward cascade lights up and hepteracts are reachable via the C15
  path. (It was open at this map's first draft, cut before #265 merged.)
- Audit **C1** (global-speed mult dropped in c1) and **C2** (c10→ascension unlock dead) are **fixed**.
- ✅ **Campaign tokens — derived + all 14 reward consumers fed.** `compute_campaign_tokens` (tick)
  ports `updateTokens()` as a pure derivation over the 50-campaign limit/isMeta table
  (`CAMPAIGNS_LEN` 10→50, a latent sizing bug), inheritance + the GQ/octeract bonus-token upgrades.
  Every `campaign_token_rewards` formula now feeds from it, and the `campaignTokens` achievement
  group (426–435) awards in the per-tick sweep. Tokens flow from `highestSingularityCount ≥ 5`
  without the UI-tier campaign runner (which writes `campaign_completions` once it exists).
