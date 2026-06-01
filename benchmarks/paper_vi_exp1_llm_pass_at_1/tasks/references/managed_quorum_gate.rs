fn main() {
    let approvals = ["APPROVED", "PENDING", "APPROVED", "REJECTED", "APPROVED", "APPROVED"];
    let count = approvals.iter().filter(|state| **state == "APPROVED").count();
    println!("approved: {count}");
}
