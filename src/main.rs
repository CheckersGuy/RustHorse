pub mod Pos;
use Pos::Position;
fn main() {
    let mut position: Position = Position::get_start_position();
    position.print_position();
    println!();
    let movers = position.get_movers::<-1>();
    let mut temp = Position::empty();
    temp.bp = movers;
    temp.print_position();
}
