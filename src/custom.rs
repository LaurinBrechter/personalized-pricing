use std::{collections::HashMap, fs, vec};
use rand::Rng;
use crate::{ evolution::PriceMatrix, logging::log_event_history, mab::Algorithm, simulation::{simulate_revenue, ProblemSettings} };

pub struct CustomSolution {
    pub price_matrix: PriceMatrix,
}

impl CustomSolution {
    pub fn new(start_prices: Vec<f64>, settings: &ProblemSettings) -> Self {
        let mut rng = rand::thread_rng();
        let mut prices = HashMap::new();

        for (g, start_price) in start_prices.iter().enumerate() {
            let mut group_map = HashMap::new();
            for w in 0..settings.n_visits {
                let mut period_prices = Vec::new();
                // TODO: change back
                for _ in 0..settings.n_periods {
                    period_prices.push(*start_price);
                }
                group_map.insert(w as usize, period_prices);
            }
            prices.insert(g as usize, group_map);
        }

        let mut ind = Self {
            price_matrix: PriceMatrix(prices),
        };
        ind
    }
}

impl<'a> Algorithm for CustomSolution {
    fn get_price(&mut self, group_id: usize, visit: usize, period: usize) -> i32 {
        // Since ES maintains a price matrix with visits and periods,
        // we'll use the first visit and period for now
        // TODO: Extend the Algorithm trait to handle multiple visits/periods
        self.price_matrix.get_price(group_id, visit, period) as i32
    }

    fn update_average_reward(
        &mut self,
        _group_id: usize,
        _visit: usize,
        _period: usize,
        _reward: f64,
        _price: i32
    ) {
        // ES doesn't update prices based on individual rewards
        // Instead, it uses the total fitness score for evolution
    }
}

pub fn simulate_custom(settings: &ProblemSettings, run_id: i32, writer: &mut csv::Writer<fs::File>) {

    

    let scenarios = vec![
        // vec![140.0, 350.0, 87.5],
        vec![200.0, 500.0, 125.0],
        // vec![260.0, 650.0, 162.5],
    ];
    let n_runs = 30;
    
    
    println!("Running {} simulations for each price vector...", n_runs);
    
    
    
    for (scenario_id, scenario) in scenarios.iter().enumerate() {
        let mut total_revenue = 0.0;
        let mut best_result = None;
        let mut best_revenue= 0.0;
        for i in 0..n_runs {
            let mut solution = CustomSolution::new(scenario.clone(), settings);
            
            let res = simulate_revenue(&mut solution, settings);
            
            

            total_revenue += res.revenue as f64;
            
            writer
                .write_record(&[
                    scenario_id.to_string(), 
                    (run_id as usize).to_string(), 
                    res.revenue.to_string()])
                .unwrap();

            // println!("Run {}: Vector A: {:.2}, Vector B: {:.2}", 
            //     i + 1, 
            //     res.n_sold, 
            //     res_B.n_sold
            // );
            
            // Track the best result (highest revenue)
            if res.revenue as f64 > best_revenue {
                best_revenue = res.revenue as f64;
                best_result = Some(("A", res.clone()));
            }
    }
        
        if let Some((vector_type, best)) = best_result.as_ref() {
            println!("\nBest result: Vector {} with revenue {:.2}", vector_type, best.revenue);
            log_event_history((scenario_id as i32 + run_id), best, &settings);
        
            // let file = std::fs::OpenOptions::new()
            //     .write(true)
            //     .create(true)
            //     .append(true)
            //     .open("./results/wom_network.csv")
            //     .unwrap();

            // let mut writer = csv::Writer::from_writer(file);
            // for cust in best.customers.iter() {
            //     writer
            //         .write_record(&[
            //             cust.id.to_string(),
            //             cust.group.to_string(),
            //             cust.wtp.to_string(),
            //         ])
            //         .unwrap();
            // }

        }

    println!("Vector average revenue: {:.2}", total_revenue / n_runs as f64);
    }
    

    // Log the best result from all runs
    
}