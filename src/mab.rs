use std::{collections::HashMap, fs::File};

use rand::Rng;
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
    arms: HashMap<usize, HashMap<usize, Arm>>,
    best_arm: HashMap<usize, usize>,
    pub best_reward: HashMap<usize, f64>,
    epsilon: f64,
    arms_per_group: usize,
    action_space: Vec<i32>,
    writer: &'a mut csv::Writer<File>,
    last_action: String,
}

impl<'a> MAB<'a> {
    pub fn new(
        num_groups: usize,
        min_price: f64,
        max_price: f64,
        arms_per_group: usize,
        epsilon: f64,
        writer: &'a mut csv::Writer<File>,
    ) -> Self {
        let mut group_arms = HashMap::new();
        let mut id = 0;

        let mut action_space = Vec::new();

        for i in 0..arms_per_group {
            let price =
                min_price + (max_price - min_price) * (i as f64 / (arms_per_group - 1) as f64);
            action_space.push(price as i32);
        }
        println!("action_space: {:?}", action_space);
        let mut best_arm = HashMap::new();
        let mut best_reward = HashMap::new();

        for group_id in 0..num_groups {
            best_arm.insert(group_id, 0);
            best_reward.insert(group_id, 0.0);
            let mut arms = HashMap::new();
            for arm_id in 0..arms_per_group {
                let price = action_space[arm_id];
                arms.insert(price as usize, Arm::new(id, price as i32));
                id += 1;
            }
            group_arms.insert(group_id, arms);
        }
        Self {
            num_arms: num_groups * arms_per_group,
            arms: group_arms,
            best_arm,
            best_reward,
            epsilon,
            arms_per_group,
            action_space,
            writer,
            last_action: "".to_string(),
        }
    }

    pub fn random_action(&self) -> i32 {
        let random_offset = rand::thread_rng().gen_range(0..(self.arms_per_group - 1));
        // println!("random_offset: {}", self.action_space[random_offset]);
        return self.action_space[random_offset];
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
            price = self.best_arm[&group_id] as i32;
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
        let arms = self.arms.get_mut(&group_id).unwrap();
        let arm = arms.get_mut(&(arm_id as usize)).unwrap();

        self.writer
            .write_record(&[
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
        if arm.average_reward > *self.best_reward.get(&group_id).unwrap_or(&0.0) {
            self.best_reward.insert(group_id, arm.average_reward);
            self.best_arm.insert(group_id, arm_id as usize);
        }
    }
}
