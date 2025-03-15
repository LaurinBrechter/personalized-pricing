use crate::evolution::Individual;
use ordered_float::OrderedFloat;
use priority_queue::PriorityQueue;
use rand::{rngs::ThreadRng, Rng};
use rand_distr::{Beta, Exp, Normal};
use std::cmp::Reverse;

#[derive(Debug, Clone)]
pub struct Customer<'a> {
    id: i32,
    group: i32,
    irp: f64,
    erp: f64,
    rp: f64,
    wtp: f64,
    max_wtp: f64,
    price_hist: Vec<f64>,
    // visit_hist: Vec<i32>,
    settings: &'a ProblemSettings,
}

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct SimulationEvent {
    pub t: OrderedFloat<f32>,
    pub event: String,
    pub customer: i32,
    pub customer_wtp: i32,
    pub customer_max_wtp: i32,
    pub group: i32,
    pub price: i32,
}

impl<'a> Customer<'a> {
    pub fn new(id: i32, group: i32, wtp: f64, max_wtp: f64, settings: &'a ProblemSettings) -> Self {
        Customer {
            id,
            group,
            irp: -1.0,
            erp: -1.0,
            rp: -1.0,
            wtp,
            max_wtp,
            price_hist: vec![],
            // visit_hist: vec![],
            settings,
        }
    }
    // LABEL
    pub fn update_irp(&mut self, new_price: f64) {
        self.irp = self.irp + self.settings.tau * (new_price - self.irp)
    }

    // TODO: implement aggregation via network.
    // LABEL
    pub fn update_erp(&mut self, other_customers: &Vec<Customer>) {
        let mut rng = rand::thread_rng();
        let mut ref_prices: Vec<f64> = vec![];
        for cust in other_customers {
            if rng.gen::<f64>() < 0.2 {
                if let Some(&price) = cust.price_hist.last() {
                    ref_prices.push(price);
                }
            }
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
    pub fn next_visit(&self, rng: &mut ThreadRng, t: f32) -> f32 {
        let distr = Exp::new(0.1).unwrap();
        return t + rng.sample::<f32, _>(distr);
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
}

pub fn init_simulation(
    customers: &Vec<Customer>,
) -> PriorityQueue<SimulationEvent, Reverse<OrderedFloat<f32>>> {
    let mut rng = rand::thread_rng();

    let mut event_calendar: PriorityQueue<SimulationEvent, Reverse<OrderedFloat<f32>>> =
        PriorityQueue::new();

    for customer in customers {
        let event = SimulationEvent {
            t: OrderedFloat(customer.next_visit(&mut rng, 0.0)),
            event: "customer_arrival".to_string(),
            customer: customer.id,
            customer_wtp: customer.wtp as i32,
            customer_max_wtp: customer.max_wtp as i32,
            group: customer.group,
            price: 0.0 as i32,
        };
        event_calendar.push(event.clone(), Reverse(event.t));
    }

    return event_calendar;
}

pub struct SimulationResult {
    pub regret: f64,
    pub n_sold: f64,
    pub avg_sold_at: f32,
    pub event_history: Vec<SimulationEvent>,
    pub revenue: f64,
}

// do one simulation run
pub fn simulate_revenue(individual: &Individual, settings: &ProblemSettings) -> SimulationResult {
    let mut rng = rand::thread_rng();

    let mut customers: Vec<Customer> = Vec::new();

    let beta_dist = Beta::new(2.0, 5.0).unwrap();

    let mut id = 0;
    let mut total_wtp = 0.0;
    for i in 0..settings.group_sizes.len() {
        let group_size = settings.group_sizes[i];
        let group_mean = settings.group_means[i];
        for _ in 0..group_size {
            let normal_dist = Normal::new(group_mean * settings.scaling, 5.0).unwrap();
            let wtp_increase = 1.0 + rng.sample(beta_dist);
            let wtp0: f64 = rng.sample(normal_dist);
            customers.push(Customer::new(
                id,
                i as i32,
                wtp0,
                wtp0 * wtp_increase,
                settings,
            ));
            total_wtp += wtp0;
            id += 1;
        }
    }

    let mut event_calendar = init_simulation(&customers);
    let mut revenue = 0.0;
    let mut regret = 0.0;
    let mut n_sold = 0;
    let mut event_count = 0;
    let mut avg_sold_at = 0.0;

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
        let next_visit = customers[customer_idx].next_visit(&mut rng, event.0.t.0);
        let visit_index = customers[customer_idx].price_hist.len() as i32;
        let price = individual.get_price(
            0 as usize, // visit_index as usize,
            customers[customer_idx].group as usize,
            0 as usize, // event.0.t.0 as usize,
        );
        if price > customers[customer_idx].max_wtp {
            event_history.push(SimulationEvent {
                t: OrderedFloat(event.0.t.0),
                event: "quit".to_string(),
                customer: event.0.customer,
                customer_wtp: event.0.customer_wtp,
                customer_max_wtp: event.0.customer_max_wtp,
                group: event.0.group,
                price: price as i32,
            });
            continue;
        }
        if price < customers[customer_idx].wtp {
            revenue += price;
            customers[customer_idx].price_hist.push(price);
            regret += customers[customer_idx].wtp - price;
            n_sold += 1;
            avg_sold_at += event.0.t.0;

            event_history.push(SimulationEvent {
                t: OrderedFloat(event.0.t.0),
                event: "sold".to_string(),
                customer: event.0.customer,
                customer_wtp: event.0.customer_wtp,
                customer_max_wtp: event.0.customer_max_wtp,
                group: event.0.group,
                price: price as i32,
            });
        } else {
            let next_event = SimulationEvent {
                t: OrderedFloat(next_visit),
                event: "customer_arrival".to_string(),
                customer: event.0.customer,
                customer_wtp: event.0.customer_wtp,
                customer_max_wtp: event.0.customer_max_wtp,
                group: event.0.group,
                price: price as i32,
            };
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
        n_sold: n_sold as f64 / customers.len() as f64,
        avg_sold_at: avg_sold_at / n_sold as f32,
        event_history,
        revenue,
    };
}
