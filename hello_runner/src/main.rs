extern {
    fn add(left: usize, right: usize) -> usize;
}

fn main() {
    println!("2+2={}", add(2,2));
}
