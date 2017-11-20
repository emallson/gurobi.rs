# gurobi.rs

Rust bindings for
[Gurobi](https://gurobi.com),
an IP/MIP/QP solver. Currently highly experimental but
working for IP/MIP problems.

```toml
[dependencies]
gurobi = {version = "0.1", git = "https://github.com/emallson/gurobi.rs.git"} 
```

You *must* have Gurobi installed with a valid license on your path for
Gurobi.rs to work. The build script
attempts to locate Gurobi automatically (and should do so assuming
you've installed it at the usual location on Linux). Gurobi cannot be
statically linked to via the C API (as far as I know) so it must also be
available on each machine that you intend to run a program using `gurobi.rs`
on, including the license.

[Documentation](http://atlanis.net/doc/rs/rplex/)

# Example

```rust
fn mip1() {
    let mut env = Env::new();
    env.set_threads(6).unwrap();
    let mut model = Model::new(&env).unwrap();
    let x = model.add_var(1.0, VariableType::Binary).unwrap();
    let y = model.add_var(1.0, VariableType::Binary).unwrap();
    let z = model.add_var(1.0, VariableType::Binary).unwrap();

    model.add_con(Constraint::build().plus(x, 1.0).plus(y, 2.0).plus(z, 3.0).is_less_than(4.0)).unwrap();
    model.add_con(Constraint::build().sum(&[x, y]).is_greater_than(1.0)).unwrap();

    model.set_objective_type(ObjectiveType::Maximize);

    let sol = model.optimize().unwrap();
    assert_eq!(sol.value().unwrap(), 2.0);
    let vars = sol.variables(x, z).unwrap();
    assert_eq!(vars, vec![1.0, 1.0, 0.0]);
}
```

# License

Gurobi and associated items are (c) Gurobi.

Copyright (c) 2017, J. David Smith. All rights reserved.

Redistribution and use in source and binary forms, with or without modification, are permitted provided that the following conditions are met:

Redistributions of source code must retain the above copyright notice, this list of conditions and the following disclaimer.

Redistributions in binary form must reproduce the above copyright notice, this list of conditions and the following disclaimer in the documentation and/or other materials provided with the distribution.

Neither the name of the copyright holder nor the names of its contributors may be used to endorse or promote products derived from this software without specific prior written permission.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
