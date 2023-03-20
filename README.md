# durak

[Durak](https://en.wikipedia.org/wiki/Durak) is a card game for 2-6 players where the objective is to not be the last one holding cards.
The loser is called "durak" which is the Russian word for fool.
The game is played with a subset of a standard 52 card deck, using only the cards rank 6 though Ace.

This repo includes the following Rust crates:
- `durak-core`: The core game engine and the `DurakPlayer` trait which defines how the players interact with the game engine.
- `durak-players`: Some implementations of `DurakPlayer` including both a CLI and TUI interface and a basic network middleware.
- `durak`: Server/client binaries for playing a game of durak.
- `durak-ml`: (Coming soon) A ML based computer opponent implementation of `DurakPlayer`.

## Rules:

### Setup:

The deck is shuffled and each player is dealt six cards.
The bottom card of the deck is displayed and its suit determines the trump suit.
This card stays at the bottom of the deck and is the last card dealt out as the game progresses.

### Rounds:

The game is played as a series of rounds where one player defends and the other players take turns attacking.
Each round consists of at maximum 6 attacks and 6 defenses.
The player to the right of the defender attacks first, then the other players clock-wise from the defender's left.
When attacking each player can pass or attack as many times as they wish (without going over the limit of 6 total attacks per round).
The first attacker has precedence when attacking and can preempt any of the other attackers.(\*)
The order of precedence continues to flow clock-wise from the defender's left.

The first attack card can be of any suit or rank, but all other attacks are limited to the ranks already played (either attacks or defenses) so far that round.
To successfully defend an attack, the defender must play a card of a higher rank with the same suit or any card of the trump suit if the attack card was not of the trump suit.

### Ending the Round:

The maximum number of attacker the defender has to defend is the minimum of 6 and number of cards they hold in their hand.
The round can also end early if none of the attackers can or want to attack.

If the defender succeeds in defending all attacks, then the played cards are discarded.
Then the player to the left of the defender is the new defender for the next round and the old defender is the first attacker.

If the defender fails to defend then the defender must pickup and add all the cards played that round to their hand.
Additionally the attackers may add any other cards they wish so long as the rank of the card matches a rank already played.
Then the player two spaces to the left of the defender is the new defender for the next round and the old defender is the last attacker.
The new first attacker is player to the left of the old defender, effectively "skipping" the old defender's turn as the first attacker.

Regardless of a successful defense or not, at the end of the round cards are dealt to all players with less than 6 cards in their hand.
The order starts with first attacker and proceeds clock-wise, but the defender is skipped is dealt to last.

(\*) Only at the very beginning of their turn though. Need to change code to reflect this. Oops.

### Ending the Game:

Once a player has run out of cards and there are no more cards in the deck, they are safely out of the game.
When the second to last player gets rid of all their cards, the last player holding cards is the "durak".

## So why make this?

I was watching Season 4 of Stranger Things and very briefly in one scene some Russian guards are seen playing a card game and one of them says "durak".
I don't speak Russian, but I was curious what the word meant.
Looking it up, I discovered the game durak.
I thought it was interesting and since none of the people I asked wanted to try playing it, I made this.
My goal is to soon develop some kind of AI player for durak, but I needed a game engine first.

