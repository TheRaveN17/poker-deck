use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::HashMap;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Clone, Copy, Debug, EnumIter, Hash, Eq, Ord, PartialEq, PartialOrd)]
enum Suit {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
}

#[derive(Clone, Copy, Debug, EnumIter, Eq, Ord, PartialEq, PartialOrd)]
enum Rank {
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

impl Rank {
    fn score(self) -> u8 {
        match self {
            Rank::Two => 1,
            Rank::Three => 2,
            Rank::Four => 3,
            Rank::Five => 4,
            Rank::Six => 5,
            Rank::Seven => 6,
            Rank::Eight => 7,
            Rank::Nine => 8,
            Rank::Ten => 9,
            Rank::Jack => 10,
            Rank::Queen => 11,
            Rank::King => 12,
            Rank::Ace => 13,
        }
    }

    fn id(score: u8) -> Self {
        match score {
            1 => Rank::Two,
            2 => Rank::Three,
            3 => Rank::Four,
            4 => Rank::Five,
            5 => Rank::Six,
            6 => Rank::Seven,
            7 => Rank::Eight,
            8 => Rank::Nine,
            9 => Rank::Ten,
            10 => Rank::Jack,
            11 => Rank::Queen,
            12 => Rank::King,
            13 => Rank::Ace,
            _ => panic!("No such card with score {}", score),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd)]
enum HandRanking {
    HighCard(u16),
    Pair,
    TwoPair,
    Set,
    Straight(Rank),
    Flush(u16),
    FullHouse,
    Quads,
    StraightFlush(Rank),
    RoyalFlush,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Card {
    rank: Rank,
    suit: Suit,
}

impl Card {
    fn new(rank: Rank, suit: Suit) -> Self {
        Card { rank, suit }
    }

    fn score(&self) -> u8 {
        self.rank.score()
    }

    pub fn display(&self) {
        println!("Drew card -> {:?}", self);
    }
}

#[derive(Debug)]
pub struct Deck {
    cards: Vec<Card>,
    dealt: Vec<Card>,
}

impl Deck {
    pub fn new() -> Self {
        let mut cards = Vec::with_capacity(52);
        let dealt = Vec::with_capacity(25);

        for rank in Rank::iter() {
            for suit in Suit::iter() {
                cards.push(Card::new(rank, suit));
            }
        }

        Deck { cards, dealt }
    }

    pub fn display(&self) {
        println!("{:?}", self);
    }

    pub fn shuffle(&mut self) {
        self.cards.shuffle(&mut thread_rng());
    }

    pub fn draw(&mut self, nr: u8) -> Vec<Card> {
        let mut cards = Vec::new();

        for _ in 0..nr {
            if let Some(card) = self.cards.pop() {
                self.dealt.push(card);
                cards.push(card);
            } else {
                panic!("Deck is empty!");
            }
        }

        cards
    }

    pub fn reset(&mut self) {
        self.cards.append(&mut self.dealt);
    }
}

impl Default for Deck {
    fn default() -> Self {
        Deck::new()
    }
}

#[derive(Debug)]
pub struct Hand {
    cards: Vec<Card>,
}

impl Hand {
    pub fn new(hole_cards: &[Card], board_cards: &[Card]) -> Hand {
        let mut cards = Vec::with_capacity(7);
        cards.extend(hole_cards);
        cards.extend(board_cards);

        // Sort cards from highest(Ace) to lowest(Two) so that
        // check_flush and check_straight may work as intended.
        cards.sort();
        cards.reverse();

        Hand { cards }
    }

    fn check_flush(&self) -> Option<HandRanking> {
        let mut map = HashMap::with_capacity(4);
        let mut flush_suit: Option<Suit> = None;

        for card in &self.cards {
            let count = map.entry(card.suit).or_insert(0);
            *count += 1;

            if *count == 5 {
                flush_suit = Some(card.suit);
                break;
            }
        }

        if let Some(suit) = flush_suit {
            let mut flush_cards = Vec::with_capacity(7);
            let mut bitmask = 0x00;

            for card in &self.cards {
                if card.suit == suit {
                    flush_cards.push(*card);
                    bitmask |= 1 << card.score();
                }
            }

            if let Some(result) = self.check_straight(Some(&flush_cards)) {
                Some(result)
            } else {
                Some(HandRanking::Flush(bitmask))
            }
        } else {
            None
        }
    }

    fn check_straight(&self, flush_cards: Option<&Vec<Card>>) -> Option<HandRanking> {
        let mut index: Option<u8> = None;
        let mut bitmask: u16 = 0x00;
        let mut cards: &Vec<Card> = &self.cards;
        let flush: bool;

        if let Some(i) = flush_cards {
            cards = i;
            flush = true;
        } else {
            flush = false;
        }

        for card in cards {
            if card.score() == 13 {
                // If Ace also set bit 1
                bitmask |= 0x01;
            }
            bitmask |= 1 << card.score();
        }

        // There are ten possible straights, check from highest to lowest
        for i in (0..10).rev() {
            if bitmask & 0x1F << i == 0x1F << i {
                index = Some(i + 4);
                break;
            }
        }

        if let Some(score) = index {
            let card = Rank::id(score);

            if flush {
                if card == Rank::Ace {
                    Some(HandRanking::RoyalFlush)
                } else {
                    Some(HandRanking::StraightFlush(card))
                }
            } else {
                Some(HandRanking::Straight(card))
            }
        } else {
            None
        }
    }

    pub fn display(&self) {
        println!("Hand is {:?}", self.cards)
    }
}

pub struct Table {
    players: Vec<Player>,
    pot: usize,
    deck: Deck,
    board: Vec<Card>,
}

impl Table {
    pub fn new() -> Table {
        let deck = Deck::new();
        let pot = 0;
        let players = Vec::with_capacity(10);
        let board = Vec::with_capacity(5);

        Table {
            players,
            pot,
            deck,
            board,
        }
    }

    pub fn add_player(&mut self, name: String, cash: usize) {
        self.players.push(Player::new(name, cash));
    }

    pub fn new_round(&mut self) {
        println!("Starting a new round...");
        self.reset();
        // self.preflop();
    }

    // fn preflop(&mut self) {
    //     for player in self.players {
    //         player.cards.extend(self.deck.draw(2));
    //     }
    // }

    fn reset(&mut self) {
        self.deck.reset();
        self.deck.shuffle();
        self.pot = 0;
        self.board.clear();
    }
}

struct Player {
    name: String,
    cash: usize,
    cards: Vec<Card>,
}

impl Player {
    fn new(name: String, cash: usize) -> Player {
        let cards = Vec::with_capacity(2);

        Player { name, cash, cards }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hand_ranking() {
        assert!(
            Some(HandRanking::Flush(0b01_0011_0001_0010))
                > Some(HandRanking::Flush(0b01_0001_1101_0010))
        );
        assert!(HandRanking::RoyalFlush > HandRanking::StraightFlush(Rank::King));
        assert!(HandRanking::Straight(Rank::Queen) > HandRanking::Straight(Rank::Jack));
    }

    #[test]
    fn check_flush() {
        let hole = [
            Card {
                rank: Rank::Seven,
                suit: Suit::Hearts,
            },
            Card {
                rank: Rank::Eight,
                suit: Suit::Hearts,
            },
        ];
        let board = [
            Card {
                rank: Rank::King,
                suit: Suit::Hearts,
            },
            Card {
                rank: Rank::Five,
                suit: Suit::Hearts,
            },
            Card {
                rank: Rank::Nine,
                suit: Suit::Hearts,
            },
            Card {
                rank: Rank::Two,
                suit: Suit::Hearts,
            },
            Card {
                rank: Rank::Nine,
                suit: Suit::Clubs,
            },
        ];

        let hand = Hand::new(&hole, &board);
        assert_eq!(
            hand.check_flush(),
            Some(HandRanking::Flush(0b01_0001_1101_0010))
        );
    }

    #[test]
    fn check_straight() {
        let hole = [
            Card {
                rank: Rank::Two,
                suit: Suit::Hearts,
            },
            Card {
                rank: Rank::Three,
                suit: Suit::Clubs,
            },
        ];
        let board = [
            Card {
                rank: Rank::Two,
                suit: Suit::Diamonds,
            },
            Card {
                rank: Rank::Five,
                suit: Suit::Diamonds,
            },
            Card {
                rank: Rank::Ace,
                suit: Suit::Hearts,
            },
            Card {
                rank: Rank::Five,
                suit: Suit::Clubs,
            },
            Card {
                rank: Rank::Four,
                suit: Suit::Clubs,
            },
        ];

        let hand = Hand::new(&hole, &board);
        assert_eq!(
            hand.check_straight(None),
            Some(HandRanking::Straight(Rank::Five))
        );
    }
}