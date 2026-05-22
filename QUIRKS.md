# Synergism Quirks

A catalog of intentionally-preserved oddities in the codebase. These look like bugs at first glance but are *original behavior* — the migration into `@synergism/logic` keeps them byte-for-byte, and parity tests would fail if any of them changed. Document new ones here as they're discovered.

Format: one section per quirk. Include the current source location, what it does, and why it's preserved.

---

## Autobuyer skips the crystal-upgrade cost deduction

**Where**: [packages/logic/src/mechanics/crystalUpgrades.ts](packages/logic/src/mechanics/crystalUpgrades.ts) — `buyCrystalUpgrades`, the `if (toBuy > 0 && !input.auto)` guard.

**What**: When the autobuyer drives the function (`auto: true`), the new crystal-upgrade levels are granted but `prestigeShards` is **not** deducted. Manual clicks deduct as expected.

**Why**: The original code carries this comment:

> Automation no longer spends Crystals. Late game players experience weird 'zeroing' of Crystals when they can afford Crystal Upgrades, due to precision issues. It is easier to just not spend crystals before this becomes a significant issue.

Late-game precision in `break_infinity.js` would otherwise drain `prestigeShards` to zero spuriously. The free-for-autobuyer escape valve avoids a player-visible regression.

---

## Crystal-upgrade `+10` bonus levels are free

**Where**: [packages/logic/src/mechanics/crystalUpgrades.ts](packages/logic/src/mechanics/crystalUpgrades.ts) — `buyCrystalUpgrades`, the `c = 10` branch.

**What**: When `player.upgrades[73]` is owned **and** the player is currently inside any reincarnation challenge (`player.currentChallenge.reincarnation !== 0`), the level target jumps by `+10`. The cost formula, however, only accounts for `toBuy` — the `+10` bonus levels do not contribute to the deduction. So a player satisfying both conditions can land at level `toBuy + 10` for the price of `toBuy`.

**Why**: Intentional reward for the upgrade-73 + in-challenge combination. Behaves like a crystal-upgrade "yield bonus" rather than a price cut.

---

## How to add an entry

Pick a short noun-phrase header, name the file/function, describe what the code does in one paragraph, then add a "Why" paragraph explaining the original motivation (cite the source comment when one exists). Keep entries short — the goal is "would I be surprised reading this code cold?" not exhaustive history.
