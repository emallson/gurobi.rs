extern crate glob;
use glob::glob;

fn main() {
    let env_path = option_env!("GUROBI_HOME");

    if let Some(gurobi_home) = env_path {
        println!("cargo:rustc-link-search=native={}/lib", gurobi_home);
    } else {
        for path in glob("/opt/gurobi*/*/lib").unwrap() {
            match path {
                Ok(libdir) => println!("cargo:rustc-link-search=native={}", libdir.display()),
                Err(e) => println!("{:?}", e),
            }
        }
    }

    println!("cargo:rustc-link-lib=gurobi81");
}
