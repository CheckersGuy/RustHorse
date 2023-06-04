const BLACK: i32 = -1;
const WHITE: i32 = 1;

const BIT_BOARD: [u32; 32] = [
    18, 12, 6, 0, 19, 13, 7, 1, 26, 20, 14, 8, 27, 21, 15, 9, 2, 28, 22, 16, 3, 29, 23, 17, 10, 4,
    30, 24, 11, 5, 31, 25,
];
const BOARD_BIT: [u32; 32] = [
    0, 6, 12, 18, 1, 7, 13, 19, 8, 14, 20, 26, 9, 15, 21, 27, 16, 22, 28, 2, 17, 23, 29, 3, 24, 30,
    4, 10, 25, 31, 5, 11,
];

const BRANK_BLACK: u32 = (1 << 18) | (1 << 12) | (1 << 6) | (1 << 0);
const BRANK_WHITE: u32 = (1 << 11) | (1 << 5) | (1 << 31) | (1 << 25);

const NOT_RL_7: u32 = (1 << 18) | (1 << 26) | (1 << 2) | (1 << 10);
const NOT_RR_1: u32 = NOT_RL_7;
const NOT_RL_1: u32 = (1 << 1) | (1 << 9) | (1 << 17) | (1 << 25);
const NOT_RR_7: u32 = NOT_RL_1;

#[derive(PartialEq, Default, Clone, Copy)]
pub struct Position {
    pub bp: u32,
    pub wp: u32,
    pub k: u32,
    pub color: i32,
}

#[derive(PartialEq, Default, Copy, Clone)]
pub struct Move {
    from: u32,
    to: u32,
    captures: u32,
}
pub struct MoveList {
    pub moves: [Move; 40],
    pub length: usize,
}

fn move_left<const COLOR: i32>(maske: u32) -> u32 {
    if COLOR == BLACK {
        (maske & (!NOT_RL_7) & (!BRANK_WHITE)).rotate_left(7)
    } else {
        (maske & (!NOT_RR_7) & (!BRANK_BLACK)).rotate_right(7)
    }
}

fn move_right<const COLOR: i32>(maske: u32) -> u32 {
    if COLOR == BLACK {
        (maske & (!NOT_RL_1) & (!BRANK_WHITE)).rotate_left(1)
    } else {
        (maske & (!NOT_RR_1) & (!BRANK_BLACK)).rotate_right(1)
    }
}

impl Position {
    pub fn print_position(&self) {
        for i in (0..8).rev() {
            for j in 0..4 {
                let maske: u32 = 1u32 << (BIT_BOARD[4 * i + j]);
                let value: i32 = ((maske & self.bp) != 0) as i32
                    + (((maske & self.wp) != 0) as i32) * 2i32
                    + (((maske & self.k) != 0) as i32) * 3i32;
                if i % 2 == 1 {
                    print!("[ ]");
                }
                match value {
                    1i32 => print!("[0]"),
                    2i32 => print!("[X]"),
                    4i32 => print!("[B]"),
                    5i32 => print!("[W]"),
                    _ => print!("[ ]"),
                }
                if i % 2 == 0 {
                    print!("[ ]");
                }
            }
            println!();
        }
    }

    pub fn make_move(&mut self, m: &Move) {
        if self.color == BLACK {
            self.bp &= !m.from;
            self.bp |= m.to;
            self.wp &= !m.captures;
        } else {
            self.wp &= !m.from;
            self.wp |= m.to;
            self.bp &= !m.captures;
        }
        //need to check for promotion
        //and other things
        if self.k != 0 {}
    }

    pub fn undo_mvoe(&mut self, m: &Move) {
        //to be implemented
    }

    //need to understand the borrow trait
    pub fn empty() -> Position {
        Position {
            color: BLACK,
            ..Position::default()
        }
    }

    pub fn get_movers<const COLOR: i32>(&self) -> u32 {
        let nocc: u32 = !(self.bp | self.wp);
        let mut movers: u32 = 0;
        if self.k != 0 {
            movers |= move_left::<COLOR>(nocc);
            movers |= move_right::<COLOR>(nocc);
        }
        if COLOR == BLACK {
            movers |= move_left::<WHITE>(nocc);
            movers |= move_right::<WHITE>(nocc);
            movers &= self.bp;
        } else {
            movers |= move_left::<BLACK>(nocc);
            movers |= move_right::<BLACK>(nocc);
            movers &= self.wp;
        }

        return movers;
    }

    pub fn get_jumpers<const COLOR: i32>(&self) -> u32 {
        let nocc: u32 = !(self.bp | self.wp);
        let mut movers: u32 = 0;
        let opp: u32 = if COLOR == BLACK { self.wp } else { self.bp };
        let own: u32 = if COLOR == BLACK { self.bp } else { self.wp };
        if COLOR == BLACK {
            movers |= move_left::<WHITE>(move_left::<WHITE>(nocc) & self.wp);
            movers |= move_right::<WHITE>(move_right::<WHITE>(nocc) & self.wp);
            movers &= self.bp;
        } else {
            movers |= move_left::<BLACK>(move_left::<BLACK>(nocc) & self.bp);
            movers |= move_right::<BLACK>(move_right::<BLACK>(nocc) & self.bp);
            movers &= self.wp;
        }
        if self.k != 0 {
            movers |= move_left::<COLOR>(move_left::<COLOR>(nocc) & opp);
            movers |= move_right::<COLOR>(move_right::<COLOR>(nocc) & opp);
            movers &= own & self.k;
        }
        return movers;
    }

    pub fn get_start_position() -> Position {
        let mut start: Position = Position::empty();
        for i in 0..12 {
            start.bp |= 1 << BOARD_BIT[i];
        }
        for i in 20..32 {
            start.wp |= 1 << BOARD_BIT[i];
        }
        start.k = 0u32;
        return start;
    }
}
impl MoveList {
    pub fn empty_list() -> MoveList {
        MoveList {
            length: 0,
            moves: [Move::default(); 40],
        }
    }

    fn add_quiet_move(&mut self, from: u32, to: u32) {
        let scrap: usize = (to == 0) as usize;
        self.moves[self.length + scrap] = Move {
            from,
            to,
            captures: 0,
        };
        self.length += (to != 0) as usize;
    }

    pub fn get_silent_movers<const COLOR: i32, const OPP: i32>(&mut self, pos: &Position) {
        let movers = pos.get_movers::<COLOR>();
        let nocc = !(pos.bp | pos.wp);
        let mut pawns = movers & !pos.k;
        let mut kings = movers & pos.k;
        while pawns != 0 {
            let from = pawns & !(pawns - 1u32);
            self.add_quiet_move(from, move_left::<COLOR>(from) & nocc);
            self.add_quiet_move(from, move_right::<COLOR>(from) & nocc);
            pawns &= pawns - 1;
        }
        while kings != 0 {
            let from = kings & !(kings - 1u32);
            self.add_quiet_move(from, move_left::<COLOR>(from) & nocc);
            self.add_quiet_move(from, move_right::<COLOR>(from) & nocc);
            self.add_quiet_move(from, move_left::<OPP>(from) & nocc);
            self.add_quiet_move(from, move_right::<OPP>(from) & nocc);
            kings &= kings - 1;
        }
    }

    fn try_capture<const COLOR: i32, const OPP: i32>(from: u32, pos: Position) {}

    pub fn get_captures<const COLOR: i32, const OPP: i32>(&mut self, pos: Position) {
        let jumpers = pos.get_jumpers::<COLOR>();
        let mut pawns = jumpers & !pos.k;
        let mut kings = jumpers & pos.k;
        while pawns != 0 {
            let from = pawns & !(pawns - 1u32);

            pawns &= pawns - 1;
        }
    }

    pub fn iter(&mut self) -> std::slice::Iter<'_, Move> {
        (&self.moves[0..self.length]).iter()
    }
}
