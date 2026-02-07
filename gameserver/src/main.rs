use gameserver::gamelogic::WizardGame;

fn main() {
    let game = WizardGame::new(3, "spooky".to_string());
    dbg!(game);
}


