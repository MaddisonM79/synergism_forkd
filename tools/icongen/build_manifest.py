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
PART = " Rendered in warm-orange particle tones with plum shadows."

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
 # --- Particle shop 61..80 (reincarnation / obtainium / ant themed) ---
 61: "a glowing orange particle with a small +7 salvage spark — welcome to reincarnation." + PART,
 62: "a dusty-rose offering flame beside a challenge-completion tally." + PART,
 63: "a periwinkle crystal supercharged by a swirl of orange particles." + PART,
 64: "a lavender mythos shard supercharged by a swirl of orange particles." + PART,
 65: "a bright burst of orange particles with a bold x5 glow." + PART,
 66: "five small rune glyphs lifted by a + coefficient boost." + PART,
 67: "a glowing atom orbited by tiny orange particle motes." + PART,
 68: "a multiplication 'x' fed by a rising tax percent-sign." + PART,
 69: "a pink hexagonal obtainium gem fed by orange particles." + PART,
 70: "a clock face speeding up, ringed with orange particles." + PART,
 71: "a rune glyph soaking up EXP sparks from an offering flame." + PART,
 72: "a pink obtainium hexagon multiplied by challenge tally marks." + PART,
 73: "a bursting accelerator-boost chevron inside a challenge crest." + PART,
 74: "a pink obtainium hexagon fed by a small hoard of offering flames." + PART,
 75: "a dusty-rose offering flame fed by a small hoard of obtainium gems." + PART,
 76: "a stylized ant sprinting with a bold x5 speed burst." + PART,
 77: "a worker ant carrying a small speed-multiplier symbol." + PART,
 78: "a stylized ant accelerated by a rising offerings curve." + PART,
 79: "a stylized ant supercharged by a global-speed vortex." + PART,
 80: "a stylized ant earning an ELO medal beside a sacrificial altar." + PART,
}

# Building / producer cards — painted object/character icons (not medallions),
# tinted per economy layer. ids: coin1-5 / diamond1-5 / mythos1-5 + the three
# special buildings. Category "building".
BUILDINGS = [
    ("coin1", "a hard-hatted worker laborer with a pickaxe, warm gold tones."),
    ("coin2", "a briefcase overflowing with gold coins — an investment, warm gold tones."),
    ("coin3", "a coin printing press machine stamping gold coins, warm gold tones."),
    ("coin4", "a coin-minting stamp press striking gold coins, warm gold tones."),
    ("coin5", "an alchemist's flask transmuting liquid into gold, warm gold tones."),
    ("diamond1", "an industrial refinery tower with pipes, cyan-diamond tones."),
    ("diamond2", "a coal power plant with smokestacks, cyan-diamond tones."),
    ("diamond3", "a coal mining drilling rig, cyan-diamond tones."),
    ("diamond4", "a pickaxe striking a large cyan diamond, cyan tones."),
    ("diamond5", "an ornate mysterious Pandora's box glowing cyan."),
    ("mythos1", "an ornate arcane augment rune sigil, lavender mythos tones."),
    ("mythos2", "a glowing enchantment scroll with a sigil, lavender mythos tones."),
    ("mythos3", "a wizard's pointed star-speckled hat, lavender mythos tones."),
    ("mythos4", "a glowing oracle's all-seeing eye orb, lavender mythos tones."),
    ("mythos5", "a grandmaster's crowned staff, lavender mythos tones."),
    ("accelerator", "a glowing speedometer with forward speed chevrons, gold-and-cyan tones."),
    ("multiplier", "a bold ornate multiplication 'x' emblem, gold tones."),
    ("acceleratorboost", "a bursting rocket-boost chevron with energy sparks, cyan tones."),
]

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
# Achievements 151-200 — purchase ladders (Accelerators/Multipliers/Boosts),
# Galactic Crumbs, Immortal-ELO + Ant tiers, Ascensions, math constants, and
# the Reduced-Ants challenge.
ACH4_GROUPS = [
    (151, 5, "forward speed chevrons — an accelerator"),
    (156, 7, "a bold multiplication 'x' symbol"),
    (163, 7, "a bursting accelerator-boost chevron"),
    (170, 7, "a glowing golden galactic breadcrumb morsel — a cosmic crumb"),
    (177, 7, "a stylized ant beside a champion's ELO medal"),
    (184, 7, "an upward ascension burst of radiant rising rays"),
    (191, 7, "a glowing mathematical constant symbol pi"),
    (198, 3, "a crossed-swords challenge crest over a struck-through ant"),
]
# Achievements 201-250 — more challenge ladders (Reduced Ants, No Reincarnation,
# Tax+++, No Research), Ascension-score tiers, and Rune Blessing/Spirit levels.
ACH5_GROUPS = [
    (201, 4, "a crossed-swords challenge crest over a struck-through ant"),
    (205, 7, "a crossed-swords challenge crest over a struck-through reincarnation particle swirl"),
    (212, 7, "a crossed-swords challenge crest over a tax percent-sign with three steep upward arrows"),
    (219, 7, "a crossed-swords challenge crest over a struck-through research light-bulb"),
    (226, 7, "an upward ascension burst with a glowing score star"),
    (233, 3, "a radiant blessing halo over a blue speed rune"),
    (236, 3, "an ethereal spirit wisp swirling around a blue speed rune"),
]
# 239-253 are deliberately cryptic "secret/hint" achievements — give them a
# uniform sealed-mystery medallion rather than spoiling the goal.
for _a in list(range(239, 251)) + [251, 252, 253]:
    ACH[_a] = ("an enigmatic obsidian commemorative medallion sealed with a glowing "
               "question mark — a mysterious secret achievement, a slim ribbon at the "
               "base, soft pastel-goth palette.")

# Achievements 254-300 — Ascension-score & total-ascension tiers, math constants,
# the Singularity reset ladder (new black-hole motif), and big producer/prestige
# milestones reusing the established producer motifs.
ACH6_GROUPS = [
    (254, 7, "an upward ascension burst with a glowing score star"),
    (261, 7, "an upward ascension burst of radiant rising rays"),
    (268, 7, "a glowing mathematical constant symbol pi"),
    (275, 7, "a swirling singularity black-hole vortex ringed with golden motes"),
    (282, 3, "a worker's hard hat"),
    (285, 3, "a rising stack of gold coins"),
    (288, 3, "a small printing press"),
    (291, 3, "a coin-mint stamp"),
    (294, 3, "an alchemist's bubbling flask"),
    (297, 3, "a cut cyan diamond"),
    (300, 1, "a lavender four-point mythos star"),
]

# Achievements 301-350 — deep prestige/transcend/reincarnate goals and high-count
# repeats of every challenge ladder + producer/crumb/ELO milestones. All reuse
# the established motifs.
ACH7_GROUPS = [
    (301, 2, "a lavender four-point mythos star"),
    (303, 3, "a warm-orange glowing particle mote"),
    (306, 3, "a crossed-swords challenge crest over a struck-through multiplication 'x'"),
    (309, 3, "a crossed-swords challenge crest over struck-through speed chevrons"),
    (312, 3, "a crossed-swords challenge crest over a struck-through lavender mythos shard"),
    (315, 3, "a crossed-swords challenge crest over a price tag with an upward arrow — rising costs"),
    (318, 3, "a crossed-swords challenge crest over a pickaxe striking a small cyan diamond"),
    (321, 3, "a crossed-swords challenge crest over a tax percent-sign with an upward arrow"),
    (324, 3, "a crossed-swords challenge crest over struck-through speed chevrons and a struck-through multiplication 'x'"),
    (327, 3, "a crossed-swords challenge crest over a price tag with two steep upward arrows"),
    (330, 3, "a crossed-swords challenge crest over a struck-through glowing rune glyph"),
    (333, 3, "a menacing horned skull challenge crest — a sadistic trial"),
    (336, 3, "forward speed chevrons — an accelerator"),
    (339, 3, "a bold multiplication 'x' symbol"),
    (342, 3, "a bursting accelerator-boost chevron"),
    (345, 3, "a glowing golden galactic breadcrumb morsel — a cosmic crumb"),
    (348, 3, "a stylized ant beside a champion's ELO medal"),
]

# Achievements 351-400 — extreme Ascension/constant tiers, high-count challenge
# ladders, and Speed Blessing/Spirit/Rune level milestones.
ACH8_GROUPS = [
    (351, 6, "an upward ascension burst of radiant rising rays"),
    (357, 6, "a glowing mathematical constant symbol pi"),
    (363, 5, "a crossed-swords challenge crest over a struck-through ant"),
    (368, 5, "a crossed-swords challenge crest over a struck-through reincarnation particle swirl"),
    (373, 5, "a crossed-swords challenge crest over a tax percent-sign with three steep upward arrows"),
    (378, 5, "a crossed-swords challenge crest over a struck-through research light-bulb"),
    (383, 7, "a radiant blessing halo over a blue speed rune"),
    (390, 7, "an ethereal spirit wisp swirling around a blue speed rune"),
    (397, 4, "a glowing blue speed rune glyph"),
]

# Achievements 401-450 — Speed Rune level & free-level milestones, Campaign
# Tokens (new token motif), and the Prestige-count ladder.
ACH9_GROUPS = [
    (401, 11, "a glowing blue speed rune glyph"),
    (412, 15, "a blue speed rune glyph haloed with bonus free-level sparkles"),
    (427, 10, "a glowing engraved campaign token coin"),
    (437, 14, "a cut cyan diamond"),
]

# Achievements 451-500 — Prestige/Transcend/Reincarnate count ladders, Anthill
# Sacrifice, and the "Code: add" promo achievements (new ticket motif).
ACH10_GROUPS = [
    (451, 1, "a cut cyan diamond"),
    (452, 15, "a lavender four-point mythos star"),
    (467, 15, "a warm-orange glowing particle mote"),
    (482, 13, "a stylized ant on a sacrificial altar with a reward sparkle"),
    (495, 6, "a glowing promo-code ticket stamped with a plus sign"),
]

# Achievements 501-509 — the final "Code: add" promo block (same ticket motif).
ACH11_GROUPS = [
    (501, 9, "a glowing promo-code ticket stamped with a plus sign"),
]

for first, count, motif in (ACH2_GROUPS + ACH3_GROUPS + ACH4_GROUPS + ACH5_GROUPS
                            + ACH6_GROUPS + ACH7_GROUPS + ACH8_GROUPS + ACH9_GROUPS
                            + ACH10_GROUPS + ACH11_GROUPS):
    for i in range(count):
        ACH[first + i] = (f"{spread_tier(i, count)} commemorative achievement medallion "
                          f"embossed with {motif}, a slim ribbon at the base, soft "
                          f"pastel-goth palette.")

icons = [{"id": cid, "category": "currency", "prompt": p} for cid, p in CURRENCY]
for idx in sorted(U):
    icons.append({"id": f"upgrade{idx}", "category": "upgrade", "prompt": U[idx]})
for idx in sorted(ACH):
    icons.append({"id": f"ach{idx}", "category": "achievement", "prompt": ACH[idx]})
for bid, bp in BUILDINGS:
    icons.append({"id": bid, "category": "building", "prompt": bp})

out = {"_comment": "Icon specs. id -> assets/pictures/<category>/<id>.png. prompt is appended to style.txt. Upgrade ids match the in-game bitmap index (upgrade<idx>). Regenerate with build_manifest.py.", "icons": icons}
p = pathlib.Path("tools/icongen/manifest.json")
p.write_text(json.dumps(out, indent=2) + "\n")
print(f"wrote {p} with {len(icons)} icons ({sum(1 for i in icons if i['category']=='upgrade')} upgrades)")
