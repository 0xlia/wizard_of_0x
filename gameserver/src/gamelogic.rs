#![allow(unused)]
use std::{fmt::Display};
use rand::{Rng, seq::SliceRandom};
use rand_distr::Alphanumeric;
use thiserror::Error;

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

#[derive(Debug)]
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

    

#[derive(Debug)]
struct Player {
    name: String, 
    hand: Vec<Card>,
    prediction: usize,
    tricks_won: usize, 
    points: usize,
}

impl Player {
    pub fn new(name: String) -> Self {
        Player {
            name,
            hand: Vec::new(),
            prediction: 0, 
            tricks_won: 0, 
            points: 0,
        }
    }
}


#[derive(Debug)]
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
    /// let game = WizardGame::new(3, "JARV1S".to_string());
    /// ```
    #[must_use]
    pub fn new(num_players: usize, player_name: String) -> Self {
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
    fn add_player(&mut self, player_name: String) -> Result<(), ()>{
        assert!(self.players.len() < self.num_players);
        Ok(())
    }








}

