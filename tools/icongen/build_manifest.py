#!/usr/bin/env python3
"""Regenerate manifest.json: currencies + all coin/diamond/automation/generator
upgrade icons. Prompts derive from each upgrade's in-game effect. Run:
    python tools/icongen/build_manifest.py
"""
import json, pathlib

CURRENCY = [
    ("coins", "a single gleaming gold coin seen face-on, warm gold (#ecc95c), a soft mythic emblem embossed on its face."),
    ("quarks", "a glowing teal quark mote (#79dede), a small luminous elementary particle with a faint orbiting wisp, ethereal."),
    ("goldenquarks", "a radiant five-point golden star-mote (#f5d674), a premium golden quark, brighter and more ornate than an ordinary quark."),
    ("ambrosia", "a cluster of two dewy sky-blue ambrosia berries (#9ab6f5) with a small leaf, nectar-like sheen."),
    ("diamonds", "a cut cyan diamond gemstone (#84dbe8), faceted, catching the rim light."),
    ("crystals", "a small cluster of faceted periwinkle crystal shards (#b7c8f7), sharp angular cut-stone fragments grouped together, soft inner glow."),
    ("mythos", "a glowing lavender four-point mythos star (#c39bf0), arcane and radiant."),
    ("mythosshards", "a pair of glowing lavender mythos shards (#c39bf0), twin arcane crystal splinters."),
    ("particles", "a warm-orange particle mote (#f2a877), a bright energetic spark with a faint motion trail."),
    ("offerings", "a dusty-rose votive offering flame (#ee8f86), a teardrop of soft sacrificial light."),
    ("obtainium", "a pink hexagonal essence crystal (#ee8cc6), a glowing six-sided gem of knowledge."),
]

# Per-shop material/palette cue, appended to every subject in that shop.
COIN = " Rendered in warm gold and plum tones."
DIA  = " Rendered in cyan-diamond and periwinkle tones."
AUTO = " Rendered as brass-and-violet clockwork in a pastel-goth palette."
GEN  = " Rendered in teal-and-periwinkle machinery tones."
MYTH = " Rendered in arcane lavender mythos tones."

# idx -> subject (the icon's focal idea, drawn from the effect text)
U = {
 # --- Coin shop 6..20 ---
 6:  "an upward-pointing boost arrow over a cluster of small gold coins and tiny factory shapes — all production rising." + COIN,
 7:  "a glowing multiplication 'x' symbol fed by an alchemist's flask — free multipliers from alchemy." + COIN,
 8:  "speed chevrons (an accelerator) beside a multiplication 'x' — free accelerators from multipliers." + COIN,
 9:  "a multiplication 'x' symbol beside three forward speed chevrons — free multipliers from accelerators." + COIN,
 10: "a worker's hard hat resting on a rising stack of coins — workers improved by investments." + COIN,
 11: "three forward speed chevrons energizing a small gear — accelerators boosting generation." + COIN,
 12: "a small cyan diamond with a rising x1.01 growth curve — prestige compounding production." + COIN,
 13: "an ornate rune-augment sigil over a coin stack — augments buffing investments." + COIN,
 14: "speed chevrons feeding a small printing press — accelerators buffing printers." + COIN,
 15: "speed chevrons feeding a coin-mint stamp — accelerators buffing mints." + COIN,
 16: "speed chevrons beside a cyan diamond — acceleration buffing diamond gain." + COIN,
 17: "a coin-mint stamp blazing with light and a bold x10^100 glow — massive mint multiplier." + COIN,
 18: "a printing press beside a glowing lavender mythos shard — printers powered by mythos shards." + COIN,
 19: "a coin stack beside a lavender four-point mythos star — investments powered by mythos." + COIN,
 20: "a worker's hard hat haloed by a bold exponent '11' power-glow — coin upgrade raised to the 11th power." + COIN,
 # --- Coin shop extended 121..125 ---
 121: "a percent sign cut in half over coins, a sly 'taxes halved' motif." + COIN,
 122: "a periwinkle crystal with an upward cap bracket expanding — crystal cap raised." + COIN,
 123: "a coin stack with a bold rising exponent arrow — coin production raised to a power, more exponents." + COIN,
 124: "a champion's ELO medal wreathed in a rebirth swirl, glowing faster." + COIN,
 125: "a constant-symbol divisor crest marked with challenge tally notches." + COIN,
 # --- Diamond shop 21..40 ---
 21: "a radiant cyan diamond emitting a multiplication 'x' and five speed chevrons — a generous burst of free multipliers and accelerators." + DIA,
 22: "a cyan diamond emitting a multiplication 'x' and four speed chevrons." + DIA,
 23: "a cyan diamond emitting a multiplication 'x' and three speed chevrons." + DIA,
 24: "a cyan diamond emitting a multiplication 'x' and two speed chevrons." + DIA,
 25: "a cyan diamond emitting a multiplication 'x' and one speed chevron." + DIA,
 26: "a single speed chevron bursting with extra energy — a free accelerator boost." + DIA,
 27: "a glowing blue speed rune glyph drawing power from scattered coins." + DIA,
 28: "a glowing duplication rune glyph above small factory buildings." + DIA,
 29: "a glowing speed rune glyph above small factory buildings." + DIA,
 30: "a glowing duplication rune glyph drawing power from scattered coins." + DIA,
 31: "a bursting accelerator-boost chevron above small factory buildings." + DIA,
 32: "three forward speed chevrons drawing from a cyan diamond — free accelerators from diamonds." + DIA,
 33: "a multiplication 'x' beside a bursting accelerator-boost chevron." + DIA,
 34: "a multiplication 'x' with a bold +3% tag — more free multipliers." + DIA,
 35: "a multiplication 'x' with a bold +2% tag — more free multipliers." + DIA,
 36: "a periwinkle crystal multiplied by a cyan diamond, radiant." + DIA,
 37: "a lavender mythos shard beside a cyan diamond with a faint logarithm curve." + DIA,
 38: "a dusty-rose offering flame with a small boost-rocket and a +20% tag." + DIA,
 39: "a stylized ant sprinting with speed lines and a +60% tag." + DIA,
 40: "a stylized ant on a small altar with a reward sparkle and a +25% tag." + DIA,
 # --- Automation shop 81..100 ---
 81: "a robotic clockwork arm picking up a worker's hard hat — auto-buy workers." + AUTO,
 82: "a robotic clockwork arm over a rising coin stack — auto-buy investments." + AUTO,
 83: "a robotic clockwork arm over a printing press — auto-buy printers." + AUTO,
 84: "a robotic clockwork arm over a coin-mint stamp — auto-buy mints." + AUTO,
 85: "a robotic clockwork arm over an alchemist's flask — auto-buy alchemies." + AUTO,
 86: "a gear meshed with three speed chevrons — auto-buy accelerators." + AUTO,
 87: "a gear meshed with a multiplication 'x' — auto-buy multipliers." + AUTO,
 88: "a gear meshed with a bursting accelerator-boost chevron — auto-buy accelerator boosts." + AUTO,
 89: "a gear wrapped around a lavender transcension star — unlock auto transcensions." + AUTO,
 90: "a gear meshed with interlocked generator cogs — auto-buy the generator shop." + AUTO,
 91: "a gear stamped with a gold coin upgrade tile — auto-buy coin upgrades." + AUTO,
 92: "a gear stamped with a cyan diamond upgrade tile — auto-buy diamond upgrades." + AUTO,
 93: "a cyan diamond dripping steadily beside a small clock — passive diamond income per second." + AUTO,
 94: "a gear meshed with an ornate augment rune — auto-buy augments." + AUTO,
 95: "a gear meshed with a glowing enchantment sigil — auto-buy enchantments." + AUTO,
 96: "a gear beside a wizard's pointed hat — auto-buy wizards." + AUTO,
 97: "a gear beside an oracle's glowing eye — auto-buy oracles." + AUTO,
 98: "a gear beside a grandmaster's crowned staff — auto-buy grandmasters." + AUTO,
 99: "a gear stamped with a lavender mythos upgrade tile — auto-buy mythos upgrades." + AUTO,
 100:"a lavender mythos star dripping steadily beside a small clock — passive mythos income per second." + AUTO,
 # --- Generator shop 101..120 ---
 101:"an alchemist's flask linked by an arrow to a coin-mint stamp — alchemies produce mints." + GEN,
 102:"a coin-mint stamp linked by an arrow to a printing press — mints produce printers." + GEN,
 103:"a printing press linked by an arrow to a rising coin stack — printers produce investments." + GEN,
 104:"a rising coin stack linked by an arrow to a worker's hard hat — investments produce workers." + GEN,
 105:"a worker's hard hat linked by an arrow to an alchemist's flask — workers produce alchemies." + GEN,
 106:"an industrial refinery tower linked to an alchemist's flask with a small exponent 0.10." + GEN,
 107:"an industrial refinery and flask with a rising exponent 0.25." + GEN,
 108:"an industrial refinery and flask with a rising exponent 0.50." + GEN,
 109:"an industrial refinery and flask with a rising exponent 0.75." + GEN,
 110:"an industrial refinery and flask with a maxed exponent 1.0, glowing." + GEN,
 111:"an ornate augment rune linked to a mysterious Pandora box with a small exponent 0.08." + GEN,
 112:"an augment rune and Pandora box with a rising exponent 0.16." + GEN,
 113:"an augment rune and Pandora box with a rising exponent 0.24." + GEN,
 114:"an augment rune and Pandora box with a rising exponent 0.32." + GEN,
 115:"an augment rune and Pandora box with a maxed exponent 0.40, glowing." + GEN,
 116:"a glowing proton particle linked to a grandmaster's crowned staff with a small exponent 0.05." + GEN,
 117:"a proton and grandmaster staff with a rising exponent 0.10." + GEN,
 118:"a proton and grandmaster staff with a rising exponent 0.15." + GEN,
 119:"a proton and grandmaster staff with a rising exponent 0.20." + GEN,
 120:"a proton and grandmaster staff with a maxed exponent 0.25, glowing." + GEN,
 # --- Mythos shop 41..60 ---
 41: "a lavender four-point mythos star radiating an upward production boost." + MYTH,
 42: "a pair of lavender mythos shards beside a cyan diamond — shards powered by diamonds." + MYTH,
 43: "a gold coin haloed by a transcension rebirth swirl — coin production per transcension." + MYTH,
 44: "a lavender mythos star inside a transcension rebirth swirl — more mythos per transcension." + MYTH,
 45: "three forward speed chevrons drawing from a pair of mythos shards." + MYTH,
 46: "a bursting accelerator-boost chevron behind a small protective shield — stronger boosts that don't reset." + MYTH,
 47: "a pair of mythos shards beside a small achievement-point trophy star." + MYTH,
 48: "speed chevrons and a multiplication 'x' radiating shared production power." + MYTH,
 49: "a multiplication 'x' fed by a lavender mythos star — free multipliers from mythos." + MYTH,
 50: "crossed challenge swords with speed chevrons and a multiplication 'x' — a challenge-only boost." + MYTH,
 51: "an arcane mythos building tower lifted by a bursting accelerator boost." + MYTH,
 52: "an arcane mythos building tower haloed by a small rising exponent." + MYTH,
 53: "an ornate augment rune spilling lavender mythos shards." + MYTH,
 54: "a wizard's pointed hat conjuring a glowing enchantment sigil." + MYTH,
 55: "a grandmaster's crowned staff beside a glowing oracle eye." + MYTH,
 56: "a worker's hard hat blazing with a bold x10^5000 exponent glow." + MYTH,
 57: "a rising coin stack blazing with a bold x10^7500 exponent glow." + MYTH,
 58: "a printing press blazing with a bold x10^15000 exponent glow." + MYTH,
 59: "a coin-mint stamp blazing with a bold x10^25000 exponent glow." + MYTH,
 60: "an alchemist's flask blazing with a bold x10^35000 exponent glow." + MYTH,
}

# --- Achievements: medallion/badge style, distinct from the upgrade "object"
# look. Each is the goal's motif embossed on a metal medallion; the metal tier
# escalates within each producer group (bronze -> silver -> gold -> prismatic)
# so harder achievements look rarer. Achievement N -> achievements list[N-1].
TIERS = ["a weathered bronze", "a weathered bronze", "a polished silver",
         "a polished silver", "a gleaming gold", "a gleaming gold",
         "an iridescent prismatic"]
# (first achievement idx, motif) for each 7-step producer/reset group.
ACH_GROUPS = [
    (2,  "a worker's hard hat"),
    (9,  "a rising stack of gold coins with a small 'stonks' up-arrow"),
    (16, "a small printing press"),
    (23, "a coin-mint stamp"),
    (30, "an alchemist's bubbling flask"),
    (37, "a cut cyan diamond"),
    (44, "a lavender four-point mythos star"),
]
ACH = {1: "a gleaming gold commemorative medallion embossed with a magnifying "
          "glass crossing a star — the achievement-hunter badge, slim ribbon, "
          "soft pastel-goth palette."}
for first, motif in ACH_GROUPS:
    for i in range(7):
        m = motif
        a = first + i
        if a in (32, 33, 34):  # the "demonic" alchemy achievements
            m = "an alchemist's flask glowing with an eerie arcane sigil"
        ACH[a] = (f"{TIERS[i]} commemorative achievement medallion embossed with "
                  f"{m}, a slim ribbon at the base, soft pastel-goth palette.")

# Achievements 51-100 — variable-length groups (Particles, the "without X"
# restriction vows, out-of-order generator buys, challenge exits, and the
# No-Multiplier / No-Accelerator / No-Shards / Cost+ challenge-completion
# ladders). (first, count, motif); tier spreads bronze->prismatic over the run.
def spread_tier(i, length):
    return "a gleaming gold" if length <= 1 else TIERS[round(i * 6 / (length - 1))]

ACH2_GROUPS = [
    (51, 7, "a warm-orange glowing particle mote"),
    (58, 3, "a multiplication 'x' symbol crossed out by a prohibition slash"),
    (61, 4, "forward speed chevrons crossed out by a prohibition slash"),
    (65, 7, "a coin-upgrade tile crossed out by a prohibition slash — a miser's vow"),
    (72, 4, "interlocking generator cogs stamped with an out-of-order numeral"),
    (76, 3, "an exit doorway with a scatter of escaping gold coins"),
    (79, 7, "a crossed-swords challenge crest over a struck-through multiplication 'x'"),
    (86, 7, "a crossed-swords challenge crest over struck-through speed chevrons"),
    (93, 7, "a crossed-swords challenge crest over a struck-through lavender mythos shard"),
    (100, 1, "a price tag with a bold upward arrow — rising costs"),
]
# Achievements 101-150 — challenge-completion ladders (Cost+, Reduced Diamonds,
# Tax+, No Accel/Mult, Cost++, No Runes, Sadistic I) plus two accelerator buys.
ACH3_GROUPS = [
    (101, 6, "a crossed-swords challenge crest over a price tag with an upward arrow — rising costs"),
    (107, 7, "a crossed-swords challenge crest over a pickaxe striking a small cyan diamond"),
    (114, 7, "a crossed-swords challenge crest over a tax percent-sign with an upward arrow"),
    (121, 7, "a crossed-swords challenge crest over struck-through speed chevrons and a struck-through multiplication 'x'"),
    (128, 7, "a crossed-swords challenge crest over a price tag with two steep upward arrows"),
    (135, 7, "a crossed-swords challenge crest over a struck-through glowing rune glyph"),
    (142, 7, "a menacing horned skull challenge crest — a sadistic trial"),
    (149, 2, "forward speed chevrons — an accelerator"),
]
for first, count, motif in ACH2_GROUPS + ACH3_GROUPS:
    for i in range(count):
        ACH[first + i] = (f"{spread_tier(i, count)} commemorative achievement medallion "
                          f"embossed with {motif}, a slim ribbon at the base, soft "
                          f"pastel-goth palette.")

icons = [{"id": cid, "category": "currency", "prompt": p} for cid, p in CURRENCY]
for idx in sorted(U):
    icons.append({"id": f"upgrade{idx}", "category": "upgrade", "prompt": U[idx]})
for idx in sorted(ACH):
    icons.append({"id": f"ach{idx}", "category": "achievement", "prompt": ACH[idx]})

out = {"_comment": "Icon specs. id -> assets/pictures/<category>/<id>.png. prompt is appended to style.txt. Upgrade ids match the in-game bitmap index (upgrade<idx>). Regenerate with build_manifest.py.", "icons": icons}
p = pathlib.Path("tools/icongen/manifest.json")
p.write_text(json.dumps(out, indent=2) + "\n")
print(f"wrote {p} with {len(icons)} icons ({sum(1 for i in icons if i['category']=='upgrade')} upgrades)")
