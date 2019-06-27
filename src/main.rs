mod arguments;

fn main() {
    let (command, args) = arguments::parse();
    println!("{}, {:?}", command, args);
}
