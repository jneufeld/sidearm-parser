use std::num::ParseIntError;
use std::str::FromStr;

use regex::Regex;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Hand {
    pub game: Game,
    pub stake: Amount,
    pub seats: Vec<Seat>,
    pub actions: Vec<Action>,
}

impl Hand {
    fn default() -> Hand {
        Hand {
            game: Game::Unknown(String::from("Not Yet Created")),
            stake: Amount::default(),
            seats: vec![],
            actions: vec![],
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Amount {
    pub integer: u32,
    pub fraction: u8,
}

impl Amount {
    fn default() -> Amount {
        Amount {
            integer: 0,
            fraction: 0,
        }
    }
}

impl FromStr for Amount {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        // Remove optional leading dollar sign (e.g. "$1,500.75" -> "1,500.75")
        let s = s.replace("$", "");

        // Remove optional commas (e.g. "1,500.75" -> "1500.75")
        let s = s.replace(",", "");

        // Split integer and fraction portions (e.g. "1500.75" -> ("1500", "75"))
        let (integer, fraction) = match s.find(".") {
            None => (s.parse::<u32>().unwrap(), 0),
            Some(decimal_index) => {
                let integer = s[..decimal_index].parse::<u32>().unwrap();
                let fraction = s[decimal_index + 1..].parse::<u8>().unwrap();

                (integer, fraction)
            }
        };

        Ok(Amount { integer, fraction })
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Game {
    Unknown(String),
    NoLimitHoldem,
    NoLimitHoldemHeadsUp,
}

impl Game {
    fn from(name: &str) -> Game {
        if name.eq("Holdem  No Limit") {
            Game::NoLimitHoldem
        } else if name.eq("Holdem (1 on 1)  No Limit") {
            Game::NoLimitHoldemHeadsUp
        } else {
            Game::Unknown(String::from(name))
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Seat {
    number: u8,
    player_id: String,
    stack: Amount,
}

impl Seat {
    fn new(number: u8, player_id: String, stack: Amount) -> Seat {
        Seat {
            number,
            player_id,
            stack,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Action {
    // Player actions
    Bet(String, Amount),
    Call(String, Amount),
    Check(String),
    Collect(String, Amount),
    Fold(String),
    Muck(String),
    Post(String, Amount),
    Raise(String, Amount, Amount),
    Show(String, Card, Card),

    // Dealer actions
    PreFlop,
    Flop(Card, Card, Card),
    Turn(Card),
    River(Card),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Card {
    pub rank: Rank,
    pub suit: Suit,
}

impl Card {
    fn new(rank: Rank, suit: Suit) -> Card {
        Card { rank, suit }
    }
}

impl FromStr for Card {
    type Err = CardParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        // The suit is always represented by one character at the end of the string (e.g. 'c' for clubs)
        let suit = &s[s.len() - 1..];
        let suit = suit.parse::<Suit>()?;

        // The rank is the first one or two characters (e.g. 'A' for ace or "10" for 10).
        let rank = &s[..s.len() - 1];
        let rank = rank.parse::<Rank>()?;

        Ok(Card::new(rank, suit))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CardParseError;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Rank {
    Ace,
    King,
    Queen,
    Jack,
    Ten,
    Nine,
    Eight,
    Seven,
    Six,
    Five,
    Four,
    Three,
    Two,
}

impl FromStr for Rank {
    type Err = CardParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        match s {
            "A" => Ok(Rank::Ace),
            "K" => Ok(Rank::King),
            "Q" => Ok(Rank::Queen),
            "J" => Ok(Rank::Jack),
            "10" => Ok(Rank::Ten),
            "9" => Ok(Rank::Nine),
            "8" => Ok(Rank::Eight),
            "7" => Ok(Rank::Seven),
            "6" => Ok(Rank::Six),
            "5" => Ok(Rank::Five),
            "4" => Ok(Rank::Four),
            "3" => Ok(Rank::Three),
            "2" => Ok(Rank::Two),
            _ => Err(CardParseError),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Suit {
    Club,
    Diamond,
    Heart,
    Spade,
}

impl FromStr for Suit {
    type Err = CardParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let s = s.trim();

        match s {
            "c" => Ok(Suit::Club),
            "d" => Ok(Suit::Diamond),
            "h" => Ok(Suit::Heart),
            "s" => Ok(Suit::Spade),
            _ => Err(CardParseError),
        }
    }
}

pub fn parse(input_file: &str) -> Vec<Hand> {
    let contents = std::fs::read_to_string(input_file).unwrap();

    let mut hands = Vec::new();

    let begin_re = Regex::new(r"Stage #\d+: (?P<game>.+) \$(?P<stake>\d+)[ ,].*").unwrap();
    let seat_re =
        Regex::new(r"Seat (?P<number>\d+) - (?P<player_id>.+) \(\$(?P<stack>.+) in chips\)")
            .unwrap();

    let bet_re = Regex::new(r"(?P<player_id>.+) - Bets \$(?P<amount>.+)").unwrap();
    let call_re = Regex::new(r"(?P<player_id>.+) - Calls \$(?P<amount>.+)").unwrap();
    let check_re = Regex::new(r"(?P<player_id>.+) - Checks").unwrap();
    let collect_re = Regex::new(r"(?P<player_id>.+) Collects \$(?P<amount>.+) from.+").unwrap();
    let fold_re = Regex::new(r"(?P<player_id>.+) - Folds").unwrap();
    let muck_re = Regex::new(r"(?P<player_id>.+) - Mucks").unwrap();
    let post_re = Regex::new(r"(?P<player_id>.+) - Posts .+ \$(?P<amount>.+)").unwrap();
    let raise_re =
        Regex::new(r"(?P<player_id>.+) - Raises \$(?P<raise>.+) to \$(?P<total>.+)").unwrap();
    let show_re =
        Regex::new(r"(?P<player_id>.+) - Shows \[(?P<card_1>.+) (?P<card_2>.+)\]").unwrap();

    let preflop_re = Regex::new(r"\*\*\* POCKET CARDS \*\*\*").unwrap();
    let flop_re =
        Regex::new(r"\*\*\* FLOP \*\*\* \[(?P<card_1>.+) (?P<card_2>.+) (?P<card_3>.+)\]").unwrap();
    let turn_re = Regex::new(r"\*\*\* TURN \*\*\* \[.+\] \[(?P<card>.+)\]").unwrap();
    let river_re = Regex::new(r"\*\*\* RIVER \*\*\* \[.+\] \[(?P<card>.+)\]").unwrap();

    let mut current_hand = Hand::default();

    for line in contents.split_terminator('\n') {
        match begin_re.captures(line) {
            None => (),
            Some(captures) => {
                let game = captures.name("game").unwrap().as_str();
                current_hand.game = Game::from(game);

                let stake = captures.name("stake").unwrap().as_str();
                current_hand.stake = stake.parse::<Amount>().unwrap();

                continue;
            }
        };

        match seat_re.captures(line) {
            None => (),
            Some(captures) => {
                let number = captures.name("number").unwrap().as_str();
                let number = number.parse::<u8>().unwrap();

                let player_id = captures.name("player_id").unwrap().as_str();
                let player_id = String::from(player_id);

                let stack = captures.name("stack").unwrap().as_str();
                let stack = stack.parse::<Amount>().unwrap();

                let seat = Seat::new(number, player_id, stack);
                current_hand.seats.push(seat);

                continue;
            }
        };

        match bet_re.captures(line) {
            None => (),
            Some(captures) => {
                let player_id = captures.name("player_id").unwrap().as_str();
                let amount = captures.name("amount").unwrap().as_str();
                let amount = amount.parse::<Amount>().unwrap();

                let action = Action::Bet(String::from(player_id), amount);

                current_hand.actions.push(action);

                continue;
            }
        };

        match call_re.captures(line) {
            None => (),
            Some(captures) => {
                let player_id = captures.name("player_id").unwrap().as_str();
                let amount = captures.name("amount").unwrap().as_str();
                let amount = amount.parse::<Amount>().unwrap();

                let action = Action::Call(String::from(player_id), amount);

                current_hand.actions.push(action);

                continue;
            }
        };

        match check_re.captures(line) {
            None => (),
            Some(captures) => {
                let player_id = captures.name("player_id").unwrap().as_str();
                let action = Action::Check(String::from(player_id));

                current_hand.actions.push(action);

                continue;
            }
        };

        match collect_re.captures(line) {
            None => (),
            Some(captures) => {
                let player_id = captures.name("player_id").unwrap().as_str();
                let amount = captures.name("amount").unwrap().as_str();
                let amount = amount.parse::<Amount>().unwrap();

                let action = Action::Collect(String::from(player_id), amount);

                current_hand.actions.push(action);

                continue;
            }
        };

        match fold_re.captures(line) {
            None => (),
            Some(captures) => {
                let player_id = captures.name("player_id").unwrap().as_str();
                let action = Action::Fold(String::from(player_id));

                current_hand.actions.push(action);

                continue;
            }
        };

        match muck_re.captures(line) {
            None => (),
            Some(captures) => {
                let player_id = captures.name("player_id").unwrap().as_str();
                let action = Action::Muck(String::from(player_id));

                current_hand.actions.push(action);

                continue;
            }
        };

        match post_re.captures(line) {
            None => (),
            Some(captures) => {
                let player_id = captures.name("player_id").unwrap().as_str();
                let amount = captures.name("amount").unwrap().as_str();
                let amount = amount.parse::<Amount>().unwrap();

                let action = Action::Post(String::from(player_id), amount);

                current_hand.actions.push(action);

                continue;
            }
        };

        match raise_re.captures(line) {
            None => (),
            Some(captures) => {
                let player_id = captures.name("player_id").unwrap().as_str();

                let raise = captures.name("raise").unwrap().as_str();
                let raise = raise.parse::<Amount>().unwrap();

                let total = captures.name("total").unwrap().as_str();
                let total = total.parse::<Amount>().unwrap();

                let action = Action::Raise(String::from(player_id), raise, total);

                current_hand.actions.push(action);

                continue;
            }
        };

        match show_re.captures(line) {
            None => (),
            Some(captures) => {
                let player_id = captures.name("player_id").unwrap().as_str();

                let card_1 = captures.name("card_1").unwrap().as_str();
                let card_1 = card_1.parse::<Card>().unwrap();

                let card_2 = captures.name("card_2").unwrap().as_str();
                let card_2 = card_2.parse::<Card>().unwrap();

                let action = Action::Show(String::from(player_id), card_1, card_2);

                current_hand.actions.push(action);

                continue;
            }
        };

        if preflop_re.is_match(line) {
            current_hand.actions.push(Action::PreFlop);
            continue;
        }

        match flop_re.captures(line) {
            None => (),
            Some(captures) => {
                let card_1 = captures.name("card_1").unwrap().as_str();
                let card_1 = card_1.parse::<Card>().unwrap();

                let card_2 = captures.name("card_2").unwrap().as_str();
                let card_2 = card_2.parse::<Card>().unwrap();

                let card_3 = captures.name("card_3").unwrap().as_str();
                let card_3 = card_3.parse::<Card>().unwrap();

                let action = Action::Flop(card_1, card_2, card_3);

                current_hand.actions.push(action);

                continue;
            }
        };

        match turn_re.captures(line) {
            None => (),
            Some(captures) => {
                let card = captures.name("card").unwrap().as_str();
                let card = card.parse::<Card>().unwrap();

                let action = Action::Turn(card);

                current_hand.actions.push(action);

                continue;
            }
        };

        match river_re.captures(line) {
            None => (),
            Some(captures) => {
                let card = captures.name("card").unwrap().as_str();
                let card = card.parse::<Card>().unwrap();

                let action = Action::River(card);

                current_hand.actions.push(action);

                continue;
            }
        };

        if line.trim().len() == 0 && current_hand.seats.len() != 0 {
            hands.push(current_hand.clone());
            current_hand = Hand::default();
        }
    }

    if current_hand.seats.len() != 0 {
        hands.push(current_hand.clone());
    }

    hands
}
