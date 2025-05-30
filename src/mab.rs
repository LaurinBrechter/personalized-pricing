use std::{collections::HashMap, fs::File};

use rand::Rng;

use crate::simulation::ProblemSettings;

/// Strategy to use for multi-armed bandit exploration
#[derive(Clone, Copy, PartialEq)]
pub enum MABStrategy {
    EpsilonGreedy,
    UCB,
    DecayingEpsilonGreedy,
}

impl std::fmt::Display for MABStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MABStrategy::EpsilonGreedy => write!(f, "EpsilonGreedy"),
            MABStrategy::UCB => write!(f, "UCB"),
            MABStrategy::DecayingEpsilonGreedy => write!(f, "DecayingEpsilonGreedy"),
        }
    }
}

#[derive(Clone)]
pub struct Arm {
    price: i32,
    average_reward: f64,
    num_pulls: usize,
}

impl Arm {
    pub fn new(id: usize, price: i32) -> Self {
        Self {
            price,
            average_reward: 0.0,
            num_pulls: 0,
        }
    }
}

pub struct MAB<'a> {
    num_arms: usize,
    arms: HashMap<usize, HashMap<usize, HashMap<usize, Arm>>>, // group_id -> time_period -> arm_id (price)
    best_arms: HashMap<usize, HashMap<usize, usize>>, // group_id -> time_period -> arm_id
    pub best_rewards: HashMap<usize, HashMap<usize, f64>>, // group_id -> time_period -> reward
    epsilon: f64,
    final_epsilon: f64,
    n_runs: usize,
    ucb_param: f64,
    strategy: MABStrategy,
    arms_per_group: usize,
    action_space: Vec<i32>,
    writer: &'a mut csv::Writer<File>,
    last_action: String,
    pub run_id: usize,
    pub config_id: usize,
}

pub struct MABSettings {
    pub min_price: f64,
    pub max_price: f64,
    pub arms_per_group: usize,
    pub epsilon: f64,
    pub final_epsilon: f64,
    pub n_runs: usize,
    pub ucb_param: f64,
    pub strategy: MABStrategy,
}

impl<'a> MAB<'a> {
    pub fn new(
        settings: &ProblemSettings,
        algorithm_settings: &MABSettings,
        writer: &'a mut csv::Writer<File>,
        run_id: usize,
        config_id: usize,
    ) -> Self {
        let mut id = 0;

        let mut action_space = Vec::new();

        for i in 0..algorithm_settings.arms_per_group {
            let price =
                algorithm_settings.min_price + (algorithm_settings.max_price - algorithm_settings.min_price) * (i as f64 / (algorithm_settings.arms_per_group - 1) as f64);
            action_space.push(price as i32);
        }
        println!("action_space: {:?}", action_space);

        let mut arms = HashMap::new();
        let mut best_rewards = HashMap::new();
        let mut best_arms = HashMap::new();

        for group_id in 0..(settings.num_predicted_groups as usize) {
            let mut group_arms = HashMap::new();
            let mut best_group_arms = HashMap::new();
            let mut best_group_rewards = HashMap::new();

            for period_id in 0..(settings.n_periods as usize) {
                let mut arms = HashMap::new();
                for arm_id in 0..algorithm_settings.arms_per_group {
                    let price = action_space[arm_id];
                    arms.insert(price as usize, Arm::new(id, price as i32));
                    id += 1;
                }

                let random_arm = action_space[rand::thread_rng().gen_range(0..algorithm_settings.arms_per_group)];
                best_group_arms.insert(period_id, random_arm as usize);
                best_group_rewards.insert(period_id, 0.0);
                group_arms.insert(period_id, arms);
            }

            arms.insert(group_id, group_arms);
            best_rewards.insert(group_id, best_group_rewards);
            best_arms.insert(group_id, best_group_arms);
        }
        Self {
            num_arms: (settings.num_predicted_groups as usize) * algorithm_settings.arms_per_group,
            arms: arms,
            best_arms,
            best_rewards,
            epsilon: algorithm_settings.epsilon,
            final_epsilon: algorithm_settings.final_epsilon,
            n_runs: algorithm_settings.n_runs,
            ucb_param: algorithm_settings.ucb_param,
            strategy: algorithm_settings.strategy,
            arms_per_group: algorithm_settings.arms_per_group,
            action_space,
            writer,
            last_action: "".to_string(),
            run_id: run_id,
            config_id: config_id,
        }
    }

    pub fn random_action(&self) -> i32 {
        let random_offset = rand::thread_rng().gen_range(0..(self.arms_per_group - 1));
        // println!("random_offset: {}", self.action_space[random_offset]);
        return self.action_space[random_offset];
    }

    // Calculate UCB score for an arm
    fn calculate_ucb_score(&self, arm: &Arm, total_pulls: usize) -> f64 {
        if arm.num_pulls == 0 {
            return f64::INFINITY; // Ensure arms with zero pulls are tried first
        }

        let exploitation = arm.average_reward;
        let exploration = (self.ucb_param * (total_pulls as f64).ln() / arm.num_pulls as f64).sqrt();

        exploitation + exploration
    }

    // Select the best arm according to UCB strategy
    fn select_ucb_arm(&self, group_id: usize, period: usize) -> i32 {
        let arms = &self.arms[&group_id][&period];

        // Calculate total number of pulls across all arms
        let total_pulls: usize = arms.values().map(|arm| arm.num_pulls).sum();

        // If no pulls yet, pick a random arm
        if total_pulls == 0 {
            self.random_action()
        } else {
            // Find arm with highest UCB score
            let (best_arm_id, _) = arms
                .iter()
                .map(|(arm_id, arm)| {
                    let ucb_score = self.calculate_ucb_score(arm, total_pulls);
                    (arm_id, ucb_score)
                })
                .max_by(|(_, score1), (_, score2)| score1.partial_cmp(score2).unwrap())
                .unwrap();

            *best_arm_id as i32
        }
    }
    
    // Calculate the current epsilon based on decay schedule
    fn get_current_epsilon(&self) -> f64 {
        if self.n_runs <= 1 {
            return self.epsilon;
        }
        
        // Linear decay from initial epsilon to final epsilon
        let progress = self.run_id as f64 / (self.n_runs - 1) as f64;
        self.epsilon - progress * (self.epsilon - self.final_epsilon)
    }
    
    // Increment the current run counter
    pub fn increment_run(&mut self) {
        if self.run_id < self.n_runs {
            self.run_id += 1;
        }
    }

    pub fn log(&self, writer: &mut csv::Writer<File>) {
        for grou_arms in self.arms.iter() {
            let group_id = grou_arms.0;
            for period_arms in grou_arms.1.iter() {
                let period_id = period_arms.0;
                let best_arm = self.best_arms.get(group_id).unwrap().get(period_id).unwrap();
                let num_pulls = self.arms.get(group_id).unwrap().get(period_id).unwrap().get(best_arm).unwrap().num_pulls;
                writer
                    .write_record(&[
                        self.config_id.to_string(),
                        self.epsilon.to_string(),
                        self.strategy.to_string(),
                        period_id.to_string(),
                        group_id.to_string(),
                        best_arm.to_string(),
                        num_pulls.to_string()
                    ])
                    .unwrap();
            }
        }
    }
}

pub trait Algorithm {
    fn get_price(&mut self, group_id: usize, visit: usize, period: usize) -> i32;
    fn update_average_reward(
        &mut self,
        group_id: usize,
        visit: usize,
        period: usize,
        reward: f64,
        price: i32,
    );
}

impl Algorithm for MAB<'_> {
    fn get_price(&mut self, group_id: usize, visit: usize, period: usize) -> i32 {
        let price = match self.strategy {
            MABStrategy::EpsilonGreedy => {
                if rand::thread_rng().gen::<f64>() < self.epsilon {
                    self.last_action = "random".to_string();
                    self.random_action()
                } else {
                    self.last_action = "best".to_string();
                    self.best_arms[&group_id][&period] as i32
                }
            }
            MABStrategy::DecayingEpsilonGreedy => {
                let current_epsilon = self.get_current_epsilon();
                if rand::thread_rng().gen::<f64>() < current_epsilon {
                    self.last_action = "random".to_string();
                    self.random_action()
                } else {
                    self.last_action = "best".to_string();
                    self.best_arms[&group_id][&period] as i32
                }
            }
            MABStrategy::UCB => {
                self.last_action = "ucb".to_string();
                self.select_ucb_arm(group_id, period)
            }
        };

        return price;
    }

    fn update_average_reward(
        &mut self,
        group_id: usize,
        visit: usize,
        period: usize,
        reward: f64,
        arm_id: i32,
    ) {
        let arms = self.arms.get_mut(&group_id).unwrap().get_mut(&period).unwrap();
        let arm = arms.get_mut(&(arm_id as usize)).unwrap();

        self.writer
            .write_record(&[
                self.config_id.to_string(),
                self.run_id.to_string(),
                period.to_string(),
                group_id.to_string(),
                visit.to_string(),
                arm_id.to_string(),
                reward.to_string(),
                self.last_action.clone(),
            ])
            .unwrap();

        arm.num_pulls += 1;
        arm.average_reward =
            (arm.average_reward * (arm.num_pulls as f64 - 1.0) + reward) / arm.num_pulls as f64;
        if arm.average_reward > *self.best_rewards.get(&group_id).unwrap().get(&period).unwrap()
        {
            self.best_rewards
                .get_mut(&group_id)
                .unwrap()
                .insert(period, arm.average_reward);
            self.best_arms
                .get_mut(&group_id)
                .unwrap()
                .insert(period, arm_id as usize);
        }
    }
}
