use memuse::DynamicUsage;

fn main() {
    // let s = String::from("abcd");
    let s = "abcd";
    println!("Memory usage: {:?}", s.dynamic_usage());
}