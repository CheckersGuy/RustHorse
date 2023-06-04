pub mod Pos;
use Pos::Position;

use crate::Pos::MoveList;
fn main() {
    let mut position: Position = Position::get_start_position();
    position.print_position();
    println!();
    let mut liste = MoveList::empty_list();
    liste.get_silent_movers::<-1, 1>(&position);
    println!("Number of moves {}", liste.length);

    for m in liste.iter() {
        let mut other = position;
        other.make_move(m);
        other.print_position();
        println!();
    }
}
