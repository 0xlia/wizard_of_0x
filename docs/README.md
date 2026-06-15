# Wizard — Game Documentation

Reference docs for the card game **Wizard** by Ken Fisher (Amigo Spiel + Freizeit, art by Franz Vohwinkel). These are the official rules from the Amigo English rulebook (Version 3.0, 1996–2022), reorganized for use as an implementation reference.

## Contents

- [overview.md](overview.md) — premise, player count, deck composition, scoring summary
- [round-structure.md](round-structure.md) — the four phases of every trick round: deal, predict, play, score
- [tricks.md](tricks.md) — trick-taking rules: following suit, who wins, all the edge cases
- [special-cards.md](special-cards.md) — Wizards and Jesters in detail
- [scoring.md](scoring.md) — exact scoring formula and worked examples
- [glossary.md](glossary.md) — terms used across the rules

## Quick reference

- **Players:** 3–6
- **Cards:** 60 total — four suits (red, yellow, green, blue) numbered 1–13, plus 4 Wizards and 4 Jesters
- **Rounds:** round *n* deals *n* cards to each player. Total rounds = floor(60 / players) — 20 for 3p, 15 for 4p, 12 for 5p, 10 for 6p
- **Goal:** predict exactly how many tricks you'll take each round. Correct prediction = 20 bonus + 10 per trick taken. Wrong prediction = −10 per trick off.

## Relationship to this implementation

The Rust enum [`GamePhase`](../backend/packages/gamelogic/src/gamelogic.rs) corresponds 1:1 to the round phases described in [round-structure.md](round-structure.md):

| Code phase      | Rule phase                          |
|-----------------|-------------------------------------|
| `NotStarted`    | before round 1                      |
| `ChooseTrumpf`  | trump = Wizard, dealer picks a suit |
| `Prediction`    | phase 2 — predicting tricks         |
| `Playing`       | phase 3 — playing tricks            |
| `RoundEnd`      | phase 4 — scoring                   |
| `GameOver`      | after the final round               |

Card encoding in the code: `Card { suit, value }` where `value` is 0 for Jester, 1–13 for numbered cards, and 14 for Wizard.
