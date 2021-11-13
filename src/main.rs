use holdemrmx::Deck;
use holdemrmx::Hand;

fn main() {
    let mut deck = Deck::new();
    deck.shuffle();
    let hole = deck.draw(2);
    let board = deck.draw(5);
    let hand = Hand::new(&hole, &board);
    hand.display();
}
