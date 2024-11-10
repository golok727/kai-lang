#[derive(Default, Debug)]
pub struct Engine {}

impl Engine {
    pub fn run(&mut self, work: bool) {
        if work {
            println!("bootstrap machine!")
        }
    }
}
