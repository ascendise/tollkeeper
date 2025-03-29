pub struct TollkeeperImpl {
    hosts: Vec<Host>,
}

impl Tollkeeper for TollkeeperImpl {
    fn access<TRequest>(req: Request, on_access: &dyn Fn(TRequest)) -> Option<Challenge> {
        Option::None
    }
}

pub trait Tollkeeper {
    fn access<TRequest>(req: Request, on_access: &dyn Fn(TRequest)) -> Option<Challenge>;
}

pub struct Host {
    base_url: String,
    on_filter: Operation,
    filters: Vec<Box<dyn Filter>>,
}

pub enum Operation {
    Allow,
    Challenge,
}

pub trait Filter {
    fn filter(&self, req: Request) -> bool;
}

pub struct Request {
    client_ip: String,
    user_agent: String,
    host: String,
    path: String,
}

pub struct Challenge {
    name: String,
}
