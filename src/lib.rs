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

#[derive(Clone, Copy, Debug, EnumIter, Hash, Eq, Ord, PartialEq, PartialOrd)]
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
    OnePair(Rank, u16),
    TwoPair(Rank, Rank, Rank),
    Set(Rank, u16),
    Straight(Rank),
    Flush(u16),
    FullHouse(Rank, Rank),
    Quads(Rank, Rank),
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
    bitmask: u16,
    suit_map: HashMap<Suit, u8>,
    rank_map: HashMap<Rank, u8>,
}

impl Hand {
    pub fn new(hole_cards: &[Card], board_cards: &[Card]) -> Hand {
        let mut cards: Vec<Card> = Vec::with_capacity(7);
        let mut suit_map = HashMap::with_capacity(4);
        let mut rank_map = HashMap::with_capacity(7);
        let mut bitmask: u16 = 0x00;

        cards.extend(hole_cards);
        cards.extend(board_cards);

        for card in &cards {
            let count = suit_map.entry(card.suit).or_insert(0);
            *count += 1;

            let count = rank_map.entry(card.rank).or_insert(0);
            *count += 1;

            bitmask |= 1 << card.score();
            if card.score() == 13 {
                // If Ace also set bit 1
                bitmask |= 0x01;
            }
        }

        Hand {
            cards,
            bitmask,
            suit_map,
            rank_map,
        }
    }

    fn check_flush(&self) -> Option<HandRanking> {
        let mut flush_suit: Option<Suit> = None;

        for (suit, count) in &self.suit_map {
            if *count >= 5 {
                flush_suit = Some(*suit);
                break;
            }
        }

        if let Some(suit) = flush_suit {
            // For flush cards
            let mut bitmask = 0x00;

            for card in &self.cards {
                if card.suit == suit {
                    bitmask |= 1 << card.score();

                    // Also set bit 1 if Ace
                    if card.score() == 13 {
                        bitmask |= 0x01;
                    }
                }
            }

            if let Some(card) = self.best_straight(bitmask) {
                if card == Rank::Ace {
                    Some(HandRanking::RoyalFlush)
                } else {
                    Some(HandRanking::StraightFlush(card))
                }
            } else {
                bitmask = self.highcards(bitmask, 5);

                Some(HandRanking::Flush(bitmask))
            }
        } else {
            None
        }
    }

    // Returns the high card rank of the best possible straight, None if no straight found
    fn best_straight(&self, bitmask: u16) -> Option<Rank> {
        let mut card: Option<Rank> = None;

        // There are ten possible straights, check from highest to lowest
        for i in (0..10).rev() {
            if bitmask & 0x1F << i == 0x1F << i {
                card = Some(Rank::id(i + 4));
                break;
            }
        }

        card
    }

    // Get bitmask representation of the high cards to be used in HandRanking
    fn highcards(&self, mut bitmask: u16, bits_needed: u8) -> u16 {
        // Make sure bit 1 is unset
        bitmask &= !0x01;

        let mut bits_set = self.bits_set(bitmask);

        while bits_set > bits_needed {
            bitmask &= bitmask - 1; // Unset lsb
            bits_set -= 1;
        }

        bitmask
    }

    // Count number of bits that are set in bitmask
    fn bits_set(&self, mut bitmask: u16) -> u8 {
        let mut count = 0;

        while bitmask != 0 {
            bitmask &= bitmask - 1;
            count += 1;
        }

        count
    }

    fn best(&self) -> HandRanking {
        let mut pair: Vec<Rank> = Vec::with_capacity(3);
        let mut set: Vec<Rank> = Vec::with_capacity(2);

        for (card, count) in &self.rank_map {
            match count {
                2 => pair.push(*card),
                3 => set.push(*card),

                // Check for Quads
                4 => {
                    let bitmask = self.bitmask ^ 1 << card.score(); // Unset quads bit
                    let id = (bitmask as f64).log2() as u8; // Find most significant bit

                    return HandRanking::Quads(*card, Rank::id(id));
                }
                _ => (),
            }
        }

        // Check for FullHouse
        if set.len() == 2 {
            set.sort();
            set.reverse();

            return HandRanking::FullHouse(set[0], set[1]);
        }
        if set.len() == 1 && !pair.is_empty() {
            pair.sort();
            pair.reverse();

            return HandRanking::FullHouse(set[0], pair[0]);
        }

        // Check for Flush and StraightFlush
        if let Some(flush) = self.check_flush() {
            return flush;
        }

        // Check for Straight
        if let Some(card) = self.best_straight(self.bitmask) {
            return HandRanking::Straight(card);
        }

        // Make sure bit 1 is unset, as it is no longer needed
        let mut bitmask = self.bitmask;

        // Check for Set
        if set.len() == 1 {
            bitmask ^= 1 << set[0].score(); // Unset set1 bit

            bitmask = self.highcards(bitmask, 2); // Two high cards

            return HandRanking::Set(set[0], bitmask);
        }

        // Check for TwoPair
        if pair.len() > 1 {
            pair.sort();
            pair.reverse();

            bitmask ^= 1 << pair[0].score(); // Unset pair1 bit
            bitmask ^= 1 << pair[1].score(); // Unset pair2 bit
            let id = (bitmask as f64).log2() as u8; // Find most significant bit

            return HandRanking::TwoPair(pair[0], pair[1], Rank::id(id));
        }

        // Check for OnePair
        if !pair.is_empty() {
            bitmask ^= 1 << pair[0].score(); // Unset pair1 bit

            bitmask = self.highcards(bitmask, 3); // Three high cards

            return HandRanking::OnePair(pair[0], bitmask);
        }

        // None of the above means HighCard
        bitmask = self.highcards(bitmask, 5); // Five high cards

        HandRanking::HighCard(bitmask)
    }

    pub fn display(&self) {
        println!("Hand is {:?}", self.cards)
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
        assert!(
            HandRanking::Quads(Rank::Queen, Rank::Jack)
                < HandRanking::Quads(Rank::Queen, Rank::King)
        );
        assert!(
            HandRanking::FullHouse(Rank::King, Rank::Two)
                > HandRanking::FullHouse(Rank::Two, Rank::Seven)
        );
        assert!(
            HandRanking::Set(Rank::Two, 0b10_0001_1010_0010)
                < HandRanking::Set(Rank::Three, 0b01_0001_1010_0010)
        );
        assert!(
            HandRanking::TwoPair(Rank::Ace, Rank::Eight, Rank::Six)
                > HandRanking::TwoPair(Rank::Ace, Rank::Eight, Rank::Five)
        );
        assert!(
            HandRanking::OnePair(Rank::Ace, 0b10_0001_1010_1100)
                < HandRanking::OnePair(Rank::Ace, 0b10_0010_1010_1100)
        );
        assert!(
            HandRanking::HighCard(0b10_0001_1010_1100) < HandRanking::HighCard(0b10_0010_1010_1100)
        );
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
            Some(HandRanking::Flush(0b01_0001_1101_0000))
        );
    }

    #[test]
    fn check_flush_ranking() {
        let hole1 = [
            Card {
                rank: Rank::Seven,
                suit: Suit::Hearts,
            },
            Card {
                rank: Rank::Eight,
                suit: Suit::Hearts,
            },
        ];
        let hole2 = [
            Card {
                rank: Rank::Six,
                suit: Suit::Hearts,
            },
            Card {
                rank: Rank::Five,
                suit: Suit::Hearts,
            },
        ];
        let board = [
            Card {
                rank: Rank::King,
                suit: Suit::Hearts,
            },
            Card {
                rank: Rank::Queen,
                suit: Suit::Hearts,
            },
            Card {
                rank: Rank::Ace,
                suit: Suit::Hearts,
            },
            Card {
                rank: Rank::Jack,
                suit: Suit::Hearts,
            },
            Card {
                rank: Rank::Nine,
                suit: Suit::Hearts,
            },
        ];

        let hand1 = Hand::new(&hole1, &board);
        let hand2 = Hand::new(&hole2, &board);
        assert_eq!(hand1.check_flush(), hand2.check_flush());
    }

    #[test]
    fn check_straight_flush() {
        let hole1 = [
            Card {
                rank: Rank::Two,
                suit: Suit::Hearts,
            },
            Card {
                rank: Rank::Three,
                suit: Suit::Hearts,
            },
        ];
        let hole2 = [
            Card {
                rank: Rank::Ten,
                suit: Suit::Hearts,
            },
            Card {
                rank: Rank::Jack,
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
                rank: Rank::Four,
                suit: Suit::Hearts,
            },
            Card {
                rank: Rank::Ace,
                suit: Suit::Hearts,
            },
            Card {
                rank: Rank::Queen,
                suit: Suit::Hearts,
            },
        ];

        let hand1 = Hand::new(&hole1, &board);
        assert_eq!(
            hand1.check_flush(),
            Some(HandRanking::StraightFlush(Rank::Five))
        );
        let hand2 = Hand::new(&hole2, &board);
        assert_eq!(hand2.check_flush(), Some(HandRanking::RoyalFlush));
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
        assert_eq!(hand.best(), HandRanking::Straight(Rank::Five));
    }

    #[test]
    fn check_quads() {
        let hole = [
            Card {
                rank: Rank::King,
                suit: Suit::Hearts,
            },
            Card {
                rank: Rank::King,
                suit: Suit::Clubs,
            },
        ];
        let board = [
            Card {
                rank: Rank::King,
                suit: Suit::Diamonds,
            },
            Card {
                rank: Rank::King,
                suit: Suit::Spades,
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
        assert_eq!(hand.best(), HandRanking::Quads(Rank::King, Rank::Nine));
    }

    #[test]
    fn check_full_house_two_sets() {
        let hole = [
            Card {
                rank: Rank::Two,
                suit: Suit::Hearts,
            },
            Card {
                rank: Rank::Two,
                suit: Suit::Clubs,
            },
        ];
        let board = [
            Card {
                rank: Rank::King,
                suit: Suit::Diamonds,
            },
            Card {
                rank: Rank::Two,
                suit: Suit::Spades,
            },
            Card {
                rank: Rank::King,
                suit: Suit::Hearts,
            },
            Card {
                rank: Rank::King,
                suit: Suit::Clubs,
            },
            Card {
                rank: Rank::Nine,
                suit: Suit::Clubs,
            },
        ];

        let hand = Hand::new(&hole, &board);
        assert_eq!(hand.best(), HandRanking::FullHouse(Rank::King, Rank::Two));
    }

    #[test]
    fn check_full_house_set_and_pairs() {
        let hole = [
            Card {
                rank: Rank::Two,
                suit: Suit::Hearts,
            },
            Card {
                rank: Rank::Two,
                suit: Suit::Clubs,
            },
        ];
        let board = [
            Card {
                rank: Rank::Six,
                suit: Suit::Diamonds,
            },
            Card {
                rank: Rank::Two,
                suit: Suit::Spades,
            },
            Card {
                rank: Rank::Six,
                suit: Suit::Hearts,
            },
            Card {
                rank: Rank::Seven,
                suit: Suit::Clubs,
            },
            Card {
                rank: Rank::Seven,
                suit: Suit::Spades,
            },
        ];

        let hand = Hand::new(&hole, &board);
        assert_eq!(hand.best(), HandRanking::FullHouse(Rank::Two, Rank::Seven));
    }

    #[test]
    fn check_set() {
        let hole1 = [
            Card {
                rank: Rank::Ace,
                suit: Suit::Hearts,
            },
            Card {
                rank: Rank::Two,
                suit: Suit::Clubs,
            },
        ];
        let hole2 = [
            Card {
                rank: Rank::Ace,
                suit: Suit::Spades,
            },
            Card {
                rank: Rank::Four,
                suit: Suit::Clubs,
            },
        ];
        let board = [
            Card {
                rank: Rank::Ace,
                suit: Suit::Diamonds,
            },
            Card {
                rank: Rank::Ace,
                suit: Suit::Clubs,
            },
            Card {
                rank: Rank::Eight,
                suit: Suit::Hearts,
            },
            Card {
                rank: Rank::Nine,
                suit: Suit::Clubs,
            },
            Card {
                rank: Rank::King,
                suit: Suit::Spades,
            },
        ];

        let hand1 = Hand::new(&hole1, &board);
        let hand2 = Hand::new(&hole2, &board);
        assert_eq!(
            hand1.best(),
            HandRanking::Set(Rank::Ace, 0b01_0001_0000_0000)
        );
        assert_eq!(hand1.best(), hand2.best());
    }

    #[test]
    fn check_two_pair() {
        let hole = [
            Card {
                rank: Rank::Ace,
                suit: Suit::Hearts,
            },
            Card {
                rank: Rank::Ace,
                suit: Suit::Clubs,
            },
        ];
        let board = [
            Card {
                rank: Rank::Six,
                suit: Suit::Diamonds,
            },
            Card {
                rank: Rank::Six,
                suit: Suit::Spades,
            },
            Card {
                rank: Rank::Eight,
                suit: Suit::Hearts,
            },
            Card {
                rank: Rank::Eight,
                suit: Suit::Clubs,
            },
            Card {
                rank: Rank::Three,
                suit: Suit::Spades,
            },
        ];

        let hand = Hand::new(&hole, &board);
        assert_eq!(
            hand.best(),
            HandRanking::TwoPair(Rank::Ace, Rank::Eight, Rank::Six)
        );
    }

    #[test]
    fn check_one_pair() {
        let hole = [
            Card {
                rank: Rank::Ace,
                suit: Suit::Hearts,
            },
            Card {
                rank: Rank::Nine,
                suit: Suit::Clubs,
            },
        ];
        let board = [
            Card {
                rank: Rank::Ace,
                suit: Suit::Diamonds,
            },
            Card {
                rank: Rank::Six,
                suit: Suit::Spades,
            },
            Card {
                rank: Rank::Eight,
                suit: Suit::Hearts,
            },
            Card {
                rank: Rank::Four,
                suit: Suit::Clubs,
            },
            Card {
                rank: Rank::Three,
                suit: Suit::Spades,
            },
        ];

        let hand = Hand::new(&hole, &board);
        assert_eq!(
            hand.best(),
            HandRanking::OnePair(Rank::Ace, 0b00_0001_1010_0000)
        );
    }

    #[test]
    fn check_high_card() {
        let hole = [
            Card {
                rank: Rank::Ace,
                suit: Suit::Hearts,
            },
            Card {
                rank: Rank::Nine,
                suit: Suit::Clubs,
            },
        ];
        let board = [
            Card {
                rank: Rank::King,
                suit: Suit::Diamonds,
            },
            Card {
                rank: Rank::Six,
                suit: Suit::Spades,
            },
            Card {
                rank: Rank::Eight,
                suit: Suit::Hearts,
            },
            Card {
                rank: Rank::Four,
                suit: Suit::Clubs,
            },
            Card {
                rank: Rank::Three,
                suit: Suit::Spades,
            },
        ];

        let hand = Hand::new(&hole, &board);
        assert_eq!(hand.best(), HandRanking::HighCard(0b11_0001_1010_0000));
    }
}
