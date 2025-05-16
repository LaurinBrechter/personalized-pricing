use crate::{custom, evolution::ESSettings, network_formation::create_network};
use ordered_float::OrderedFloat;
use priority_queue::PriorityQueue;
use rand::{rngs::ThreadRng, Rng};
use rand_distr::{Beta, Exp, Normal};
use std::cmp::Reverse;

use crate::mab::Algorithm;
#[derive(Debug, Clone)]
pub struct Customer<'a> {
    id: i32,              // unique identifier for the customer
    group: i32,           // true underlying group to which the customer belongs
    predicted_group: i32, // group to which the customer is predicted to belong based on clustering
    irp: f64,             // internal reference price
    erp: f64,             // external reference price
    rp: f64,              // reference price
    wtp: f64,             // willingness to pay
    max_wtp: f64,         // maximum willingness to pay
    price_hist: Vec<f64>, // history of prices
    settings: &'a ProblemSettings,
    neighbors: Vec<i32>, // list of the ids of neighboring customers
    initial_wtp: f64,
}

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct SimulationEvent {
    pub t: OrderedFloat<f32>,
    pub event: String,
    pub customer: i32,
    pub customer_wtp: i32,
    pub customer_max_wtp: i32,
    pub predicted_group: i32,
    pub actual_group: i32,
    pub price: i32,
    pub irp: i32,
    pub erp: i32,
    pub rp: i32,
    pub adjusted_wtp: i32,
}

impl SimulationEvent {
    pub fn new(customer: &Customer, t: OrderedFloat<f32>, event: String, price: f64, adjusted_wtp:f64) -> Self {
        SimulationEvent {
            t,
            event,
            customer: customer.id,
            customer_wtp: customer.wtp as i32,
            customer_max_wtp: customer.max_wtp as i32,
            irp: customer.irp as i32,
            erp: customer.erp as i32,
            rp: customer.rp as i32,
            actual_group: customer.group,
            predicted_group: customer.predicted_group,
            price: price as i32,
            adjusted_wtp: adjusted_wtp as i32,
        }
    }
}

impl<'a> Customer<'a> {
    pub fn new(
        id: i32,
        group: i32,
        predicted_group: i32,
        wtp: f64,
        max_wtp: f64,
        settings: &'a ProblemSettings,
        neighbors: Vec<i32>,
    ) -> Self {
        Customer {
            id,
            group,
            predicted_group,
            irp: wtp,
            erp: wtp,
            rp: wtp,
            wtp,
            max_wtp,
            price_hist: vec![],
            // visit_hist: vec![],
            settings,
            neighbors,
            initial_wtp: wtp,
        }
    }
    // LABEL
    pub fn update_irp(&mut self, new_price: f64) {
        self.irp = self.settings.tau * new_price + self.irp * (1.0 - self.settings.tau)
    }

    // TODO: implement aggregation via network.
    // LABEL
    pub fn update_erp(&mut self, other_customers: &Vec<Customer>) {
        let mut ref_prices: Vec<f64> = vec![];
        for &neighbor_id in &self.neighbors {
            if let Some(&price) = other_customers[neighbor_id as usize].price_hist.last() {
                ref_prices.push(price);
            }
        }
        for customer in other_customers {
            if customer.id == self.id {
                continue;
            } else {
                if rand::thread_rng().gen::<f64>() < self.settings.global_wom_prob {
                    ref_prices.push(customer.wtp);
                }
            }
        }

        // Calculate average of reference prices and update erp
        if !ref_prices.is_empty() {
            let avg_price = ref_prices.iter().sum::<f64>() / ref_prices.len() as f64;
            self.erp = avg_price;
        } else if self.erp < 0.0 {
            // Initialize erp if it hasn't been set yet
            self.erp = self.wtp;
        }
    }
    // LABEL
    pub fn update_rp(&mut self) {
        if self.erp > self.irp {
            self.rp = self.irp;
        } else {
            self.rp = self.settings.eta * self.erp + (1.0 - self.settings.eta) * self.irp;
        }
    }

    // LABEL
    pub fn update_wtp(&mut self) {
        if self.rp > self.wtp {
            self.wtp += (self.rp - self.wtp).powf(self.settings.alpha);
        } else {
            self.wtp -= self.settings.lambda * (self.wtp - self.rp).powf(self.settings.alpha);
        }
    }

    // LABEL
    // TODO: fix
    pub fn next_visit(&self, rng: &mut ThreadRng, t: f32, price: f64) -> f32 {
        // Calculate the relative price difference
        let price_diff = (self.wtp - price) / self.wtp;

        // Scale the rate parameter: smaller difference → larger rate → shorter intervals
        // larger difference → smaller rate → longer intervals
        let base_rate = 0.1;
        // Ensure rate stays positive by using max() and adding a small constant
        let rate = (base_rate * (1.0 / (1.0 + price_diff))).max(0.001);

        let distr = Exp::new(rate as f32).unwrap();
        t + rng.sample::<f32, _>(distr)
    }
}

#[derive(Debug)]
pub struct ProblemSettings {
    pub n_visits: i32,         // number of visits
    pub n_periods: i32,        // number of periods
    pub n_groups: i32,         // number of groups
    pub n_customers: i32,      // total number of customers
    pub tau: f64,              // how much customers are influenced by the current price
    pub scaling: f64,          // scaling factor for the normal distribution
    pub group_sizes: Vec<i32>, // number of customers in each group
    pub group_means: Vec<f64>, // mean wtp for each group
    pub alpha: f64,
    pub lambda: f64, // loss aversion
    pub eta: f64,    // price sensitivity
    pub max_events: i32,
    pub clustering_accuracy: f64,
    pub k_neighbors: i32,
    pub p_intra: f64,
    pub p_inter: f64,
    pub global_wom_prob: f64,
    pub max_price: f64,
    pub num_predicted_groups: i32,
    pub sigmoid_scale: f64,
}

pub fn init_simulation(
    customers: &Vec<Customer>,
) -> PriorityQueue<SimulationEvent, Reverse<OrderedFloat<f32>>> {
    let mut rng = rand::thread_rng();

    let mut event_calendar: PriorityQueue<SimulationEvent, Reverse<OrderedFloat<f32>>> =
        PriorityQueue::new();

    for customer in customers {
        let event = SimulationEvent::new(
            customer,
            OrderedFloat(customer.next_visit(&mut rng, 0.0, 0.0)),
            "customer_arrival".to_string(),
            0.0,
            customer.wtp,
        );
        event_calendar.push(event.clone(), Reverse(event.t));
    }

    return event_calendar;
}

#[derive(Debug, Clone)]
pub struct SimulationResult<'a> {
    pub regret: f64,
    pub n_sold: f64,
    pub avg_time_sold_at: f32,
    pub event_history: Vec<SimulationEvent>,
    pub revenue: f64,
    pub avg_regret: f64,
    pub customers: Vec<Customer<'a>>,
}

pub fn simulate_revenue<'a>(
    algorithm: &mut dyn Algorithm,
    settings: &'a ProblemSettings,
) -> SimulationResult<'a> {
    let mut rng = rand::thread_rng();

    let mut customers: Vec<Customer> = Vec::new();

    let beta_dist = Beta::new(3.0, 6.0).unwrap();

    let network = create_network(settings);

    let mut id = 0;
    for customer_group in 0..settings.group_sizes.len() {
        let group_size = settings.group_sizes[customer_group];
        let group_mean = settings.group_means[customer_group];
        for _ in 0..group_size {
            let neighbors = network[id as usize].clone();
            let normal_dist = Normal::new(
                group_mean * settings.scaling,
                (group_mean * settings.scaling * 0.2).powf(0.5),
            )
            .unwrap();
            let wtp_increase = 2.0; //+ rng.sample(beta_dist);
            let wtp0: f64 = rng.sample(normal_dist);

            // With X% probability, assign a random group prediction
            let predicted_group: usize = if rng.gen_bool(1.0-settings.clustering_accuracy) {
                rng.gen_range(0..(settings.num_predicted_groups as usize))
            } else {
                if (customer_group as usize) < settings.num_predicted_groups as usize {
                    customer_group as usize
                } else {
                    rng.gen_range(0..(settings.num_predicted_groups as usize))
                }
            };

            customers.push(Customer::new(
                id,
                customer_group as i32,
                predicted_group as i32,
                wtp0,
                wtp0 * wtp_increase,
                settings,
                neighbors,
            ));
            // total_wtp += wtp0;
            id += 1;
        }
    }

    let mut event_calendar = init_simulation(&customers);
    let mut revenue = 0.0;
    let mut regret = 0.0;
    let mut n_sold = 0;
    let mut event_count = 0;
    let mut avg_sold_at = 0.0;
    let mut avg_wtp_increase = 0.0;

    let mut event_history: Vec<SimulationEvent> = Vec::new();

    while !event_calendar.is_empty() && event_count < settings.max_events {
        event_count += 1;
        let event = event_calendar.pop();
        if event.is_none() {
            break;
        }
        let event = event.unwrap();
        if event.0.t > OrderedFloat(settings.n_periods as f32) {
            continue;
        }

        event_history.push(event.0.clone());

        let customer_idx = event.0.customer as usize;

        let visit_index = customers[customer_idx].price_hist.len() as i32;
        let price = algorithm.get_price(
            customers[customer_idx].predicted_group as usize,
            visit_index as usize,
            // 0,
            // 0
            event.0.t.0 as usize,
            // 0
        ) as f64;
        
        let period = 10.0;
        let amplitude = 0.0;

        let time_factor = amplitude * ((2.0 * std::f64::consts::PI * event.0.t.0 as f64 / period).sin());

        // println!("Time factor: {}", time_factor);
        let adjusted_wtp = customers[customer_idx].wtp * (1.0 + time_factor as f64);

        let price_diff_pct = (adjusted_wtp - price) / adjusted_wtp;
        let purchase_prob = 1.0 / (1.0 + (-price_diff_pct * settings.sigmoid_scale).exp());



        if price > adjusted_wtp * 1.2 {
            regret += adjusted_wtp;
            event_history.push(SimulationEvent::new(
                &customers[customer_idx],
                event.0.t,
                "quit".to_string(),
                price,
                adjusted_wtp
            ));
            algorithm.update_average_reward(
                customers[customer_idx].predicted_group as usize,
                visit_index as usize,
                event.0.t.0 as usize,
                0.0,
                price as i32,
            );
            continue;
        } else if rng.gen::<f64>() < purchase_prob {
            revenue += price;
            customers[customer_idx].price_hist.push(price);
            regret += adjusted_wtp - price;
            n_sold += 1;
            avg_sold_at += event.0.t.0;
            avg_wtp_increase += adjusted_wtp - customers[customer_idx].initial_wtp;

            // Update the algorithm with the reward (revenue in this case)
            algorithm.update_average_reward(
                customers[customer_idx].predicted_group as usize,
                visit_index as usize,
                event.0.t.0 as usize,
                price,
                price as i32,
            );

            event_history.push(SimulationEvent::new(
                &customers[customer_idx],
                event.0.t,
                "sold".to_string(),
                price,
                adjusted_wtp
            ));
        } else {
            let next_visit =
                customers[customer_idx].next_visit(&mut rng, event.0.t.0, event.0.price as f64);
            let next_event = SimulationEvent::new(
                &customers[customer_idx],
                OrderedFloat(next_visit),
                "customer_arrival".to_string(),
                price,
                adjusted_wtp
            );
            // algorithm.update_average_reward(
            //     customers[customer_idx].predicted_group as usize,
            //     visit_index as usize,
            //     event.0.t.0 as usize,
            //     0.0,
            //     price as i32,
            // );
            event_calendar.push(next_event, Reverse(OrderedFloat(next_visit)));
        }
        let customers_copy = customers.to_vec();
        customers[customer_idx].update_irp(price);
        customers[customer_idx].update_erp(&customers_copy);
        customers[customer_idx].update_rp();
        customers[customer_idx].update_wtp();
    }

    return SimulationResult {
        regret,
        avg_regret: regret / customers.len() as f64,
        n_sold: n_sold as f64 / customers.len() as f64,
        avg_time_sold_at: avg_sold_at / n_sold as f32,
        event_history,
        revenue,
        customers,
    };
}
