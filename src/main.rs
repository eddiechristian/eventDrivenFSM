mod v1; // no generics, one file - start here
mod v2; // generics: the FSM mechanism is reusable

fn main() {
    println!("--- v1: no generics ---");
    v1::run();

    println!();
    println!("--- v2: generics ---");
    v2::run();
}
