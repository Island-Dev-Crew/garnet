fn main() {
    let mut caps = vec!["net", "fs", "net"];
    caps.sort();
    caps.dedup();
    println!("caps: {}", caps.join(","));
}
