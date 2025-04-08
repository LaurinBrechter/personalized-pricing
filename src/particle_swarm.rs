use crate::evolution::{Individual, PriceMatrix};
use crate::simulation::{simulate_revenue, ProblemSettings, SimulationEvent, SimulationResult};
use rand::Rng;
use std::{collections::HashMap, fs::File};
pub struct PSOSettings {
    pub num_iterations: i32,
    pub swarm_size: i32,
    pub inertia_weight: f64,
    pub cognitive_coefficient: f64, // c1
    pub social_coefficient: f64,    // c2
}
use crate::mab::Algorithm;

#[derive(Clone, Debug)]
struct Particle {
    position: PriceMatrix, // Same structure as Individual's prices
    velocity: PriceMatrix,
    best_position: PriceMatrix,
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
            position: PriceMatrix(position.clone()),
            velocity: PriceMatrix(velocity.clone()),
            best_position: PriceMatrix(position.clone()),
            current_fitness: 0.0,
            best_fitness: 0.0,
            particle_id,
            event_history: vec![],
        };

        // Evaluate initial position
        let result = simulate_revenue(&mut particle, settings);
        particle.current_fitness = result.revenue;
        particle.best_fitness = result.revenue;
        particle.event_history = result.event_history;

        particle
    }

    fn to_individual<'a>(&self, result: SimulationResult<'a>) -> Individual<'a> {
        Individual {
            prices: self.position.clone(),
            fitness_score: self.current_fitness,
            ind_id: self.particle_id,
            simulation_result: result,
        }
    }

    fn update_velocity(&mut self, global_best: &PriceMatrix, settings: &PSOSettings) {
        let mut rng = rand::thread_rng();

        for (g, group_map) in self.velocity.0.iter_mut() {
            for (w, velocities) in group_map.iter_mut() {
                for t in 0..velocities.len() {
                    let r1 = rng.gen::<f64>();
                    let r2 = rng.gen::<f64>();

                    velocities[t] = settings.inertia_weight * velocities[t]
                        + settings.cognitive_coefficient
                            * r1
                            * (self.best_position.0[g][w][t] - self.position.0[g][w][t])
                        + settings.social_coefficient
                            * r2
                            * (global_best.get_price(*g, *w, t)
                                - self.position.get_price(*g, *w, t));

                    // Limit velocity if needed
                    velocities[t] = velocities[t].clamp(-10.0, 10.0);
                }
            }
        }
    }

    fn update_position(&mut self) {
        for (g, group_map) in self.position.0.iter_mut() {
            for (w, prices) in group_map.iter_mut() {
                for t in 0..prices.len() {
                    prices[t] += self.velocity.get_price(*g, *w, t);
                    // Ensure prices stay within bounds
                    prices[t] = prices[t].max(0.0);
                }
            }
        }
    }
}

impl Algorithm for Particle {
    fn get_price(&mut self, group_id: usize, visit: usize, period: usize) -> i32 {
        self.position.get_price(group_id, visit, period) as i32
    }

    fn update_average_reward(
        &mut self,
        _group_id: usize,
        _visit: usize,
        _period: usize,
        reward: f64,
        _price: i32,
    ) {
        self.current_fitness = reward;
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

pub fn optimize_pricing<'a>(
    settings: &'a ProblemSettings,
    pso_settings: &PSOSettings,
    mut writer: &mut csv::Writer<File>,
) -> Individual<'a> {
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
            let result = simulate_revenue(particle, settings);
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

    let best_particle = particles
        .iter_mut()
        .max_by(|a, b| a.current_fitness.partial_cmp(&b.current_fitness).unwrap())
        .unwrap();

    let final_result = simulate_revenue(best_particle, settings);
    best_particle.to_individual(final_result)
}
