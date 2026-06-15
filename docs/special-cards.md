# Special Cards: Wizards and Jesters

The 60-card deck contains 8 special cards: **4 Wizards** and **4 Jesters**. They have no suit/color. Understanding them is most of what makes Wizard distinct from a generic trick-taker.

## Wizards

Wizards are the **strongest** cards in the game. A Wizard beats every numbered card of every suit, including trumpf.

### Wizards in play

- **First Wizard played wins the trick.** All four Wizards are equal in strength — order of play decides.
- A Wizard **does not have a suit**, so playing one **does not require** following suit. You may play a Wizard at any time, even if you could follow suit with a numbered card.
- A Wizard played later in a trick **does not change which suit other players must follow** — the led color is fixed by the lead card.

### Leading a trick with a Wizard

- The Wizard wins the trick immediately (no later card can beat it).
- **All other players may play any card of their choice**, including other Wizards or Jesters. There is no suit to follow.
- A common tactical use: when you must win a trick but don't want to commit a strong numbered card, lead with a Wizard — opponents will dump their worst cards.

### Wizard as the trumpf-determining flip

- When the dealer flips a Wizard to determine trumpf (see [round-structure.md](round-structure.md)), the round enters the `ChooseTrumpf` phase: the **dealer** looks at their hand and **chooses any of the four colors as trumpf**. They announce the choice clearly before predictions begin.
- Note: this is the **dealer**, not the leader. The dealer always picks trumpf in this case.

## Jesters

Jesters are the **weakest** cards in the game. A Jester loses to every numbered card, every trumpf, and every Wizard.

### Jesters in play

- Like Wizards, Jesters **have no suit** and may be played at any time, even when you could follow suit.
- A Jester **almost always loses** the trick. The only exception is the all-Jesters trick.
- Typical use: dump a Jester when you cannot follow suit and do not want to waste a useful card, or when you are trying to *avoid* winning the trick.

### Leading a trick with a Jester

A lead Jester is a special case because it doesn't establish a suit on its own.

- Following players may play **any card** at first.
- The **first non-Jester** card played determines what happens next:
  - **Numbered card** → its color becomes the trick's suit. All *remaining* players must follow that suit if they can (Wizards/Jesters remain always-legal).
  - **Wizard** → the Wizard wins; remaining players may play any card.
- If every card played in the trick is a Jester, the **first Jester played** wins (this is the one and only way a Jester wins a trick).

### Jester as the trumpf-determining flip

- When the dealer flips a Jester to determine trump, the round has **no trumpf color**. Only the led suit matters for winning tricks that round.

## Quick reference: card legality

A card you play is legal if and only if at least one of these is true:

- It is a **Wizard** or **Jester** (always legal).
- It is the **same color** as the lead card.
- The lead card was a **Wizard** (no suit established).
- The lead card was a **Jester** and no non-Jester has been played yet in this trick (no suit established yet).
- You **hold no cards** of the led color.

## Quick reference: card strength

For a single trick, ranking from strongest to weakest:

1. The **first Wizard** played in the trick (other Wizards tie below it but only the first matters for winning).
2. Subsequent Wizards (cannot win, but block nothing — they just lose to the earlier Wizard).
3. Highest trumpf.
4. Lower trumps in descending order.
5. Highest card of the led color.
6. Lower cards of the led color in descending order.
7. Cards of any other (non-trumpf, non-led) color — cannot win.
8. Jesters — cannot win unless the entire trick is Jesters.
