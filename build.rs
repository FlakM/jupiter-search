fn main() {
    std::process::Command::new("git").arg("submodule").arg("update").arg("--init").arg("--recursive").output().expect("Failed to clone submodules");
}
