mod parser;

fn main() {
    let input = "apple, banana, cherry";
    println!("Count: {}", parser::parse_count(input));
    println!("Items: {:?}", parser::parse_items(input));
}
