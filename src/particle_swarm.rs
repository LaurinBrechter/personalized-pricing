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
    pub fn_evals: i32,
}
use crate::mab::Algorithm;

#[derive(Clone, Debug)]
struct Particle<'a> {
    position: PriceMatrix, // Same structure as Individual's prices
    velocity: PriceMatrix,
    best_position: PriceMatrix,
    current_fitness: f64,
    best_fitness: f64,
    particle_id: i32,
    simulation_result: SimulationResult<'a>,
}

impl<'a> Particle<'a> {
    fn new(
        particle_id: i32,
        n_visits: usize,
        n_periods: usize,
        n_groups: usize,
        settings: &'a ProblemSettings,
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
                    period_prices.push(rng.gen_range(0.0..settings.max_price));
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
            simulation_result: SimulationResult {
            event_history: vec![],
            avg_regret: 0.0,
            regret: 0.0,
            customers: vec![],
            avg_time_sold_at: 0.0,
            n_sold: 0.0,
            revenue: 0.0,
        }
        };

        // Evaluate initial position
        let result = simulate_revenue(&mut particle, settings);
        particle.current_fitness = result.revenue;
        particle.best_fitness = result.revenue;
        particle.simulation_result = result;

        particle
    }

    fn to_individual(&self, result: SimulationResult<'a>) -> Individual<'a> {
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
                    // velocities[t] = velocities[t].clamp(-10.0, 10.0);
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

fn simulate_and_average<'a>(algorithm: &mut dyn Algorithm, settings: &'a ProblemSettings, n_evals:i32) -> (f64, SimulationResult<'a>) {
    let mut total_revenue = 0.0;
    let mut rng = rand::thread_rng();
    let n_runs = 10;
    let mut best_simulation_result: Option<SimulationResult<'a>> = None;

    for _ in 0..n_runs {
        let result = simulate_revenue(algorithm, settings);
        total_revenue += result.revenue;

        if best_simulation_result.is_none() || result.revenue > best_simulation_result.as_ref().unwrap().revenue {
            best_simulation_result = Some(result);
        }
    }

    return (total_revenue / n_runs as f64, best_simulation_result.unwrap());
}

impl<'a> Algorithm for Particle<'a> {
    fn get_price(&mut self, group_id: usize, visit: usize, period: usize) -> i32 {
        let converted_period = period % 10;
        self.position.get_price(group_id, visit, converted_period) as i32
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
    run_id: i32,
    writer: &mut csv::Writer<File>,
    particles: &Vec<Particle>,
    iteration: i32,
    settings: &PSOSettings,
) {
    for particle in particles {
        writer
            .write_record(&[
                run_id.to_string(),
                iteration.to_string(),
                particle.particle_id.to_string(),
                particle.current_fitness.to_string(),
                particle.best_fitness.to_string(),
                settings.swarm_size.to_string(),
                settings.inertia_weight.to_string(),
                settings.cognitive_coefficient.to_string(),
                settings.social_coefficient.to_string(),
                settings.fn_evals.to_string()
            ])
            .unwrap();
    }
}

pub fn optimize_pricing<'a>(
    run_id: i32,
    settings: &'a ProblemSettings,
    pso_settings: &PSOSettings,
    mut writer: &mut csv::Writer<File>,
) -> Individual<'a> {
    let mut particles: Vec<Particle> = Vec::new();
    let mut global_best_position = None;
    let mut global_best_fitness = f64::NEG_INFINITY;

    let mut num_evals = 0;

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
        
        num_evals += 1;
    }


    // Main PSO loop
    for iteration in 0..pso_settings.num_iterations {
        for particle in particles.iter_mut() {
            // Update velocity and position
            particle.update_velocity(global_best_position.as_ref().unwrap(), pso_settings);
            particle.update_position();

            // Evaluate new position
            let result = simulate_and_average(particle, settings, pso_settings.fn_evals);
            particle.current_fitness = result.0;
            particle.simulation_result = result.1;

            // Update particle's best if needed
            if particle.current_fitness > particle.best_fitness {
                println!("Particle {} improved its best fitness from {} to {}", particle.particle_id, particle.best_fitness, particle.current_fitness);
                particle.best_fitness = particle.current_fitness;
                particle.best_position = particle.position.clone();

                // Update global best if needed
                if particle.current_fitness > global_best_fitness {
                    global_best_fitness = particle.current_fitness;
                    global_best_position = Some(particle.position.clone());
                }
            }
        }
        num_evals += particles.len() as i32;

        log_iteration(run_id, &mut writer, &particles, num_evals, pso_settings);
        println!(
            "Iteration {}: Best revenue = {}",
            num_evals, global_best_fitness
        );
    }

    let best_particle = particles
        .iter_mut()
        .max_by(|a, b| a.current_fitness.partial_cmp(&b.current_fitness).unwrap())
        .unwrap();

    let final_result = simulate_and_average(best_particle, settings, pso_settings.fn_evals);
    best_particle.to_individual(final_result.1)
}
