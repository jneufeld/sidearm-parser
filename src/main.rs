use std::num::ParseIntError;
use std::str::FromStr;

use regex::Regex;

// Hand parser for text files at:
// http://web.archive.org/web/20110205042259/http://www.outflopped.com/questions/286/obfuscated-datamined-hand-histories
fn main() {
    let hands = raw("hands-full.txt");
    let hands = parse(hands);

    println!("Hands: {}", hands.len());
    //println!("{:#?}", hands);
}

#[derive(Clone, Debug)]
struct Hand {
    game: Game,
    stake: Amount,
    seats: Vec<Seat>
}

impl Hand {
    fn new(game: Game, stake: Amount, seats: Vec<Seat>) -> Hand {
        Hand { game, stake, seats }
    }

    fn default() -> Hand {
        Hand { game: Game::Unknown(String::from("Not Yet Created")), stake: Amount::default(), seats: vec!() }
    }
}

#[derive(Copy, Clone, Debug)]
struct Amount {
    integer: u32,
    fraction: u8
}

impl Amount {
    fn default() -> Amount {
        Amount { integer: 0, fraction: 0 }
    }
}

impl FromStr for Amount {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
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

#[derive(Clone, Debug)]
enum Game {
    Unknown(String),
    NoLimitHoldem,
    NoLimitHoldemHeadsUp
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

#[derive(Clone, Debug)]
struct Seat {
    number: u8,
    player_id: String,
    stack: Amount
}

impl Seat {
    fn new(number: u8, player_id: String, stack: Amount) -> Seat {
        Seat { number, player_id, stack }
    }
}

fn raw(path: &str) -> String {
    std::fs::read_to_string(path).unwrap()
}

fn parse(raw: String) -> Vec<Hand> {
    let mut hands = Vec::new();

    let begin_re = Regex::new(r"Stage #\d+: (?P<game>.+) \$(?P<stake>\d+)[ ,].*").unwrap();
    let seat_re = Regex::new(r"Seat (?P<number>\d+) - (?P<player_id>.+) \(\$(?P<stack>.+) in chips\)").unwrap();

    let mut current_hand = Hand::default();

    for line in raw.split_terminator('\n') {
        match begin_re.captures(line) {
            None => (),
            Some(captures) => {
                let game = captures.name("game").unwrap().as_str();
                current_hand.game = Game::from(game);

                let stake = captures.name("stake").unwrap().as_str();
                current_hand.stake = stake.parse::<Amount>().unwrap();
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
            }
        };

        if line.trim().len() == 0  && current_hand.seats.len() != 0 {
            hands.push(current_hand.clone());
            current_hand = Hand::default();
        }
    }

    hands
}