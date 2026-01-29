use std::fmt::Display;

#[derive(PartialEq, Eq)]
enum GamePhase {
    NotStarted,
    ChooseTrumpf,
    Prediction,
    Playing,
    RoundEnd, 
    GameOver,
}



#[derive(PartialEq, Eq)]
enum Suit {
    Red,
    Blue,
    Green,
    Yellow,
}


struct Card {
    suit: Suit,
    value: usize,
}

impl PartialEq for Card {
    fn eq(&self, other: &Self) -> bool {
        if self.suit == other.suit {
            if self.value == other.value {
                return true;
            }
        }
        return false;
    }
}

impl Card {
    pub fn new(suit: Suit, value: usize) -> Self{
        Card {
            suit,
            value
        }
    }
}

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
            name: name,
            hand: Vec::new(),
            prediction: 0, 
            tricks_won: 0, 
            points: 0,
        }
    }
}


struct WizardGame {
    id: String,

    // players
    num_players: usize,
    players: Vec<Player>,
    current_player_index: usize,

    // round
    game_phase: GamePhase, 
    round_number: usize,

    // Cards
    deck: Vec<Card>,
    trumpf: Option<Suit>, 
    waiting_for_trumpf_choice: bool, 
    trumpf_chooser: Option<Player>,
    trick_starter: Option<Player>, 
    current_trick: Vec<Card>,
}

impl WizardGame {
    pub fn new(num_players: usize, player_name: String) -> Self {
        let new_player = Player::new(player_name);
        
        Self {
            id: "abc".to_string(),

            // players
            num_players: num_players,
            players: vec![new_player],
            current_player_index: 0,

            // round
            game_phase: GamePhase::NotStarted, 
            round_number: 0,

            // Cards
            deck: Self::create_deck(),
            trumpf: None, 
            waiting_for_trumpf_choice: false, 
            trumpf_chooser: None,
            trick_starter: None, 
            current_trick: Vec::new(),
        }
    }


    // not started
    fn create_deck() -> Vec<Card> {
        let mut cards = Vec::new();
        for value in 0..15 {
            cards.push(Card::new(Suit::Blue, value));
            cards.push(Card::new(Suit::Red, value));
            cards.push(Card::new(Suit::Green, value));
            cards.push(Card::new(Suit::Yellow, value));
        }
        cards

    }









}

