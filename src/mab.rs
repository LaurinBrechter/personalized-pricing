use std::{collections::HashMap, fs::File};

use rand::Rng;

use crate::simulation::ProblemSettings;
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
    arms_per_group: usize,
    action_space: Vec<i32>,
    writer: &'a mut csv::Writer<File>,
    last_action: String,
    pub run_id: usize,
    pub config_id: usize,
}

impl<'a> MAB<'a> {
    pub fn new(
        settings: &ProblemSettings,
        min_price: f64,
        max_price: f64,
        arms_per_group: usize,
        epsilon: f64,
        writer: &'a mut csv::Writer<File>,
        run_id: usize,
        config_id: usize
    ) -> Self {
        let mut id = 0;

        let mut action_space = Vec::new();

        for i in 0..arms_per_group {
            let price =
                min_price + (max_price - min_price) * (i as f64 / (arms_per_group - 1) as f64);
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
                for arm_id in 0..arms_per_group {
                    let price = action_space[arm_id];
                    arms.insert(price as usize, Arm::new(id, price as i32));
                    id += 1;
                }
                
                let random_arm = action_space[rand::thread_rng().gen_range(0..arms_per_group)];
                best_group_arms.insert(period_id, random_arm as usize);
                best_group_rewards.insert(period_id, 0.0);
                group_arms.insert(period_id, arms);
            }

            arms.insert(group_id, group_arms);
            best_rewards.insert(group_id, best_group_rewards);
            best_arms.insert(group_id, best_group_arms);
            
        }
        Self {
            num_arms: (settings.num_predicted_groups as usize) * arms_per_group,
            arms: arms,
            best_arms,
            best_rewards,
            epsilon,
            arms_per_group,
            action_space,
            writer,
            last_action: "".to_string(),
            run_id: run_id,
            config_id: config_id
        }
    }

    pub fn random_action(&self) -> i32 {
        let random_offset = rand::thread_rng().gen_range(0..(self.arms_per_group - 1));
        // println!("random_offset: {}", self.action_space[random_offset]);
        return self.action_space[random_offset];
    }

    pub fn log(&self, writer: &mut csv::Writer<File>) {
        

        for grou_arms in self.arms.iter() {
            let group_id = grou_arms.0;
            for period_arms in grou_arms.1.iter() {
                let period_id = period_arms.0;
                let best_arm = self.best_arms.get(group_id).unwrap().get(period_id).unwrap();
                writer
                    .write_record(&[
                        self.config_id.to_string(),
                        period_id.to_string(),
                        group_id.to_string(),
                        best_arm.to_string(),
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
        let mut price = 0;

        if rand::thread_rng().gen::<f64>() < self.epsilon {
            price = self.random_action();
            self.last_action = "random".to_string();
        } else {
            price = self.best_arms[&group_id][&period] as i32;
            self.last_action = "best".to_string();
        }
        // self.writer
        //     .write_record(&[
        //         period.to_string(),
        //         group_id.to_string(),
        //         visit.to_string(),
        //         price.to_string(),
        //         "0".to_string(),
        //     ])
        //     .unwrap();

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
        if arm.average_reward > *self.best_rewards.get(&group_id).unwrap().get(&period).unwrap() {
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
