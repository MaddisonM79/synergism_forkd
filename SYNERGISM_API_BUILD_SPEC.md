<!--
Build spec for the Synergism Forkd backend, to be implemented as a new app inside ~/github/lmam_api (api.lmam.tech).
Authored 2026-06-06 in the synergism_forkd repo for review. DESTINATION: copy into lmam_api (suggest docs/synergism-api-spec.md) when implementation starts.
Companion analysis (contract derivation + evidence): repo-root BACKEND_API_PLAN.md in synergism_forkd.
This is a spec, not code. Another session implements from it.
-->

# Synergism Forkd Backend — Build Spec

**Audience:** an implementing session working in `~/github/lmam_api`. This document is self-contained — you should not need to read the legacy game to build Phases 1–3. Where the legacy contract matters, it's quoted here. Deeper derivation + evidence lives in `synergism_forkd/BACKEND_API_PLAN.md`.

**What you're building:** a new app, `synergism`, inside the `lmam_api` Cloudflare Worker monorepo, that serves the Synergism Forkd game client. The game is a Rust/Dioxus/WASM client (a fork of the TS game "Synergism", on a deliberately divergent path). The game is fully playable offline; this backend is **additive** (cloud saves, live events, news, and a reserved economy/store seam).

---

## 0. Locked decisions (read first — these are settled, don't re-litigate)

1. **The Rust client is the only consumer.** There is no obligation to match the legacy TS wire format. Design clean. All routes mount under `/api/synergism/*` (plus a small public subtree, see §3.1).
2. **Auth = Hanko bearer** (the platform default). Do **not** build cookie sessions, OAuth-as-login, `/register`/`/signin`/`/forgot-password`, or Turnstile — Hanko owns identity. Discord/Patreon linking and role perks are **out of scope** for now (reserved for later).
3. **Flat account to start.** The profile is "a logged-in user with a display name and save quota." `membershipTier` and `quarkBonus` exist as **dormant, default-0 fields** reserved for when memberships return; nothing sets them yet.
4. **Real-money transactions (RMT) are parked, not dropped.** Build an **inert store seam**: the `store` endpoints exist, return empty/disabled, and are the clean place to wire the fork's **own** processor and **own** items later. Use **none** of the upstream's Stripe/PayPal/NOWPayments/Fourthwall accounts, the upstream PayPal clientId, the upstream PseudoCoin item catalog, or the upstream Discord role IDs.
5. **Divergent fork, not a re-host.** Favor clean, extensible design over fidelity to the legacy API. The legacy endpoints are a **feature checklist**, not a wire spec.
6. **Fresh save format.** No compatibility with the legacy save blob codec (`base64(gzip(...))`, ASCII-only, delete-by-name). Design saves natively.

---

## 1. Platform & conventions (lmam_api)

You are extending an existing monorepo. Mirror its patterns exactly; the lint config enforces the import boundaries.

- **Runtime:** one Cloudflare Worker (`apps/api`), **Hono** routing, deployed via `wrangler`.
- **Data:** **Prisma 7 → Neon Postgres, one Neon project per app.** Synergism gets its **own** Neon project. Pooled URL for runtime, direct URL for migrations.
- **Auth:** **Hanko** JWT validated via JWKS in `api-core` → a `Principal`. Authenticated routes under `/api/*` expect `Authorization: Bearer <jwt>`. `Principal` carries the Hanko subject (use it as the stable user id).
- **Layering (lint-enforced):**

  | Package | May import | Must never import |
  |---|---|---|
  | `@lmam/api-core` | external only | any app package |
  | `@lmam/synergism-db` | external only | another app's anything |
  | `@lmam/synergism-service` | its own `synergism-db`, sibling `*-service`, `api-core` | another app's `*-db` |
  | `apps/api/src/routes/synergism.ts` | `@lmam/synergism-service`, `api-core` | any `*-db` |

- **Authorization + Zod input validation live in the service layer**, never in routes/middleware. Every service fn is `(ctx: Ctx, input) => ...` and self-enforces authz against `ctx.principal`.
- **Env:** single root `.env`, per-app short prefix. **Synergism prefix = `SYN_`.** Shared/core config is `API_`-prefixed (e.g. `API_HANKO_JWKS_URL`).
- **Copy-from sources:** `packages/template-db` and `packages/template-service` are the worked examples to clone. `apps/api/src/do/CampaignSession.ts` is the Durable Object pattern. `@aws-sdk/client-s3` + `s3-request-presigner` are already deps (use for save blobs).

### Packages/files to create
```
packages/synergism-db/         # Prisma schema (§4) + Neon client factory   (clone template-db)
packages/synergism-service/    # business logic: authz + Zod (§5)           (clone template-service)
apps/api/src/routes/synergism.ts          # Hono sub-router (§3), mounted in apps/api/src/index.ts
apps/api/src/do/ConsumablesHub.ts         # Durable Object (Phase 4 only — defer)
```

### Env vars to add (root `.env` + `.env.example` + `apps/api/src/env.ts` + wrangler bindings)
```
SYN_DATABASE_URL            # Neon pooled (runtime)
SYN_DATABASE_URL_UNPOOLED   # Neon direct (migrations)
SYN_SAVES_BUCKET            # S3/R2 bucket name for save blobs
SYN_SAVES_S3_ENDPOINT       # if using R2 via S3 API (else AWS region/creds per existing convention)
# (reuse existing S3 credential convention already used by apps/api; do NOT invent a new auth scheme)
SYN_CONSUMABLES             # Durable Object binding (Phase 4 only)
```

---

## 2. Build phases (sequence + acceptance)

Build in order. Each phase is independently shippable. **Phases 1–3 are the near-term, fully-specified work. Phases 4–5 are design-reserved** (sketched contract + open questions; do not implement until §6 questions are answered).

| Phase | Scope | Auth | New infra | Acceptance |
|---|---|---|---|---|
| **1** | Live services: events, quark-bonus, contributors | public | Neon project, `synergism-db`, `synergism-service`, route mounted | `GET /api/synergism/events/active` etc. return correct JSON; admin can create an event row and it goes active in its window |
| **2** | Identity (Hanko) + cloud saves | bearer | S3 bucket, presigned flow | A logged-in client can list/upload/download/delete saves; profile auto-provisions; quota enforced |
| **3** | Messages / news | bearer | — | Unread list (active, per-user) + mark-read idempotent |
| **4** *(reserved)* | Economy: wallet, upgrade shop, consumables + WS | bearer | `ConsumablesHub` DO | *design-gated — see §6 Q-A/Q-B* |
| **5** *(reserved)* | RMT: real items + processor | bearer | webhooks | *parked — inert seam only until the fork decides* |

---

## 3. HTTP API surface

Base: `https://api.lmam.tech`. All JSON. Errors use the platform's existing error envelope (mirror `api-core`); the shapes below are the success bodies. Validation failures → `400`; missing/invalid bearer on an authed route → `401`; not-found → `404`; disabled/reserved → `501`.

### 3.1 Public live services (Phase 1) — no auth

These must be reachable **without** a bearer token. The platform applies `hankoAuth` to `/api/*`. Choose one (recommend **A**) and note it in the PR:
- **A (recommended):** make the Synergism live routes use an *optional-auth* variant — attach `Principal` if a valid token is present, but do not `401` when absent. Keeps the clean `/api/synergism/*` namespace.
- **B:** mount these under a non-`/api` public prefix (e.g. `/synergism/public/*`).

| Method | Path | Response | Notes |
|---|---|---|---|
| GET | `/api/synergism/events/active` | `{ event: GameEvent \| null }` | The single currently-active event, or null. "Active" = `now ∈ [startsAt, endsAt)` and `enabled`. |
| GET | `/api/synergism/quark-bonus` | `{ bonus: number }` | Global quark bonus **percentage** (e.g. `50` = +50%). For now: a single admin-set `SiteConfig` value (later: computed from active events/consumables). |
| GET | `/api/synergism/contributors` | `{ contributors: { login: string; avatarUrl: string }[]; artists: string[] }` | Server-side cached GitHub contributors fetch (cache ≥1h; no per-request GitHub call). `artists` is an admin-maintained list. |

`GameEvent` (response shape):
```ts
type GameEvent = {
  id: string
  name: string                      // display name (single string; legacy used string[] for i18n — drop that)
  startsAt: string                  // ISO-8601
  endsAt: string                    // ISO-8601
  color: string | null              // CSS color for the banner, optional
  url: string | null                // announcement link, optional
  buffs: {                          // additive fractions; 0.25 = +25%. Omitted keys default to 0.
    quark?: number; goldenQuark?: number; cubes?: number; powderConversion?: number
    ascensionSpeed?: number; globalSpeed?: number; ascensionScore?: number; antSacrifice?: number
    offering?: number; obtainium?: number; octeract?: number; blueberryTime?: number
    ambrosiaLuck?: number; oneMind?: number
  }
}
```
> The 14 buff keys mirror the game's `BuffType` and the Rust `Event` port — keep these names. The client treats missing buffs as 0 and an absent event as "no event," so failing soft is correct.

*(Translations: out of API scope. The client loads `translations/{lang}.json` as static assets; host them on R2/Pages, not through this Worker.)*

### 3.2 Identity (Phase 2) — bearer

Profile is auto-provisioned on first authenticated request (upsert by Hanko subject).

| Method | Path | Request | Response |
|---|---|---|---|
| GET | `/api/synergism/me` | — | `Profile` |
| PATCH | `/api/synergism/me` | `{ displayName?: string }` (1–32 chars, sanitized) | `Profile` |

```ts
type Profile = {
  userId: string                    // Hanko subject
  displayName: string | null
  membershipTier: number            // DORMANT — always 0 for now (reserved)
  quarkBonus: number                // DORMANT — always 0 for now (reserved, personal quark %)
  linkedAccounts: string[]          // DORMANT — always [] for now (reserved: discord/patreon)
  saveQuota: { used: number; max: number; maxBytes: number }
}
```

### 3.3 Cloud saves (Phase 2) — bearer

Saves are user-scoped named slots. Blob goes to S3/R2 via **presigned URLs** (keep the Worker out of the blob path); metadata in Neon. Default quota (flat, tier-less for now): **`max = 5` slots, `maxBytes = 5 MiB`/slot**. `membershipTier` is the reserved lever to raise these later.

| Method | Path | Request | Response | Notes |
|---|---|---|---|---|
| GET | `/api/synergism/saves` | — | `{ saves: SaveMeta[] }` | metadata only, no blob |
| POST | `/api/synergism/saves` | `{ name: string; sizeBytes: number }` | `{ id: string; uploadUrl: string; expiresAt: string }` | reserves a slot (or replaces same-`name`), returns a presigned **PUT**; enforces quota + size here |
| POST | `/api/synergism/saves/:id/commit` | — | `SaveMeta` | call after the client PUTs the blob; server HEADs the object, records size/`uploadedAt` |
| GET | `/api/synergism/saves/:id` | — | `SaveMeta & { downloadUrl: string; expiresAt: string }` | presigned **GET** |
| DELETE | `/api/synergism/saves/:id` | — | `204` | deletes row + S3 object |

```ts
type SaveMeta = { id: string; name: string; sizeBytes: number; uploadedAt: string; updatedAt: string }
```
- `(userId, name)` is unique → re-uploading the same `name` replaces that slot (preserves the legacy "named slot" feel without the delete-by-name quirk).
- The blob is **opaque** to the server (the Rust client's native save format). Server stores bytes; never parses them.
- **Simpler inline alternative** (acceptable for an MVP if presigning is fiddly): `POST /api/synergism/saves` takes the blob in the body (base64 or octet-stream), server streams to S3, returns `SaveMeta`. Pick one; note it in the PR. Presigned is preferred for scale.

### 3.4 Messages / news (Phase 3) — bearer

| Method | Path | Request | Response | Notes |
|---|---|---|---|---|
| GET | `/api/synergism/messages` | — | `{ messages: Message[] }` | active + unread-for-user, sorted `priority` desc |
| POST | `/api/synergism/messages/:id/read` | — | `{ ok: true }` | idempotent; records `(userId, messageId)` |

```ts
type Message = {
  id: string; title: string; content: string
  type: 'info' | 'warning' | 'error' | 'success'
  priority: number
  createdAt: string; expiresAt: string | null
}
```
Server filters out inactive, expired, and already-read messages. Authoring messages is an admin/seed concern (no public write endpoint).

### 3.5 Economy (Phase 4 — RESERVED, design-gated)

Contract sketch only; **do not implement until §6 Q-A and Q-B are answered.** All bearer. This is the fork's **own** economy (own items), not the upstream catalog.

| Method | Path | Response (sketch) |
|---|---|---|
| GET | `/api/synergism/wallet` | `{ coins: number }` |
| GET | `/api/synergism/shop/upgrades` | `{ coins, upgrades: UpgradeView[] }` (catalog + player level) |
| POST | `/api/synergism/shop/upgrades/:id/buy` | `{ upgradeId, level, coins }` (spend coins) |
| GET | `/api/synergism/shop/consumables` | `{ consumables: ConsumableView[] }` (fork's own catalog) |
| WS | `/api/synergism/consumables` | Durable Object channel (see §3.7) |

### 3.6 Store / RMT seam (Phase 5 — INERT, build the stub now)

Build these as a visible-but-disabled seam in Phase 2 or 3 (cheap), so the client can render a "store" surface that shows nothing yet.

| Method | Path | Response now | Later |
|---|---|---|---|
| GET | `/api/synergism/store/products` | `{ enabled: false, products: [] }` | the fork's own items |
| POST | `/api/synergism/store/checkout` | `501 { error: 'store_disabled' }` | processor checkout |
| POST | `/api/synergism/store/webhook/:processor` | `501` (reserved path) | processor fulfillment |

### 3.7 Consumables WebSocket / Durable Object (Phase 4 — RESERVED)

When the economy is built, real-time consumables (shared timed buffs, per-user time-skips, per-user inventory) run over a Durable Object (`ConsumablesHub`), mirroring the `CampaignSession` DO pattern. **WS auth:** browsers can't set headers on the WS upgrade — pass the Hanko JWT as a query param (`?token=`) or subprotocol and validate it in the DO/Worker before accepting the socket. This is the fork's own protocol; the legacy 19-message protocol (`consumed`/`time-skip`/`lotus`/`tips`/…) in `BACKEND_API_PLAN.md §5` is **reference**, redesign to fit the fork's items. Defer entirely until Phase 4 is greenlit.

---

## 4. Data model (Prisma — `packages/synergism-db`)

Phase 1–3 tables are concrete. Phase 4–5 tables are reserved (create when those phases start). Use the platform's id convention (cuid/uuid). `userId` = Hanko subject (string).

```prisma
// ---- Phase 1 ----
model SiteConfig {                 // singleton (id = "default")
  id               String  @id @default("default")
  globalQuarkBonus Float   @default(0)   // percentage
  artists          String[]               // contributor "artists" list
  updatedAt        DateTime @updatedAt
}

model GameEvent {
  id        String   @id @default(cuid())
  name      String
  startsAt  DateTime
  endsAt    DateTime
  color     String?
  url       String?
  buffs     Json                            // { quark?:number, ... } — the 14 additive-fraction keys
  enabled   Boolean  @default(true)
  createdAt DateTime @default(now())
  @@index([enabled, startsAt, endsAt])
}

// contributors: cache the GitHub fetch (KV, DO storage, or a ContributorCache singleton row + TTL). No per-request GitHub call.

// ---- Phase 2 ----
model Profile {
  userId         String   @id            // Hanko subject
  displayName    String?
  membershipTier Int      @default(0)    // DORMANT
  quarkBonus     Float    @default(0)    // DORMANT
  linkedAccounts Json     @default("[]") // DORMANT
  createdAt      DateTime @default(now())
  updatedAt      DateTime @updatedAt
  saves          Save[]
}

model Save {
  id         String   @id @default(cuid())
  userId     String
  name       String
  blobKey    String                       // S3/R2 object key
  sizeBytes  Int      @default(0)
  uploadedAt DateTime @default(now())
  updatedAt  DateTime @updatedAt
  profile    Profile  @relation(fields: [userId], references: [userId], onDelete: Cascade)
  @@unique([userId, name])
  @@index([userId])
}

// ---- Phase 3 ----
model Message {
  id        String   @id @default(cuid())
  title     String
  content   String
  type      String                         // 'info'|'warning'|'error'|'success' (enum or checked string)
  priority  Int      @default(0)
  isActive  Boolean  @default(true)
  createdAt DateTime @default(now())
  expiresAt DateTime?
  reads     MessageRead[]
  @@index([isActive, priority])
}

model MessageRead {
  userId    String
  messageId String
  readAt    DateTime @default(now())
  message   Message  @relation(fields: [messageId], references: [id], onDelete: Cascade)
  @@id([userId, messageId])
}

// ---- Phase 4/5 (RESERVED — create when greenlit) ----
// Wallet(userId @id, coins BigInt @default(0))
// UpgradeCatalog(id, internalName @unique, name, description, maxLevel, costs Json)   // fork's own items
// OwnedUpgrade(userId, upgradeId, level)  @@id([userId, upgradeId])
// ConsumableCatalog(id, internalName @unique, name, description, kind, cost, params Json)
// StoreProduct(id, sku @unique, name, priceCents, enabled Boolean @default(false))    // inert until RMT returns
// Purchase / Entitlement (reserved for processor fulfillment)
```

---

## 5. Service layer (`packages/synergism-service`)

One module per domain (`live`, `profile`, `saves`, `messages`; reserved: `economy`, `store`). Every exported fn takes `(ctx: Ctx, input)`, validates `input` with **Zod**, and enforces authz against `ctx.principal`. Routes are thin: parse params → call service → return JSON.

Representative signatures:
```ts
// live (public — ctx.principal may be undefined)
getActiveEvent(ctx): Promise<{ event: GameEvent | null }>
getQuarkBonus(ctx): Promise<{ bonus: number }>
getContributors(ctx): Promise<{ contributors: {login:string;avatarUrl:string}[]; artists: string[] }>

// profile (authed) — auto-provision on first call
getMe(ctx): Promise<Profile>
updateMe(ctx, { displayName }): Promise<Profile>

// saves (authed) — all scoped to ctx.principal.sub; enforce quota in createSave
listSaves(ctx): Promise<{ saves: SaveMeta[] }>
createSave(ctx, { name, sizeBytes }): Promise<{ id; uploadUrl; expiresAt }>   // checks quota + maxBytes
commitSave(ctx, { id }): Promise<SaveMeta>                                     // HEAD the object, record size
getSave(ctx, { id }): Promise<SaveMeta & { downloadUrl; expiresAt }>           // 404 if not owner
deleteSave(ctx, { id }): Promise<void>                                         // 404 if not owner

// messages (authed)
listUnread(ctx): Promise<{ messages: Message[] }>
markRead(ctx, { id }): Promise<{ ok: true }>
```
**Authz rule for saves/messages:** every row access filters by `ctx.principal.sub`; never trust an id alone. Quota check lives in `createSave`.

---

## 6. Open questions for the implementer (confirm before the relevant phase)

- **Phase 1:** Contributors source — proxy the GitHub repo's contributors API (which repo? the fork's), or an admin-maintained list? Cache location (KV vs DO vs row)?
- **Phase 2:** Presigned-PUT flow vs inline-blob upload (§3.3) — pick one. Confirm quota numbers (5 slots / 5 MiB default). Confirm the S3 vs R2 choice + which credential/binding convention `apps/api` already uses.
- **Q-A (Phase 4 gate):** **How do coins enter the economy?** In this architecture game logic runs client-side (WASM), so server-granted currency is a trust problem. Options: (a) keep the economy **dormant** until RMT returns (coins only ever bought); (b) coins are **earned** via server-validated actions (needs an anti-cheat design); (c) coins are cosmetic/non-competitive so client-claimed amounts are acceptable. **Do not build Phase 4 until this is chosen.**
- **Q-B (Phase 4 gate):** Define the fork's **own** consumable/upgrade catalog (items, effects, costs) and the redesigned WS protocol. Until these exist, Phase 4 has nothing to serve.
- **Phase 5:** Which processor(s), which items, refund/webhook policy — all deferred; only the inert seam (§3.6) ships now.
- **Infra:** Provision the Synergism **Neon project** (+ dev branch) and the **save bucket**; add `SYN_*` to root `.env`/`.env.example`/`env.ts`/wrangler.

---

## 7. Explicitly out of scope / dropped (do NOT build)

- Cookie/session auth, CSRF, `/login?with=…`, `/register`, `/signin`, `/forgot-password`, Cloudflare Turnstile — **Hanko replaces all of this.**
- Discord/Patreon as login providers; Discord-guild **role-based perk gating**; the upstream Discord role IDs.
- Upstream commerce: Stripe Checkout, Stripe billing portal/subscriptions, PayPal orders/subscriptions, NOWPayments, Fourthwall merch — and the upstream's accounts/clientId/catalog. (The fork's own RMT comes later via the §3.6 seam.)
- Legacy save codec (`base64(gzip(...))`, ASCII-only, multipart `file`+`name`, delete-by-`name`) and the `/saves/transfer` "old system" migration.
- The `/api/v1/` prefix and root-mounted legacy paths; legacy response quirks (e.g. the misleading `stripe/` prefix on wallet endpoints, the unused `globalBonus` field).

---

## 8. Definition of done (Phases 1–3)

- `synergism` app exists in the monorepo following the `template-*` structure; lint (import boundaries) passes; `pnpm typecheck` clean.
- Neon project provisioned; Prisma schema migrated; `SYN_*` env wired.
- **Phase 1:** the three public routes return correct JSON; an event row created via seed/admin goes active within its window and feeds `events/active`; `quark-bonus` reflects `SiteConfig`.
- **Phase 2:** a Hanko-authenticated client can round-trip a save (create→PUT→commit→list→download→delete); profile auto-provisions; quota + ownership enforced; `store/products` returns the inert `{enabled:false, products:[]}`.
- **Phase 3:** unread list returns active, non-expired, unread-for-user messages sorted by priority; mark-read is idempotent and per-user.
- A short `README` in `packages/synergism-service` documenting the routes + the dormant/reserved fields.

---

### Reference
- Full legacy contract + decision rationale: `synergism_forkd/BACKEND_API_PLAN.md`.
- Legacy evidence (read-only, in the `synergism_forkd` repo): client call sites `legacy/original/src/{Login,Quark,Event,Messages,Toggles,i18n}.ts` + `legacy/original/src/purchases/*`; MSW mocks `legacy/original/src/mock/**` (the concrete response shapes).
- Platform patterns to clone: `lmam_api/packages/template-{db,service}`, `lmam_api/apps/api/src/do/CampaignSession.ts`, `lmam_api/README.md`.
