# Scoring

Scoring happens once per round, after every card has been played.

## The formula

Let `p` = a player's prediction for the round and `t` = the number of tricks they actually took.

```
if p == t:    score_delta = +20 + 10 * t
else:         score_delta = -10 * |t - p|
```

The result (positive or negative) is added to that player's running total. **Negative totals are allowed.**

### Examples (single round)

| Prediction (p) | Tricks taken (t) | Delta              |
|----------------|------------------|--------------------|
| 0              | 0                | +20                |
| 2              | 2                | +40 (20 + 2×10)    |
| 5              | 5                | +70 (20 + 5×10)    |
| 2              | 1                | −10                |
| 2              | 3                | −10                |
| 0              | 4                | −40                |
| 5              | 0                | −50                |

## Example from the official rulebook

Round 3, three players (Henry, Anya, Freya). Three tricks are available. Predictions: Henry 2, Anya 2, Freya 0.

Actual results: Henry took 2, Anya took 1, Freya took 0.

| Player | Prev. total | Prediction | Taken | Round score                | New total |
|--------|-------------|------------|-------|----------------------------|-----------|
| Henry  | 10          | 2          | 2     | +40 (20 bonus + 2×10)      | 50        |
| Anya   | 10          | 2          | 1     | −10 (one off)              | 0         |
| Freya  | 20          | 0          | 0     | +20 (bonus, no per-trick)  | 40        |

(The cumulative previous totals reflect rounds 1 and 2; Henry had 10, Anya had 10, Freya had 20 going into round 3 in this example.)

## Why predicting 0 still scores

The bonus is for being **right**, not for taking tricks. A correct prediction of 0 earns the full 20-point bonus with no per-trick component. This makes "predict zero and play defensively" a viable strategy when your hand is weak.

## Why correct predictions scale with hand size

Because the prediction bonus is `20 + 10t`, the maximum positive score per round grows linearly with the number of tricks. In late rounds (with many tricks), a single correct prediction can swing the game by 100+ points. This is why the back half of the game tends to dominate the final standings.

## End-of-game

After the final round is scored, the player with the **highest total wins**. Ties produce multiple winners; the rulebook offers no tiebreaker.

See [round-structure.md](round-structure.md) for where scoring fits into the round.