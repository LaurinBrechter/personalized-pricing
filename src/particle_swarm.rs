use crate::evolution::Individual;
use crate::simulation::{simulate_revenue, ProblemSettings, SimulationEvent};
use rand::Rng;
use std::{collections::HashMap, fs::File};
pub struct PSOSettings {
    pub num_iterations: i32,
    pub swarm_size: i32,
    pub inertia_weight: f64,
    pub cognitive_coefficient: f64, // c1
    pub social_coefficient: f64,    // c2
}

#[derive(Clone, Debug)]
struct Particle {
    position: HashMap<usize, HashMap<usize, Vec<f64>>>, // Same structure as Individual's prices
    velocity: HashMap<usize, HashMap<usize, Vec<f64>>>,
    best_position: HashMap<usize, HashMap<usize, Vec<f64>>>,
    current_fitness: f64,
    best_fitness: f64,
    particle_id: i32,
    event_history: Vec<SimulationEvent>,
}

impl Particle {
    fn new(
        particle_id: i32,
        n_visits: usize,
        n_periods: usize,
        n_groups: usize,
        settings: &ProblemSettings,
    ) -> Self {
        let mut rng = rand::thread_rng();
        let mut position = HashMap::new();
        let mut velocity = HashMap::new();

        // Initialize position and velocity
        for g in 0..n_groups {
            let mut group_map_pos = HashMap::new();
            let mut group_map_vel = HashMap::new();
            for w in 0..n_visits {
                let mut period_prices = Vec::new();
                let mut period_velocities = Vec::new();
                for _ in 0..n_periods {
                    period_prices.push(rng.gen_range(0.0..100.0));
                    period_velocities.push(rng.gen_range(-5.0..5.0));
                }
                group_map_pos.insert(w, period_prices);
                group_map_vel.insert(w, period_velocities);
            }
            position.insert(g, group_map_pos);
            velocity.insert(g, group_map_vel);
        }

        let mut particle = Self {
            position: position.clone(),
            velocity,
            best_position: position,
            current_fitness: 0.0,
            best_fitness: 0.0,
            particle_id,
            event_history: vec![],
        };

        // Evaluate initial position
        let result = simulate_revenue(&particle.to_individual(), settings);
        particle.current_fitness = result.revenue;
        particle.best_fitness = result.revenue;
        particle.event_history = result.event_history;

        particle
    }

    fn to_individual(&self) -> Individual {
        Individual {
            prices: self.position.clone(),
            event_history: self.event_history.clone(),
            fitness_score: self.current_fitness,
            ind_id: self.particle_id,
        }
    }

    fn update_velocity(
        &mut self,
        global_best: &HashMap<usize, HashMap<usize, Vec<f64>>>,
        settings: &PSOSettings,
    ) {
        let mut rng = rand::thread_rng();

        for (g, group_map) in self.velocity.iter_mut() {
            for (w, velocities) in group_map.iter_mut() {
                for t in 0..velocities.len() {
                    let r1 = rng.gen::<f64>();
                    let r2 = rng.gen::<f64>();

                    velocities[t] = settings.inertia_weight * velocities[t]
                        + settings.cognitive_coefficient
                            * r1
                            * (self.best_position[g][w][t] - self.position[g][w][t])
                        + settings.social_coefficient
                            * r2
                            * (global_best[g][w][t] - self.position[g][w][t]);

                    // Limit velocity if needed
                    velocities[t] = velocities[t].clamp(-10.0, 10.0);
                }
            }
        }
    }

    fn update_position(&mut self) {
        for (g, group_map) in self.position.iter_mut() {
            for (w, prices) in group_map.iter_mut() {
                for t in 0..prices.len() {
                    prices[t] += self.velocity[g][w][t];
                    // Ensure prices stay within bounds
                    prices[t] = prices[t].max(0.0);
                }
            }
        }
    }
}

fn log_iteration(
    writer: &mut csv::Writer<File>,
    particles: &Vec<Particle>,
    iteration: i32,
    settings: &PSOSettings,
) {
    for particle in particles {
        writer
            .write_record(&[
                iteration.to_string(),
                particle.particle_id.to_string(),
                particle.current_fitness.to_string(),
                particle.best_fitness.to_string(),
                settings.swarm_size.to_string(),
                settings.inertia_weight.to_string(),
                settings.cognitive_coefficient.to_string(),
                settings.social_coefficient.to_string(),
            ])
            .unwrap();
    }
}

pub fn optimize_pricing(
    settings: &ProblemSettings,
    pso_settings: &PSOSettings,
    mut writer: &mut csv::Writer<File>,
) -> Individual {
    let mut particles: Vec<Particle> = Vec::new();
    let mut global_best_position = None;
    let mut global_best_fitness = f64::NEG_INFINITY;

    // Initialize swarm
    for i in 0..pso_settings.swarm_size {
        particles.push(Particle::new(
            i,
            settings.n_visits as usize,
            settings.n_periods as usize,
            settings.n_groups as usize,
            settings,
        ));

        // Update global best if needed
        if particles.last().unwrap().current_fitness > global_best_fitness {
            global_best_fitness = particles.last().unwrap().current_fitness;
            global_best_position = Some(particles.last().unwrap().position.clone());
        }
    }

    // Main PSO loop
    for iteration in 0..pso_settings.num_iterations {
        for particle in particles.iter_mut() {
            // Update velocity and position
            particle.update_velocity(global_best_position.as_ref().unwrap(), pso_settings);
            particle.update_position();

            // Evaluate new position
            let result = simulate_revenue(&particle.to_individual(), settings);
            particle.current_fitness = result.revenue;
            particle.event_history = result.event_history;

            // Update particle's best if needed
            if particle.current_fitness > particle.best_fitness {
                particle.best_fitness = particle.current_fitness;
                particle.best_position = particle.position.clone();

                // Update global best if needed
                if particle.current_fitness > global_best_fitness {
                    global_best_fitness = particle.current_fitness;
                    global_best_position = Some(particle.position.clone());
                }
            }
        }

        log_iteration(&mut writer, &particles, iteration, pso_settings);
        println!(
            "Iteration {}: Best revenue = {}",
            iteration, global_best_fitness
        );
    }

    // Convert best solution to Individual and return
    Individual {
        prices: global_best_position.unwrap(),
        event_history: vec![],
        fitness_score: global_best_fitness,
        ind_id: -1,
    }
}
