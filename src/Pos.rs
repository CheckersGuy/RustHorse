const BLACK: i32 = -1;
const WHITE: i32 = 1;

const BIT_BOARD: [i32; 32] = [
    18, 12, 6, 0, 19, 13, 7, 1, 26, 20, 14, 8, 27, 21, 15, 9, 2, 28, 22, 16, 3, 29, 23, 17, 10, 4,
    30, 24, 11, 5, 31, 25,
];
const BOARD_BIT: [i32; 32] = [
    0, 6, 12, 18, 1, 7, 13, 19, 8, 14, 20, 26, 9, 15, 21, 27, 16, 22, 28, 2, 17, 23, 29, 3, 24, 30,
    4, 10, 25, 31, 5, 11,
];

const LEFT_BORDER: u32 = (1 << 18) | (1 << 26) | (1 << 2) | (1 << 10);
const RIGHT_BORDER: u32 = (1 << 1) | (1 << 9) | (1 << 17) | (1 << 25);

const BRANK_BLACK: u32 = (1 << 18) | (1 << 12) | (1 << 6) | (1 << 0);
const WRANK_WHITE: u32 = (1 << 11) | (1 << 5) | (1 << 31) | (1 << 25);
const NOT_ROT_LEFT: u32 = WRANK_WHITE | LEFT_BORDER;
const NOT_ROT_RIGHT: u32 = BRANK_BLACK | RIGHT_BORDER;
pub struct Position {
    pub bp: u32,
    pub wp: u32,
    pub k: u32,
    pub color: i32,
}

pub struct Move {
    from: u32,
    to: u32,
    captures: u32,
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

    pub fn empty() -> Position {
        return Position {
            bp: 0,
            wp: 0,
            k: 0,
            color: BLACK,
        };
    }

    //below alwas from the perspective of the player to move
    //the rest needs to be implemented
    pub fn move_left<const COLOR: i32>(maske: u32) -> u32 {
        if COLOR == BLACK {
            return (maske & (!NOT_ROT_LEFT)).rotate_left(7);
        } else {
            return (maske & (!NOT_ROT_RIGHT)).rotate_right(7);
        }
    }

    pub fn move_right<const COLOR: i32>(maske: u32) -> u32 {
        if COLOR == BLACK {
            return (maske & (!NOT_ROT_LEFT)).rotate_left(1);
        } else {
            return (maske & (!NOT_ROT_RIGHT)).rotate_right(1);
        }
    }
    pub fn get_movers<const COLOR: i32>(&self) -> u32 {
        let nocc: u32 = !(self.bp | self.wp);
        let mut movers: u32 = 0;
        if self.k != 0 {
            movers |= Position::move_left::<COLOR>(nocc);
            movers |= Position::move_right::<COLOR>(nocc);
        }
        if COLOR == BLACK {
            movers |= Position::move_left::<WHITE>(nocc);
            movers |= Position::move_right::<WHITE>(nocc);
            movers &= self.bp;
        } else {
            movers |= Position::move_left::<BLACK>(nocc);
            movers |= Position::move_right::<BLACK>(nocc);
            movers &= self.wp;
        }

        return movers;
    }

    pub fn get_jumpers<const COLOR: i32>(&self) -> u32 {
        return 0u32;
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

impl PartialEq for Position {
    fn eq(&self, other: &Self) -> bool {
        return self.bp == other.bp
            && self.wp == other.wp
            && self.k == other.k
            && self.color == other.color;
    }
}

impl PartialEq for Move {
    fn eq(&self, other: &Self) -> bool {
        return self.captures == other.captures && self.from == other.from && self.to == other.to;
    }
}
