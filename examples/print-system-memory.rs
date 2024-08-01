
use system_memory::{available, total};

fn main() {
    println!("Currently available: {} bytes\nTotal: {} bytes", available(), total());
}
