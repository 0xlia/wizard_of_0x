# Tricks: Playing and Winning

## What is a trick?

A **trick** is one card played by each player in turn. In a round where each player holds *n* cards, exactly *n* tricks will be played.

## Who plays first

- The **first trick** of a round is led by the **dealer's left-hand neighbor**.
- Every subsequent trick is led by the **winner of the previous trick**.

## Order of play within a trick

Play proceeds **clockwise** from the leader. Each player plays exactly one card, then the trick is resolved.

## Follow-suit rule

When a trick is led with a **numbered card**, every later player must:

1. Play a card of the **same color** as the lead card, if they have one.
2. If they have **no** card of that color, they may play any card — including a trumpf card or a Wizard or a Jester.

A player may **always** play a Wizard or Jester instead of following suit, even when they hold a card of the led color. Wizards and Jesters have no color.

The "led color" of the trick is fixed at the moment the lead card determines it (see "Leading with a Wizard or Jester" below for the edge cases) — playing a Wizard or Jester partway through the trick does **not** change which color other players must follow.

## Winning a trick

Apply these rules in order. The first that matches selects the winner:

1. **First Wizard wins.** If any Wizard was played, the trick is won by whoever played the **first** Wizard.
2. **Otherwise, highest trumpf wins.** If there is a trump color this round and at least one trump was played, the **highest-value trump** (1–13) wins.
3. **Otherwise, highest card of the led color wins.** Cards of any non-trump, non-led color cannot win — they are effectively discarded into the trick.
4. **All-Jesters edge case:** if the trick contains *only* Jesters (no Wizard, no numbered cards), the **first Jester played** wins.

Jesters otherwise always lose. A Jester can never beat a numbered card or a Wizard.

## Leading with a Wizard

If the leader plays a **Wizard** as the lead card:

- The Wizard immediately claims the trick (no later card can beat it — rule 1).
- **Every other player may play any card** of their choice, including Wizards and Jesters. There is no suit to follow.

## Leading with a Jester

If the leader plays a **Jester** as the lead card:

- Each subsequent player may **freely choose** their card, **until** someone plays a Wizard or a numbered card.
- The **first non-Jester card played** sets the trick's terms:
  - If it is a **numbered card**, its color becomes the suit that all *remaining* players must follow (using the normal follow-suit rule — they may play off-color only if they hold none of that suit, and Wizards/Jesters are still always legal).
  - If it is a **Wizard**, all remaining players may play any card; the Wizard wins the trick.
- If every card in the trick is a Jester, see rule 4 above (the first Jester played wins).

## Collecting tricks

Won tricks are stacked **face-down** in front of the winning player so the count of tricks-won-this-round is visible to everyone. Cards from earlier tricks are not consulted again.

See [special-cards.md](special-cards.md) for more detail on Wizards/Jesters, and [round-structure.md](round-structure.md) for how tricks fit into the round.
