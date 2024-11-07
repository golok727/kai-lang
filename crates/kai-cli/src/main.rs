use kai_core::Vm;

pub fn main() {
    println!("Welcome to kai");

    let mut vm = Vm::default();
    vm.run();
}
