#![allow(unused)]
use rand::{Rng, seq::SliceRandom};
use rand_distr::Alphanumeric;
use std::fmt::Display;
use thiserror::Error;

const WIZARD_VALUE: usize = 14;
const JESTER_VALUE: usize = 0;
const DECK_SIZE: usize = 60;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum GameLogicError {
    #[error(transparent)]
    AddPlayer(#[from] AddPlayerError),
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AddPlayerError {
    #[error("The lobby was already at max capacity {max_players}/{max_players}.")]
    LobbyFull { max_players: usize },

    #[error("{name} already taken.")]
    NameTaken { name: String },
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PredictionError {
    #[error("not in prediction phase")]
    WrongPhase,
    #[error("not your turn")]
    NotYourTurn,
    #[error("prediction must be between 0 and {max}")]
    OutOfRange { max: usize },
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PlayCardError {
    #[error("not in playing phase")]
    WrongPhase,
    #[error("not your turn")]
    NotYourTurn,
    #[error("card index {index} out of range (hand has {hand_size} cards)")]
    BadIndex { index: usize, hand_size: usize },
    #[error("must follow lead suit")]
    MustFollowSuit,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameEvent {
    RoundStarted {
        round: usize,
        dealer: usize,
        flipped_card: Option<Card>,
    },
    TrumpfChosen {
        chooser: usize,
        suit: Suit,
    },
    Predicted {
        player: usize,
        prediction: usize,
    },
    CardPlayed {
        player: usize,
        card: Card,
    },
    TrickWon {
        winner: usize,
    },
    RoundScored {
        round: usize,
        deltas: Vec<i32>,
    },
    GameOver,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum GamePhase {
    NotStarted,
    ChooseTrumpf,
    Prediction,
    Playing,
    RoundEnd,
    GameOver,
}

#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub enum Suit {
    Red,
    Yellow,
    Green,
    Blue,
}

#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord)]
pub struct Card {
    pub suit: Suit,
    pub value: usize,
}

impl Card {
    pub fn new(suit: Suit, value: usize) -> Self {
        Card { suit, value }
    }

    pub fn is_wizard(&self) -> bool {
        self.value == WIZARD_VALUE
    }

    pub fn is_jester(&self) -> bool {
        self.value == JESTER_VALUE
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Deck {
    deck: [Card; DECK_SIZE],
}

impl Deck {
    pub fn new() -> Self {
        let deck: [Card; DECK_SIZE] = std::array::from_fn(|i| {
            Card::new(
                match i % 4 {
                    0 => Suit::Blue,
                    1 => Suit::Red,
                    2 => Suit::Green,
                    3 => Suit::Yellow,
                    _ => unreachable!(),
                },
                i / 4,
            )
        });

        Self { deck }
    }

    fn shuffle(&mut self) {
        self.deck.shuffle(&mut rand::rng());
    }

    fn deal_cards(&self, players: &mut [Player], round_number: usize) -> Option<Card> {
        for (player_index, player) in players.iter_mut().enumerate() {
            for i in 0..round_number {
                player
                    .hand
                    .push(self.deck[player_index * round_number + i].clone());
            }
        }
        if players.len() * round_number < self.deck.len() {
            Some(self.deck[players.len() * round_number].clone())
        } else {
            None
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Player {
    name: String,
    hand: Vec<Card>,
    prediction: Option<usize>,
    tricks_won: usize,
    points: i32,
    is_bot: bool,
}

impl Player {
    pub fn new(name: impl Into<String>) -> Self {
        Player {
            name: name.into(),
            hand: Vec::new(),
            prediction: None,
            tricks_won: 0,
            points: 0,
            is_bot: false,
        }
    }

    pub fn new_bot(name: impl Into<String>) -> Self {
        Self {
            is_bot: true,
            ..Self::new(name)
        }
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn is_bot(&self) -> bool {
        self.is_bot
    }

    #[must_use]
    pub fn hand(&self) -> &[Card] {
        &self.hand
    }

    #[must_use]
    pub fn prediction(&self) -> Option<usize> {
        self.prediction
    }

    #[must_use]
    pub fn tricks_won(&self) -> usize {
        self.tricks_won
    }

    #[must_use]
    pub fn points(&self) -> i32 {
        self.points
    }

    pub fn reset(&mut self) {
        self.hand = Vec::new();
        self.prediction = None;
        self.tricks_won = 0;
    }

    pub fn sort_hand(&mut self) {
        self.hand.sort();
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct WizardGame {
    num_players: usize,
    players: Vec<Player>,
    current_player_index: usize,
    current_trick_starter: usize,

    game_phase: GamePhase,
    round_number: usize,

    deck: Deck,
    trumpf: Option<Card>,
    current_trick: Vec<(usize, Card)>,
    event_log: Vec<GameEvent>,
}

impl WizardGame {
    /// Creates new gamestate
    ///
    /// # Examples
    ///
    /// ```
    /// use gamelogic::gamelogic::WizardGame;
    /// let game = WizardGame::new(3, "JARV1S");
    /// ```
    #[must_use]
    pub fn new(num_players: usize, player_name: impl Into<String>) -> Self {
        let new_player = Player::new(player_name);

        Self {
            num_players,
            players: vec![new_player],
            current_player_index: 0,
            current_trick_starter: 0,

            game_phase: GamePhase::NotStarted,
            round_number: 0,

            deck: Deck::new(),
            trumpf: None,
            current_trick: Vec::new(),
            event_log: Vec::new(),
        }
    }

    #[must_use]
    pub fn event_log(&self) -> &[GameEvent] {
        &self.event_log
    }

    #[must_use]
    pub fn num_players(&self) -> usize {
        self.num_players
    }

    #[must_use]
    pub fn players(&self) -> &[Player] {
        &self.players
    }

    #[must_use]
    pub fn is_started(&self) -> bool {
        self.game_phase != GamePhase::NotStarted
    }

    #[must_use]
    pub fn is_full(&self) -> bool {
        self.players.len() >= self.num_players
    }

    #[must_use]
    pub fn round_number(&self) -> usize {
        self.round_number
    }

    #[must_use]
    pub fn max_rounds(&self) -> usize {
        DECK_SIZE / self.num_players
    }

    #[must_use]
    pub fn trumpf(&self) -> Option<&Card> {
        self.trumpf.as_ref()
    }

    #[must_use]
    pub fn current_player_index(&self) -> usize {
        self.current_player_index
    }

    /// Dealer for the current round. Rotates clockwise each round starting from player 0.
    pub fn dealer_index(&self) -> usize {
        (self.round_number - 1) % self.num_players
    }

    #[must_use]
    pub fn current_trick_starter(&self) -> usize {
        self.current_trick_starter
    }

    #[must_use]
    pub fn game_phase(&self) -> &GamePhase {
        &self.game_phase
    }

    #[must_use]
    pub fn current_trick(&self) -> &[(usize, Card)] {
        &self.current_trick
    }

    /// PHASE: `NOT_STARTED`
    /// returns random ID
    pub fn create_id() -> String {
        let password_len = 5;
        let mut rng = rand::rng();
        let id: String = (0..password_len)
            .map(|_| rng.sample(Alphanumeric) as char)
            .collect();
        id
    }

    /// add player to game
    ///
    /// # Examples
    ///
    /// ```
    /// use gamelogic::gamelogic::{Player, WizardGame};
    ///
    /// let mut game = WizardGame::new(3, "Justus");
    /// let new_player = Player::new("TruelyMostWanted");
    /// game.add_player(new_player).unwrap();
    /// ```
    pub fn add_player(&mut self, player: Player) -> Result<(), AddPlayerError> {
        if self.players.len() >= self.num_players {
            return Err(AddPlayerError::LobbyFull {
                max_players: self.num_players,
            });
        }

        if self.players.iter().any(|other| player.name == other.name) {
            return Err(AddPlayerError::NameTaken { name: player.name });
        }

        self.players.push(player);

        Ok(())
    }

    pub fn start_game(&mut self) {
        self.start_new_round();
    }

    // Game Phase NOT STARTED
    fn start_new_round(&mut self) {
        self.round_number += 1;
        let dealer = self.dealer_index();
        self.current_player_index = (dealer + 1) % self.num_players;
        self.current_trick_starter = self.current_player_index;
        self.deck.shuffle();

        // reset players
        for player in &mut self.players {
            player.reset();
        }

        // reset current trick
        self.current_trick = Vec::new();

        // set trumpf and deal cards
        self.trumpf = self.deck.deal_cards(&mut self.players, self.round_number);

        self.event_log.push(GameEvent::RoundStarted {
            round: self.round_number,
            dealer,
            flipped_card: self.trumpf.clone(),
        });

        if let Some(card) = &self.trumpf
            && card.is_wizard()
        {
            self.game_phase = GamePhase::ChooseTrumpf;
        } else {
            self.game_phase = GamePhase::Prediction;
        }
    }

    // GAME PHASE: Choose Trumpf
    pub fn choose_trumpf(&mut self, suit: Suit) {
        let chooser = self.dealer_index();
        self.trumpf = Some(Card::new(suit.clone(), WIZARD_VALUE));
        self.game_phase = GamePhase::Prediction;
        self.event_log
            .push(GameEvent::TrumpfChosen { chooser, suit });
    }

    // GAME PHASE: Prediction
    pub fn place_prediction(
        &mut self,
        player_index: usize,
        prediction: usize,
    ) -> Result<(), PredictionError> {
        if self.game_phase != GamePhase::Prediction {
            return Err(PredictionError::WrongPhase);
        }
        if player_index != self.current_player_index {
            return Err(PredictionError::NotYourTurn);
        }
        if prediction > self.round_number {
            return Err(PredictionError::OutOfRange {
                max: self.round_number,
            });
        }
        self.players[player_index].prediction = Some(prediction);
        self.event_log.push(GameEvent::Predicted {
            player: player_index,
            prediction,
        });
        self.current_player_index = (self.current_player_index + 1) % self.num_players;

        if self.players.iter().all(|p| p.prediction.is_some()) {
            self.game_phase = GamePhase::Playing;
            self.current_player_index = (self.dealer_index() + 1) % self.num_players;
        }
        Ok(())
    }

    pub fn play_card(
        &mut self,
        player_index: usize,
        card_index: usize,
    ) -> Result<(), PlayCardError> {
        if self.game_phase != GamePhase::Playing {
            return Err(PlayCardError::WrongPhase);
        }
        if player_index != self.current_player_index {
            return Err(PlayCardError::NotYourTurn);
        }
        let hand_size = self.players[player_index].hand.len();
        if card_index >= hand_size {
            return Err(PlayCardError::BadIndex {
                index: card_index,
                hand_size,
            });
        }

        let lead = lead_suit_in_trick(&self.current_trick);
        let candidate = &self.players[player_index].hand[card_index];
        if violates_follow_suit(candidate, &self.players[player_index].hand, lead) {
            return Err(PlayCardError::MustFollowSuit);
        }

        let card = self.players[player_index].hand.remove(card_index);
        self.event_log.push(GameEvent::CardPlayed {
            player: player_index,
            card: card.clone(),
        });
        self.current_trick.push((player_index, card));

        if self.current_trick.len() == self.num_players {
            self.resolve_trick();
        } else {
            self.current_player_index = (self.current_player_index + 1) % self.num_players;
        }
        Ok(())
    }

    fn resolve_trick(&mut self) {
        let trumpf_suit = self.trumpf.as_ref().map(|c| &c.suit);
        let winner = determine_trick_winner(&self.current_trick, trumpf_suit);

        // update player
        self.players[winner].tricks_won += 1;

        // update current player
        self.current_trick_starter = winner;
        self.current_player_index = winner;

        // update trick
        self.current_trick.clear();
        self.event_log.push(GameEvent::TrickWon { winner });

        if self.players.iter().all(|p| p.hand.is_empty()) {
            self.score_round();
        }
    }

    fn score_round(&mut self) {
        let mut deltas = Vec::with_capacity(self.players.len());
        for p in &mut self.players {
            let prediction = p.prediction.unwrap_or(0) as i32;
            let tricks = p.tricks_won as i32;
            let delta = if prediction == tricks {
                20 + 10 * prediction
            } else {
                -10 * (prediction - tricks).abs()
            };
            p.points += delta;
            deltas.push(delta);
        }
        let scored_round = self.round_number;
        self.event_log.push(GameEvent::RoundScored {
            round: scored_round,
            deltas,
        });

        if self.round_number >= self.max_rounds() {
            self.game_phase = GamePhase::GameOver;
            self.event_log.push(GameEvent::GameOver);
        } else {
            self.start_new_round();
        }
    }
}

fn lead_suit_in_trick(trick: &[(usize, Card)]) -> Option<&Suit> {
    if trick[0].1.is_wizard() {
        return None;
    }

    trick
        .iter()
        .find(|(_, c)| !c.is_wizard() && !c.is_jester())
        .map(|(_, c)| &c.suit)
}

fn violates_follow_suit(card: &Card, hand: &[Card], lead: Option<&Suit>) -> bool {
    if card.is_wizard() || card.is_jester() {
        return false;
    }
    let Some(lead) = lead else {
        return false;
    };
    let has_lead = hand
        .iter()
        .any(|c| !c.is_wizard() && !c.is_jester() && &c.suit == lead);
    has_lead && &card.suit != lead
}

/// Returns indices into the player's hand that are legal plays this trick.
#[must_use]
pub fn legal_card_indices(game: &WizardGame, player_index: usize) -> Vec<usize> {
    let hand = game.players()[player_index].hand();
    let lead = lead_suit_in_trick(game.current_trick());
    let any_lead = hand
        .iter()
        .any(|c| !c.is_wizard() && !c.is_jester() && Some(&c.suit) == lead);
    hand.iter()
        .enumerate()
        .filter(|(_, c)| {
            if c.is_wizard() || c.is_jester() {
                true
            } else if let Some(lead_suit) = lead {
                !any_lead || &c.suit == lead_suit
            } else {
                true
            }
        })
        .map(|(i, _)| i)
        .collect()
}

fn determine_trick_winner(trick: &[(usize, Card)], trumpf_suit: Option<&Suit>) -> usize {
    // first wizard wins outright
    if let Some((player, _)) = trick.iter().find(|(_, c)| c.is_wizard()) {
        return *player;
    }
    let lead_suit = lead_suit_in_trick(trick);
    let scored: Vec<(usize, u8, usize)> = trick
        .iter()
        .map(|(player, card)| {
            let (tier, val) = if card.is_jester() {
                (0u8, 0)
            } else if Some(&card.suit) == trumpf_suit {
                (3, card.value)
            } else if Some(&card.suit) == lead_suit {
                (2, card.value)
            } else {
                (1, card.value)
            };
            (*player, tier, val)
        })
        .collect();
    let best = scored
        .iter()
        .max_by(|a, b| (a.1, a.2).cmp(&(b.1, b.2)))
        .expect("trick is non-empty when resolving");
    best.0
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn add_player_succeeds() {
        let mut lobby = WizardGame::new(3, "Nyuchen");

        let hase = Player::new("Wuschelhase");
        lobby.add_player(hase).unwrap();

        assert!(
            lobby
                .players
                .into_iter()
                .eq(vec![Player::new("Nyuchen"), Player::new("Wuschelhase")])
        );
    }

    #[test]
    fn add_player_full_lobby_fails() {
        let mut lobby = WizardGame::new(3, "CallMeJooooooo");
        lobby.add_player(Player::new("Justus_Fluegel")).unwrap();
        lobby.add_player(Player::new("gast_lurksAALot")).unwrap();

        assert_eq!(
            lobby.add_player(Player::new("Adri86rose")).unwrap_err(),
            AddPlayerError::LobbyFull { max_players: 3 }
        );
    }

    #[test]
    fn add_player_name_taken_fails() {
        let mut lobby = WizardGame::new(3, "kaipai5");
        assert_eq!(
            lobby.add_player(Player::new("kaipai5")).unwrap_err(),
            AddPlayerError::NameTaken {
                name: "kaipai5".to_string()
            }
        );
    }

    #[test]
    fn deal_cards_per_round() {
        let mut lobby = WizardGame::new(3, "lil_maeve");
        lobby.add_player(Player::new("ThePseud0")).unwrap();
        lobby.add_player(Player::new("kredit0r")).unwrap();

        lobby.start_new_round();
        assert_eq!(lobby.round_number, 1);
        for player in &lobby.players {
            assert_eq!(player.hand.len(), 1);
        }

        lobby.start_new_round();
        assert_eq!(lobby.round_number, 2);
        for player in &lobby.players {
            assert_eq!(player.hand.len(), 2);
        }
    }

    /// drive a full round of round_number 1 (one card each) by playing the first legal card.
    #[test]
    fn round_one_plays_to_completion() {
        let mut game = WizardGame::new(3, "a");
        game.add_player(Player::new("b")).unwrap();
        game.add_player(Player::new("c")).unwrap();
        game.start_game();

        // resolve choose_trumpf if needed by picking blue arbitrarily
        if *game.game_phase() == GamePhase::ChooseTrumpf {
            game.choose_trumpf(Suit::Blue);
        }
        assert_eq!(*game.game_phase(), GamePhase::Prediction);

        // each player predicts 0
        for _ in 0..3 {
            let idx = game.current_player_index();
            game.place_prediction(idx, 0).unwrap();
        }
        assert_eq!(*game.game_phase(), GamePhase::Playing);

        // each player plays the first card in their hand
        for _ in 0..3 {
            let idx = game.current_player_index();
            let legal = legal_card_indices(&game, idx);
            assert!(!legal.is_empty());
            game.play_card(idx, legal[0]).unwrap();
        }

        // after round 1 finishes, scoring runs and round 2 begins
        assert_eq!(game.round_number(), 2);
        assert!(matches!(
            game.game_phase(),
            GamePhase::ChooseTrumpf | GamePhase::Prediction
        ));
        // someone won, someone got penalized for predicting 0
        let any_nonzero = game.players().iter().any(|p| p.points() != 0);
        assert!(any_nonzero, "scoring should mark prediction hits/misses");
    }

    #[test]
    fn follow_suit_enforced() {
        let mut game = WizardGame::new(3, "a");
        game.add_player(Player::new("b")).unwrap();
        game.add_player(Player::new("c")).unwrap();
        game.start_game();
        if *game.game_phase() == GamePhase::ChooseTrumpf {
            game.choose_trumpf(Suit::Blue);
        }

        // jump straight into a fabricated playing state we can reason about
        for _ in 0..3 {
            let idx = game.current_player_index();
            game.place_prediction(idx, 0).unwrap();
        }
        assert_eq!(*game.game_phase(), GamePhase::Playing);

        // Construct a hand for current player with a non-special card; pretend the trick lead
        // a specific suit they have — then a different-suit non-wizard play should be rejected.
        let cp = game.current_player_index();
        // Replace hands and trick deterministically.
        game.players[cp].hand = vec![Card::new(Suit::Red, 5), Card::new(Suit::Blue, 5)];
        game.current_trick = vec![((cp + 2) % 3, Card::new(Suit::Red, 3))];
        // Playing blue while holding red and red was led should error
        let err = game.play_card(cp, 1).unwrap_err();
        assert_eq!(err, PlayCardError::MustFollowSuit);
        // Playing red (index 0) is legal
        game.play_card(cp, 0).unwrap();
    }
}
