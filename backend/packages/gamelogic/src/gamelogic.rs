#![allow(unused)]
use rand::{Rng, seq::SliceRandom};
use rand_distr::Alphanumeric;
use std::fmt::Display;
use thiserror::Error;

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

#[derive(Debug, PartialEq, Eq)]
enum GamePhase {
    NotStarted,
    ChooseTrumpf,
    Prediction,
    Playing,
    RoundEnd,
    GameOver,
}

#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord)]
enum Suit {
    Red,
    Yellow,
    Green,
    Blue,
}

#[derive(Debug, PartialEq, Eq, Clone, PartialOrd, Ord)]
struct Card {
    suit: Suit,
    value: usize,
}

impl Card {
    pub fn new(suit: Suit, value: usize) -> Self {
        Card { suit, value }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Deck {
    deck: [Card; 60],
}

impl Deck {
    pub fn new() -> Self {
        let deck: [Card; 60] = std::array::from_fn(|i| {
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

    /// shuffle cards
    fn shuffle(&mut self) {
        self.deck.shuffle(&mut rand::rng());
    }

    fn deal_cards(&self, players: &mut Vec<Player>, round_number: usize) -> Option<Card> {
        for (player_index, player) in players.into_iter().enumerate() {
            for i in 0..round_number {
                player
                    .hand
                    .push(self.deck[player_index * round_number + i].clone());
            }
        }
        if players.len() * round_number  < self.deck.len() {
            return Some(self.deck[players.len() * round_number].clone())
        };
        return None
    }
}


#[derive(Debug, PartialEq, Eq)]
pub struct Player {
    name: String,
    hand: Vec<Card>,
    prediction: usize,
    tricks_won: usize,
    points: usize,
}

impl Player {
    pub fn new(name: impl Into<String>) -> Self {
        Player {
            name: name.into(),
            hand: Vec::new(),
            prediction: 0,
            tricks_won: 0,
            points: 0,
        }
    }

    pub fn reset(&mut self) {
        self.hand = Vec::new();
        self.prediction = 0;
        self.tricks_won = 0;
    }

    pub fn sort_hand(&mut self) {
        self.hand.sort();
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct WizardGame {
    // players
    num_players: usize,
    players: Vec<Player>,
    current_player_index: usize,

    // round
    game_phase: GamePhase,
    round_number: usize,

    // Cards
    deck: Deck,
    trumpf: Option<Card>,
    trumpf_chooser_index: Option<usize>,
    trick_starter: Option<Player>,
    current_trick: Vec<Card>,
}

impl WizardGame {
    /// Creates new gamestate
    ///
    /// # Examples
    ///
    /// ```
    /// use gameserver::gamelogic::WizardGame;
    /// let game = WizardGame::new(3, "JARV1S");
    /// ```
    #[must_use]
    pub fn new(num_players: usize, player_name: impl Into<String>) -> Self {
        let new_player = Player::new(player_name);

        Self {
            // players
            num_players,
            players: vec![new_player],
            current_player_index: 0,

            // round
            game_phase: GamePhase::NotStarted,
            round_number: 0,

            // Cards
            deck: Deck::new(),
            trumpf: None,
            trumpf_chooser_index: None,
            trick_starter: None,
            current_trick: Vec::new(),
        }
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
    /// use gameserver::gamelogic::WizardGame;
    ///
    /// let game = WizardGame::new(3, "Justus");
    /// let new_player = Player::new("TruelyMostWanted")
    /// game.add_player(new_player);
    /// ```
    ///
    /// # Errors
    ///
    ///
    pub fn add_player(&mut self, player: Player) -> Result<(), AddPlayerError> {
        // lobby full
        if self.players.len() >= self.num_players {
            return Err(AddPlayerError::LobbyFull {
                max_players: self.num_players,
            });
        }

        // name already exists
        if self.players.iter().any(|other| player.name == other.name) {
            return Err(AddPlayerError::NameTaken { name: player.name });
        }

        // add new player
        self.players.push(player);

        Ok(())
    }

    /// start game
    pub fn start_game(&mut self) {
        self.start_new_round();
    }

    // Game Phase NOT STARTED
    fn start_new_round(&mut self) {
        self.round_number += 1;
        self.current_player_index = self.round_number % self.players.len() - 1;
        self.deck.shuffle();

        // reset players
        for player in &mut self.players {
            player.reset();
        }

        // reset WizardGame
        self.current_trick = Vec::new();
        self.trick_starter = None; 

        // deal cards and get trumpf card 
        self.trumpf = self.deck.deal_cards(&mut self.players, self.round_number);

        // trumpf == WIZARD? -> set new Game Phase
        if let Some(card) = &self.trumpf && card.value == 14 {
            self.game_phase = GamePhase::ChooseTrumpf;
            if self.current_player_index == 0{
                self.trumpf_chooser_index = Some(self.players.len() - 1);
            } else {
                self.trumpf_chooser_index = Some(self.current_player_index - 1);
            }
            
        } else {
            self.game_phase = GamePhase::Prediction;
        }
    }

    fn choose_trumpf(&mut self, suit: Suit) {
        self.trumpf = Some(Card::new(suit, 14));
        self.game_phase = GamePhase::Prediction;
    }

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
    fn deal_cards() {
        let mut lobby = WizardGame::new(3, "lil_maeve");
        lobby.add_player(Player::new("ThePseud0"));
        lobby.add_player(Player::new("kredit0r"));

        lobby.start_new_round();
        lobby.start_new_round();

        dbg!(lobby.players);

        


        
    }
}
