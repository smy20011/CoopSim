use std::{
    collections::{BTreeMap, HashMap},
    usize,
};

use crate::agent::{Action, Agent, Coord, Strategy};

pub struct Environment {
    num_row: usize,
    num_col: usize,
    noise: f32,
    grid: Vec<Agent>,
}

#[derive(Debug, Clone)]
pub struct Metric {
    pub strategies: BTreeMap<Strategy, usize>,
    pub max_score: BTreeMap<Strategy, f32>,
    pub coop_actions: i32,
    pub snapshot: Vec<Vec<Strategy>>,
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
        let noise = self.noise;
        self.for_each_cell(|curr, neighbors| {
            for n in neighbors {
                let my_action = actions[&(curr.coord, n.coord)].with_noise(noise);
                let their_action = actions[&(n.coord, curr.coord)].with_noise(noise);
                curr.score(n, their_action, Environment::score(my_action, their_action));
            }
        });

        let mut strategies: BTreeMap<Strategy, usize> = BTreeMap::new();
        let mut max_score: BTreeMap<Strategy, f32> = BTreeMap::new();
        self.grid.iter().for_each(|curr| {
            let count = strategies.get(&curr.strategy).cloned().unwrap_or(0) + 1;
            strategies.insert(curr.strategy, count);

            let score = max_score.get(&curr.strategy).cloned().unwrap_or(0.0);
            max_score.insert(curr.strategy, score.max(curr.score));
        });
        let snapshot: Vec<Vec<Strategy>> = self
            .grid
            .iter()
            .map(|a| a.strategy)
            .collect::<Vec<Strategy>>()
            .chunks(self.num_col)
            .map(|c| c.iter().cloned().collect())
            .collect();

        let coop_actions = actions.values().filter(|a| (**a == Action::Coop)).count() as i32;

        Metric {
            coop_actions,
            strategies,
            max_score,
            snapshot,
        }
    }

    pub fn new(num_row: usize, num_col: usize, noise: f32) -> Environment {
        let strategies = vec![Strategy::Deflect, Strategy::TicToc];
        Environment::new_with_agent_func(num_row, num_col, noise, |c| Agent::random(c, &strategies))
    }

    pub fn new_with_agent_func<F>(
        num_row: usize,
        num_col: usize,
        noise: f32,
        mut agent_fn: F,
    ) -> Environment
    where
        F: FnMut(Coord) -> Agent,
    {
        let mut grid: Vec<Agent> = Vec::with_capacity(num_row * num_col);
        for i in 0..num_row {
            for j in 0..num_col {
                grid.push(agent_fn((i, j)));
            }
        }

        Environment {
            num_row,
            num_col,
            noise,
            grid,
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
