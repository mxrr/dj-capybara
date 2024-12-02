fn main() {
  let timestamp = chrono::Local::now().to_rfc3339();
  println!("cargo:rustc-env=BUILD_TIMESTAMP={timestamp}");

  let git_commit = std::env::var("GIT_COMMIT").unwrap_or_else(|_| {
    let git_output = std::process::Command::new("git")
      .args(["rev-parse", "--short", "HEAD"])
      .output()
      .unwrap()
      .stdout;

    if git_output.is_empty() {
      panic!("failed getting latest commit hash from git");
    }

    String::from_utf8(git_output).expect("invalid utf8 data received from git")
  });
  println!("cargo:rustc-env=GIT_COMMIT={git_commit}");

  let rustc_version_output = String::from_utf8(
    std::process::Command::new("rustc")
      .arg("-vV")
      .output()
      .unwrap()
      .stdout,
  )
  .expect("invalid utf8 data received from rustc");

  rustc_version_output.split('\n').for_each(|line| {
    if line.starts_with("host: ") {
      println!("cargo:rustc-env=RUSTC_HOST_TRIPLE={}", &line[6..]);
    } else if line.starts_with("release: ") {
      println!("cargo:rustc-env=RUSTC_SEMVER={}", &line[9..]);
    } else if line.starts_with("LLVM version: ") {
      println!("cargo:rustc-env=RUSTC_LLVM_VERSION={}", &line[14..]);
    }
  });
}
