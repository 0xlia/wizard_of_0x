# Round Structure

Every round of Wizard runs through the same four phases in order.

1. [Dealing cards](#1-dealing-cards)
2. [Predicting tricks](#2-predicting-tricks)
3. [Playing tricks](#3-playing-tricks)
4. [Earning points](#4-earning-points)

## 1. Dealing cards

- In round *n*, each player is dealt *n* cards.
- The **top card of the remaining stack is flipped face-up** to determine the **trumpf color** for the round:
  - **Number card flipped** → that card's color is trump for the round.
  - **Wizard flipped** → the dealer looks at their hand, chooses a trumpf color, and announces it clearly. (This is the `ChooseTrumpf` phase in code.)
  - **Jester flipped** → this round has **no trumpf color**.
- **Final round exception:** all 60 cards are dealt, so there is no card left to flip. The final round has **no trumpf color**.

See [special-cards.md](special-cards.md) for how Wizards/Jesters in the trumpf flip work.

## 2. Predicting tricks

- Starting with the dealer's left-hand neighbor and proceeding **clockwise**, each player announces how many tricks they think they will win this round.
- Predictions are integers in the range **`[0, n]`** where *n* is the number of cards in hand.
- Predictions are made **out loud** and **written down** on the scorepad before any trick is played.
- The **sum of predictions needs to differ** from the number of tricks available — players can collectively over- or under-predict.
- Once locked in, a prediction cannot be changed.

## 3. Playing tricks

- The dealer's left-hand neighbor leads the **first trick** by playing any card from their hand.
- Play proceeds **clockwise**; each remaining player contributes one card to the trick.
- **Follow suit:** if the lead card is a numbered card (red/yellow/green/blue), every other player **must play a card of that color if they have one**. If they have no card of the led suit, they may play any other card — including a trumpf or a Wizard.
- **Wizards and Jesters override the follow-suit rule:** they may be played at any time, even if the player could follow suit. They have no color.
- The color you must follow does **not change** mid-trick (a Wizard or Jester played later in the trick does not redefine the led suit).
- The winner of the trick collects it in a **face-down pile** in front of them, kept visible enough to see how many tricks they have won so far.
- The winner of the trick **leads the next trick**.

Full trick-resolution rules and edge cases are in [tricks.md](tricks.md).

## 4. Earning points

After every card has been played:

- **Correct prediction:** `20 + 10 × tricks_taken` points.
- **Wrong prediction:** `−10 × |tricks_taken − prediction|` points.

Points are added to (or subtracted from) the running total on the scorepad. Negative totals are possible.

See [scoring.md](scoring.md) for examples.

After scoring, the dealer role passes one seat clockwise and the next round begins with one more card per player.
