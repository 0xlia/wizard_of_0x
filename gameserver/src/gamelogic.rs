#![allow(unused)]
use std::{fmt::Display};
use rand::{Rng, seq::SliceRandom};
use rand_distr::Alphanumeric;
use thiserror::Error;


#[derive(Debug, Error, PartialEq, Eq)]
pub enum GameLogicError {
    #[error(transparent)]
    AddPlayer(#[from] AddPlayerError)
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum AddPlayerError {
    #[error("The lobby was already at max capacity {max_players}/{max_players}.")]
    LobbyFull{
        max_players: usize
    },

    #[error("{name} already taken.")]
    NameTaken{
        name: String
    }
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


#[derive(Debug, PartialEq, Eq)]
enum Suit {
    Red,
    Yellow,
    Green,
    Blue,
}

#[derive(Debug, PartialEq, Eq)]
struct Card {
    suit: Suit,
    value: usize,
}

impl Card {
    pub fn new(suit: Suit, value: usize) -> Self{
        Card {
            suit,
            value
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Deck {
    deck: [Card; 60]
}

impl Deck {
    fn new() -> Self {
        let deck: [Card; 60] = std::array::from_fn(|i| Card::new(match i % 4 {
            0 => Suit::Blue,
            1 => Suit::Red,
            2 => Suit::Green,
            3 => Suit::Yellow,
            _ => unreachable!(),
        }, i / 4));

        dbg!(&deck);
        Self { deck }
    }

    /// shuffle cards
    fn shuffle(&mut self){
        self.deck.shuffle(&mut rand::rng());
    }
}

    

#[derive(Debug, PartialEq, Eq)]
struct Player {
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
}


#[derive(Debug, PartialEq, Eq)]
pub struct WizardGame {
    id: String,

    // players
    num_players: usize,
    players: Vec<Player>,
    current_player_index: usize,

    // round
    game_phase: GamePhase, 
    round_number: usize,

    // Cards
    deck: Deck,
    trumpf: Option<Suit>, 
    waiting_for_trumpf_choice: bool, 
    trumpf_chooser: Option<Player>,
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
            id: Self::create_id(),

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
            waiting_for_trumpf_choice: false, 
            trumpf_chooser: None,
            trick_starter: None,
            current_trick: Vec::new(),
        }
    }

    /// PHASE: `NOT_STARTED`
    /// returns random ID
    fn create_id() -> String {
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
    pub fn add_player(&mut self, player: Player) -> Result<(), AddPlayerError>{
        // lobby full
        if self.players.len() >= self.num_players {
            return Err(AddPlayerError::LobbyFull { max_players: self.num_players })
        }

        // name already exists
        if self.players.iter().any(|other| player.name == other.name) {
            return Err(AddPlayerError::NameTaken { name: player.name })
        }

        // add new player
        self.players.push(player);
        
        Ok(())
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

        assert!(lobby.players.into_iter().eq(vec![Player::new("Nyuchen"), Player::new("Wuschelhase")]));
    }

    #[test]
    fn add_player_full_lobby_fails() {
        let mut lobby = WizardGame::new(3, "CallMeJooooooo");
        lobby.add_player(Player::new("Justus_Fluegel")).unwrap();
        lobby.add_player(Player::new("gast_lurksAALot")).unwrap();
        
        assert_eq!(
            lobby.add_player(Player::new("Adri86rose")).unwrap_err(),
            AddPlayerError::LobbyFull { max_players: 3}
        );
    }

    #[test]
    fn add_player_name_taken_fails() {
        let mut lobby = WizardGame::new(3, "kaipai5");
        assert_eq!(
            lobby.add_player(Player::new("kaipai5")).unwrap_err(),
            AddPlayerError::NameTaken { name: "kaipai5".to_string() }
        );
    }
}