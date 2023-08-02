fn main() {
    let feature1 = cfg!(feature = "doublestack");
    let feature2 = cfg!(feature = "vecdeque");

    if feature1 && feature2 {
        println!("error: Features `doublestack` and `vecdeque` are mutually exclusive and cannot be enabled at the same time.");
        std::process::exit(1);
    }
}
