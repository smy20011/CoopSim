use std::{collections::HashMap, usize};

use rand::{seq::SliceRandom, thread_rng};

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub enum Action {
    Coop,
    Deflect,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum Strategy {
    Deflect,
    TicToc,
    Coop,
    Random,
}

impl Strategy {
    fn get_action(self: &Self, history: &Vec<Action>) -> Action {
        let mut rand = thread_rng();
        match *self {
            Strategy::Deflect => Action::Deflect,
            Strategy::TicToc => history.last().unwrap_or(&Action::Coop).to_owned(),
            Strategy::Coop => Action::Coop,
            Strategy::Random => [Action::Coop, Action::Deflect]
                .choose(&mut rand)
                .unwrap()
                .to_owned(),
        }
    }
}

type Coord = (usize, usize);

pub struct Agent {
    coord: Coord,
    history: HashMap<Coord, Vec<Action>>,
    strategy: Strategy,
    score: f32,
}

impl Agent {
    fn adapt(self: &mut Self, neighbors: Vec<&Agent>) {
        let best_neighbor = neighbors
            .into_iter()
            .max_by(|a, b| a.score.partial_cmp(&b.score).unwrap());

        if let Some(n) = best_neighbor {
            if n.score > self.score {
                self.strategy = n.strategy;
            }
        }
    }

    fn get_action(self: &Self, agent: &Agent) -> Action {
        let empty: Vec<Action> = vec![];
        let history: &Vec<Action> = self.history.get(&agent.coord).unwrap_or(&empty);
        self.strategy.get_action(history)
    }

    fn score(self: &mut Self, agnet: &Agent, other_action: Action, score: f32) {
        if let Some(history) = self.history.get_mut(&agnet.coord) {
            history.push(other_action);
        } else {
            self.history.insert(agnet.coord, vec![other_action]);
        }
        self.score += score;
    }

    fn new(coord: Coord, strategy: Strategy) -> Agent {
        Agent {
            coord,
            history: HashMap::new(),
            strategy,
            score: 0.0,
        }
    }
}

pub struct Metric {
    coop_agents: i32,
    coop_actions: i32,
}

pub struct Environment {
    num_row: usize,
    num_col: usize,
    grid: Vec<Agent>,
}

impl Environment {
    pub fn step(&mut self) -> Metric {
        self.for_each_cell(|curr, neighbors| {
            curr.adapt(neighbors);
        });
        let mut actions: HashMap<(Coord, Coord), Action> = HashMap::new();
        self.for_each_cell(|curr, neighbors| {
            for n in neighbors {
                actions.insert((curr.coord, n.coord), curr.get_action(n));
            }
        });
        self.for_each_cell(|curr, neighbors| {
            for n in neighbors {
                let my_action = actions[&(curr.coord, n.coord)];
                let their_action = actions[&(n.coord, curr.coord)];
                curr.score(n, their_action, Environment::score(my_action, their_action));
            }
        });

        let mut coop_agents = 0;
        self.for_each_cell(|curr, _| {
            if curr.strategy == Strategy::Coop {
                coop_agents += 1;
            }
        });

        let coop_actions = actions.values().filter(|a| (**a == Action::Coop)).count() as i32;

        Metric {
            coop_actions,
            coop_agents,
        }
    }

    fn score(a: Action, b: Action) -> f32 {
        match (a, b) {
            (Action::Coop, Action::Coop) => 3.0,
            (Action::Deflect, Action::Coop) => 4.0,
            _ => 0.0,
        }
    }

    fn for_each_cell<F>(&mut self, mut f: F)
    where
        F: FnMut(&mut Agent, Vec<&Agent>),
    {
        for x in 0..self.num_row {
            for y in 0..self.num_col {
                // Make sure all indicies are unique.
                let mut all_coords: Vec<Coord> = self
                    .neighbor_coord((x, y))
                    .into_iter()
                    .chain(vec![(x, y)])
                    .collect();
                all_coords.sort();
                let before_dedup = all_coords.len();
                all_coords.dedup();
                let after_dedpup = all_coords.len();
                assert_eq!(before_dedup, after_dedpup);
                // Make sure all index are valid.
                all_coords
                    .into_iter()
                    .map(|c| self.to_vec_index(c))
                    .for_each(|i| assert!(i < self.grid.len()));

                // SAFETY: All indexes are within grid.len and are unique.
                unsafe {
                    let ptr = self.grid.as_mut_ptr();
                    let current = ptr.add(self.to_vec_index((x, y))).as_mut().unwrap();
                    let agents: Vec<&Agent> = self
                        .neighbor_coord((x, y))
                        .iter()
                        .map(|c| ptr.add(self.to_vec_index(*c)).as_ref().unwrap())
                        .collect();
                    f(current, agents);
                }
            }
        }
    }

    fn neighbor_coord(&self, coord: Coord) -> Vec<Coord> {
        let mut result: Vec<Coord> = Vec::new();
        let upper = |bond: usize| move |s: usize| (s < bond).then_some(s);
        for dx in [-1, 0, 1] {
            for dy in [-1, 0, 1] {
                if dx != 0 || dy != 0 {
                    let x = coord.0.checked_add_signed(dx).and_then(upper(self.num_row));
                    let y = coord.1.checked_add_signed(dy).and_then(upper(self.num_col));

                    if let (Some(x), Some(y)) = (x, y) {
                        result.push((x, y));
                    }
                }
            }
        }
        result
    }

    fn to_vec_index(&self, coord: Coord) -> usize {
        self.num_col * coord.0 + coord.1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strategy() {
        let deflect_history = vec![Action::Deflect];
        let coop_history = vec![Action::Coop];
        for history in [deflect_history, coop_history] {
            assert_eq!(
                Strategy::TicToc.get_action(&history),
                *history.last().unwrap()
            );
            assert_eq!(Strategy::Coop.get_action(&history), Action::Coop);
            assert_eq!(Strategy::Deflect.get_action(&history), Action::Deflect);
        }
    }
}
