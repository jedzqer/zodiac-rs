use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Camp {
    Red,
    Black,
}

impl Camp {
    pub fn opponent(self) -> Camp {
        match self {
            Camp::Red => Camp::Black,
            Camp::Black => Camp::Red,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Piece {
    pub name: String,
    pub lv: u8,
    pub camp: Camp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MoveResult {
    Move,
    Eat,
    Pong,
    Cow,
    Sheep,
    Chicken,
    Dog,
    Monkey,
    MonkeyMove,
    MonkeyEat,
    MonkeyPong,
}

pub fn create_all_pieces() -> Vec<Piece> {
    let names: [(u8, &str); 12] = [
        (1, "鼠"), (2, "牛"), (3, "虎"), (4, "兔"),
        (5, "龙"), (6, "蛇"), (7, "马"), (8, "羊"),
        (9, "猴"), (10, "鸡"), (11, "狗"), (12, "猪"),
    ];
    let mut pieces = Vec::with_capacity(24);
    for camp in [Camp::Red, Camp::Black] {
        for &(lv, name) in &names {
            pieces.push(Piece { name: name.to_string(), lv, camp });
        }
    }
    pieces
}

impl Piece {
    pub fn is_valid_move(
        &self,
        _target_pos: (usize, usize),
        target_info: (Option<u8>, bool),
        _self_pos: (usize, usize),
        path_state: Option<(bool, bool)>,
        target_camp: Option<Camp>,
        dx: usize,
        dy: usize,
    ) -> Option<MoveResult> {
        match self.lv {
            1 => self.move_mouse(target_info, target_camp, dx, dy),
            2 => self.move_cow(target_info, path_state, target_camp, dx, dy),
            3 => self.move_tiger(target_info, path_state, target_camp, dx, dy),
            4 => self.move_rabbit(target_info, target_camp, dx, dy),
            5 => self.move_dragon(target_info, target_camp, dx, dy),
            6 => self.move_snake(target_info, target_camp, dx, dy),
            7 => self.move_horse(target_info, path_state, target_camp, dx, dy),
            8 => self.move_sheep(target_info, target_camp, dx, dy),
            9 => self.move_monkey(target_info, target_camp, dx, dy),
            10 => self.move_chicken(target_info, target_camp, dx, dy),
            11 => self.move_dog(target_info, target_camp, dx, dy),
            12 => self.move_pig(target_info, target_camp, dx, dy),
            _ => None,
        }
    }

    fn move_mouse(
        &self,
        target_info: (Option<u8>, bool),
        target_camp: Option<Camp>,
        dx: usize,
        dy: usize,
    ) -> Option<MoveResult> {
        if dx + dy != 1 {
            return None;
        }
        match target_info.0 {
            None => Some(MoveResult::Move),
            Some(lv) => {
                if !target_info.1 {
                    return None;
                }
                let tc = target_camp?;
                if self.camp == tc {
                    return None;
                }
                if lv == 1 {
                    Some(MoveResult::Pong)
                } else if lv != 12 {
                    Some(MoveResult::Eat)
                } else {
                    None
                }
            }
        }
    }

    fn move_cow(
        &self,
        target_info: (Option<u8>, bool),
        path_state: Option<(bool, bool)>,
        target_camp: Option<Camp>,
        dx: usize,
        dy: usize,
    ) -> Option<MoveResult> {
        if dx == 0 && dy == 0 {
            return None;
        }
        if dx == 0 || dy == 0 {
            if dx + dy == 1 {
                return match target_info.0 {
                    None => Some(MoveResult::Move),
                    Some(lv) => {
                        if !target_info.1 {
                            return None;
                        }
                        let tc = target_camp?;
                        if self.camp == tc {
                            return None;
                        }
                        if lv > 2 {
                            Some(MoveResult::Eat)
                        } else if lv == 2 {
                            Some(MoveResult::Pong)
                        } else {
                            None
                        }
                    }
                };
            }
            if dx + dy == 2 && target_info.0.is_none() {
                if let Some((has_piece, revealed)) = path_state {
                    if has_piece && !revealed {
                        return Some(MoveResult::Cow);
                    }
                }
            }
        }
        None
    }

    fn move_tiger(
        &self,
        target_info: (Option<u8>, bool),
        path_state: Option<(bool, bool)>,
        target_camp: Option<Camp>,
        dx: usize,
        dy: usize,
    ) -> Option<MoveResult> {
        if dx == 0 || dy == 0 {
            if dx + dy == 1 {
                return match target_info.0 {
                    None => Some(MoveResult::Move),
                    Some(lv) => {
                        if !target_info.1 {
                            return None;
                        }
                        let tc = target_camp?;
                        if self.camp == tc {
                            return None;
                        }
                        if lv > 3 {
                            Some(MoveResult::Eat)
                        } else if lv == 3 {
                            Some(MoveResult::Pong)
                        } else {
                            None
                        }
                    }
                };
            }
            if dx + dy == 2 {
                if let Some((has_piece, _)) = path_state {
                    if has_piece {
                        if !target_info.1 || target_info.0.is_none() {
                            return None;
                        }
                        let tc = target_camp?;
                        if self.camp == tc {
                            return None;
                        }
                        let lv = target_info.0.unwrap();
                        if lv > 3 {
                            return Some(MoveResult::Eat);
                        } else if lv == 3 {
                            return Some(MoveResult::Pong);
                        }
                    }
                }
            }
        }
        None
    }

    fn move_rabbit(
        &self,
        target_info: (Option<u8>, bool),
        target_camp: Option<Camp>,
        dx: usize,
        dy: usize,
    ) -> Option<MoveResult> {
        if dx == 0 || dy == 0 {
            if dx + dy == 1 || dx + dy == 2 {
                return match target_info.0 {
                    None => Some(MoveResult::Move),
                    Some(lv) => {
                        if !target_info.1 {
                            return None;
                        }
                        let tc = target_camp?;
                        if self.camp == tc {
                            return None;
                        }
                        if lv > 4 {
                            Some(MoveResult::Eat)
                        } else if lv == 4 {
                            Some(MoveResult::Pong)
                        } else {
                            None
                        }
                    }
                };
            }
        }
        None
    }

    fn move_dragon(
        &self,
        target_info: (Option<u8>, bool),
        target_camp: Option<Camp>,
        dx: usize,
        dy: usize,
    ) -> Option<MoveResult> {
        if dx == 0 && dy == 0 {
            return None;
        }
        match target_info.0 {
            None => Some(MoveResult::Move),
            Some(lv) => {
                if !target_info.1 {
                    return None;
                }
                let tc = target_camp?;
                if self.camp == tc {
                    return None;
                }
                if lv > 5 {
                    Some(MoveResult::Eat)
                } else if lv == 5 {
                    Some(MoveResult::Pong)
                } else {
                    None
                }
            }
        }
    }

    fn move_snake(
        &self,
        target_info: (Option<u8>, bool),
        target_camp: Option<Camp>,
        dx: usize,
        dy: usize,
    ) -> Option<MoveResult> {
        if dx > 1 || dy > 1 || (dx == 0 && dy == 0) {
            return None;
        }
        match target_info.0 {
            None => Some(MoveResult::Move),
            Some(lv) => {
                if !target_info.1 {
                    return None;
                }
                let tc = target_camp?;
                if self.camp == tc {
                    return None;
                }
                if lv > 6 {
                    Some(MoveResult::Eat)
                } else if lv == 6 {
                    Some(MoveResult::Pong)
                } else {
                    None
                }
            }
        }
    }

    fn move_horse(
        &self,
        target_info: (Option<u8>, bool),
        path_state: Option<(bool, bool)>,
        target_camp: Option<Camp>,
        dx: usize,
        dy: usize,
    ) -> Option<MoveResult> {
        if dx == 0 && dy == 0 {
            return None;
        }
        let check_target = |lv: u8| -> Option<MoveResult> {
            if !target_info.1 {
                return None;
            }
            let tc = target_camp?;
            if self.camp == tc {
                return None;
            }
            if lv > 7 {
                Some(MoveResult::Eat)
            } else if lv == 7 {
                Some(MoveResult::Pong)
            } else {
                None
            }
        };

        if dx + dy == 1 {
            return match target_info.0 {
                None => Some(MoveResult::Move),
                Some(lv) => check_target(lv),
            };
        }
        if (dx == 0 || dy == 0) && dx + dy == 2 {
            if let Some((has_piece, _)) = path_state {
                if !has_piece {
                    return match target_info.0 {
                        None => Some(MoveResult::Move),
                        Some(lv) => check_target(lv),
                    };
                }
            }
        }
        None
    }

    fn move_sheep(
        &self,
        target_info: (Option<u8>, bool),
        target_camp: Option<Camp>,
        dx: usize,
        dy: usize,
    ) -> Option<MoveResult> {
        if dx + dy == 1 {
            return match target_info.0 {
                None => Some(MoveResult::Move),
                Some(lv) => {
                    if !target_info.1 {
                        return None;
                    }
                    let tc = target_camp?;
                    if self.camp == tc {
                        return None;
                    }
                    if lv > 8 {
                        Some(MoveResult::Eat)
                    } else if lv == 8 {
                        Some(MoveResult::Pong)
                    } else {
                        None
                    }
                }
            };
        }
        if dx + dy < 3 {
            if let Some(lv) = target_info.0 {
                let tc = target_camp?;
                if target_info.1 && lv < self.lv && self.camp != tc {
                    return Some(MoveResult::Sheep);
                }
            }
        }
        None
    }

    fn move_monkey(
        &self,
        target_info: (Option<u8>, bool),
        target_camp: Option<Camp>,
        dx: usize,
        dy: usize,
    ) -> Option<MoveResult> {
        if dx + dy != 1 {
            return None;
        }
        match target_info.0 {
            None => Some(MoveResult::MonkeyMove),
            Some(lv) => {
                if !target_info.1 {
                    return Some(MoveResult::Monkey);
                }
                let tc = target_camp?;
                if self.camp == tc {
                    return None;
                }
                if lv > 9 {
                    Some(MoveResult::MonkeyEat)
                } else if lv == 9 {
                    Some(MoveResult::MonkeyPong)
                } else {
                    None
                }
            }
        }
    }

    fn move_chicken(
        &self,
        target_info: (Option<u8>, bool),
        target_camp: Option<Camp>,
        dx: usize,
        dy: usize,
    ) -> Option<MoveResult> {
        if dx == 0 && dy == 0 {
            return Some(MoveResult::Chicken);
        }
        if dx + dy != 1 {
            return None;
        }
        match target_info.0 {
            None => Some(MoveResult::Move),
            Some(lv) => {
                if !target_info.1 {
                    return None;
                }
                let tc = target_camp?;
                if self.camp == tc {
                    return None;
                }
                if lv > 10 {
                    Some(MoveResult::Eat)
                } else if lv == 10 {
                    Some(MoveResult::Pong)
                } else {
                    None
                }
            }
        }
    }

    fn move_dog(
        &self,
        target_info: (Option<u8>, bool),
        target_camp: Option<Camp>,
        dx: usize,
        dy: usize,
    ) -> Option<MoveResult> {
        if dx + dy != 1 {
            return None;
        }
        match target_info.0 {
            None => Some(MoveResult::Move),
            Some(lv) => {
                if !target_info.1 {
                    return None;
                }
                let tc = target_camp?;
                if self.camp == tc {
                    return None;
                }
                if lv == 12 {
                    Some(MoveResult::Eat)
                } else if lv == 11 {
                    Some(MoveResult::Pong)
                } else if lv < 11 {
                    Some(MoveResult::Dog)
                } else {
                    None
                }
            }
        }
    }

    fn move_pig(
        &self,
        target_info: (Option<u8>, bool),
        target_camp: Option<Camp>,
        dx: usize,
        dy: usize,
    ) -> Option<MoveResult> {
        if dx + dy != 1 {
            return None;
        }
        match target_info.0 {
            None => Some(MoveResult::Move),
            Some(lv) => {
                if !target_info.1 {
                    return None;
                }
                let tc = target_camp?;
                if self.camp == tc {
                    return None;
                }
                if lv == 12 {
                    Some(MoveResult::Pong)
                } else if lv == 1 {
                    Some(MoveResult::Eat)
                } else {
                    None
                }
            }
        }
    }
}
