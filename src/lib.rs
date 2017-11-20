extern crate libc;

use libc::{c_int, c_char, c_double};
use std::ffi::CStr;
use std::ptr;
use std::borrow::Borrow;


enum GurobiEnv {}
enum GurobiModel {}

type ErrorCode = c_int;

#[allow(non_snake_case)]
extern "C" {
    // instantiation
    fn GRBloadenv(env: *mut *mut GurobiEnv, log: *const c_char) -> c_int;
    fn GRBnewmodel(env: *mut GurobiEnv,
                   model: *mut *mut GurobiModel,
                   name: *const c_char,
                   numvars: c_int,
                   obj: *const c_double,
                   lb: *const c_double,
                   ub: *const c_double,
                   vtype: *const c_char,
                   varnames: *const *const c_char)
                   -> ErrorCode;
    // error handling
    fn GRBgeterrormsg(env: *mut GurobiEnv) -> *const c_char;

    // update -- has no cplex equivalent, causes additional variables to be added and constraints
    // to be applied
    fn GRBupdatemodel(model: *mut GurobiModel) -> ErrorCode;

    // optimize
    fn GRBoptimize(model: *mut GurobiModel) -> ErrorCode;

    // add a variable
    fn GRBaddvar(model: *mut GurobiModel, numnz: c_int, vind: *const c_int, vval: *const c_double, obj: c_double, lb: c_double, ub: c_double, vtype: c_char, varname: *const c_char) -> ErrorCode;

    // add a constraint
    fn GRBaddconstr(model: *mut GurobiModel, numnz: c_int, cind: *const c_int, cval: *const c_double, sense: c_char, rhs: c_double, constrname: *const c_char) -> ErrorCode;

    // attribute manipulation
    fn GRBsetintattr(model: *mut GurobiModel, attr_id: *const c_char, value: c_int) -> ErrorCode;
    fn GRBgetintattr(model: *mut GurobiModel, attr_id: *const c_char, value: *mut c_int) -> ErrorCode;
    fn GRBsetdblattr(model: *mut GurobiModel, attr_id: *const c_char, value: c_double) -> ErrorCode;
    fn GRBgetdblattr(model: *mut GurobiModel, attr_id: *const c_char, value: *mut c_double) -> ErrorCode;

    // freeing
    fn GRBfreemodel(model: *mut GurobiModel);
    fn GRBfreeenv(env: *mut GurobiEnv);
}

fn code_to_result<'a>(code: c_int, env: *mut GurobiEnv) -> Result<(), &'a str> {
    if code == 0 {
        Ok(())
    } else {
        Err(unsafe { CStr::from_ptr(GRBgeterrormsg(env)).to_str().unwrap() })
    }
}

fn name(s: &str) -> *const c_char {
    s.as_ptr() as *const c_char
}

/// A Gurobi Environment. Create using `::new()`. Automatically freed on drop.
pub struct Env {
    inner: *mut GurobiEnv,
}

impl Env {
    pub fn new() -> Self {
        let mut env = ptr::null_mut();
        unsafe {
            GRBloadenv(&mut env, ptr::null());
        }
        Env { inner: env }
    }
}

impl Drop for Env {
    fn drop(&mut self) {
        unsafe { GRBfreeenv(self.inner) }
    }
}

pub struct Model<'a> {
    env: &'a Env,
    inner: *mut GurobiModel,
    num_vars: usize,
    num_constraints: usize,
}

impl<'a> Model<'a> {
    /// Creates a new, empty model within the given environment.
    pub fn new(env: &'a Env) -> Result<Self, &str> {
        let mut model = ptr::null_mut();
        let res = unsafe {
            code_to_result(GRBnewmodel(env.inner,
                                       &mut model,
                                       ptr::null(),
                                       0,
                                       ptr::null(),
                                       ptr::null(),
                                       ptr::null(),
                                       ptr::null(),
                                       ptr::null()),
                           env.inner)
        };

        res.map(|_| Model {
            env, inner: model, num_vars: 0, num_constraints: 0,
        })
    }

    pub fn add_var(&mut self, obj: f64, kind: VariableType) -> Result<VarIndex, &str> {
        unsafe {
            code_to_result(GRBaddvar(self.inner, 
                                     0, ptr::null(), ptr::null(), 
                                     obj, kind.lb(), kind.ub(), kind.vtype(), 
                                     ptr::null()), 
                           self.env.inner)
        }.map(|_| {
            self.num_vars += 1;
            VarIndex(self.num_vars - 1)
        })
    }

    pub fn add_con(&mut self, con: Constraint) -> Result<ConIndex, &str> {
        unsafe {
            code_to_result(
                GRBaddconstr(self.inner, con.numnz(), con.indices.as_ptr(), con.weights.as_ptr(), con.sense.sense(), con.rhs, ptr::null()),
                self.env.inner
            )
        }.map(|_| {
            self.num_constraints += 1;
            ConIndex(self.num_constraints - 1)
        })
    }

    pub fn set_objective_type(&mut self, obj: ObjectiveType) -> Result<(), &str> {
        unsafe {
            code_to_result(GRBsetintattr(self.inner, name("ModelSense"), obj.sense()), self.env.inner)
        }
    }

    pub fn update(&mut self) -> Result<(), &str> {
        unsafe {
            code_to_result(
                GRBupdatemodel(self.inner),
                self.env.inner
            )
        }
    }

    pub fn optimize(&mut self) -> Result<Solution, &str> {
        unsafe {
            code_to_result(
                GRBoptimize(self.inner),
                self.env.inner
            )
        }.map(move |_| {
            Solution {
                model: self
            }
        })
    }
}

impl<'a> Drop for Model<'a> {
    fn drop(&mut self) {
        unsafe { GRBfreemodel(self.inner) }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct VarIndex(usize);

impl VarIndex {
    fn id(&self) -> c_int {
        self.0 as c_int
    }
}
#[derive(Copy, Clone, Debug)]
pub struct ConIndex(usize);

pub enum VariableType {
    Binary,
    Continuous(f64, f64),
    Integer(f64, f64),
    SemiContinuous(f64, f64),
    SemiInteger(f64, f64),
}

impl VariableType {
    fn ub(&self) -> f64 {
        use VariableType::*;
        match self {
            &Binary                => 1.0,
            &Continuous(_, ub)     => ub,
            &Integer(_, ub)        => ub,
            &SemiContinuous(_, ub) => ub,
            &SemiInteger(_, ub)    => ub,
        }
    }

    fn lb(&self) -> f64 {
        use VariableType::*;
        match self {
            &Binary                => 0.0,
            &Continuous(lb, _)     => lb,
            &Integer(lb, _)        => lb,
            &SemiContinuous(lb, _) => lb,
            &SemiInteger(lb, _)    => lb,
        }
    }

    fn vtype(&self) -> c_char {
        use VariableType::*;
        match self {
            &Binary => 'B' as i8,
            &Continuous(_, _) => 'C' as i8,
            &Integer(_, _) => 'I' as i8,
            &SemiContinuous(_, _) => 'S' as i8,
            &SemiInteger(_, _) => 'N' as i8,
        }
    }
}

pub struct Constraint {
    indices: Vec<c_int>,
    weights: Vec<c_double>,
    sense: ConstraintType,
    rhs: c_double,
}

impl Constraint {
    fn numnz(&self) -> c_int {
        assert_eq!(self.indices.len(), self.weights.len());
        self.indices.len() as c_int
    }

    pub fn build() -> ConstraintBuilder {
        ConstraintBuilder { vars: vec![], weights: vec![] }
    }
}

pub struct ConstraintBuilder {
    vars: Vec<c_int>,
    weights: Vec<f64>,
}

impl ConstraintBuilder {
    pub fn sum<V: Borrow<VarIndex>, I: IntoIterator<Item=V>>(mut self, iter: I) -> Self {
        use std::iter::repeat;
        let old_len = self.vars.len();
        self.vars.extend(iter.into_iter().map(|x| x.borrow().id()));
        self.weights.extend(repeat(1.0).take(self.vars.len() - old_len));
        self
    }
    
    pub fn weighted_sum<V: Borrow<VarIndex>, F: Borrow<f64>, I: IntoIterator<Item=V>, J: IntoIterator<Item=F>>(mut self, iter: I, weights: J) -> Self {
        self.vars.extend(iter.into_iter().map(|x| x.borrow().id()));
        self.weights.extend(weights.into_iter().map(|f| *f.borrow()));
        self
    }

    pub fn plus(mut self, var: VarIndex, weight: f64) -> ConstraintBuilder {
        self.vars.push(var.id());
        self.weights.push(weight);
        self
    }

    pub fn equals(self, rhs: f64) -> Constraint {
        Constraint {
            indices: self.vars,
            weights: self.weights,
            sense: ConstraintType::Equal,
            rhs: rhs,
        }
    }

    pub fn is_greater_than(self, rhs: f64) -> Constraint {
        Constraint {
            indices: self.vars,
            weights: self.weights,
            sense: ConstraintType::GreaterEqual,
            rhs: rhs,
        }
    }

    pub fn is_less_than(self, rhs: f64) -> Constraint {
        Constraint {
            indices: self.vars,
            weights: self.weights,
            sense: ConstraintType::LessEqual,
            rhs: rhs,
        }
    }
}

pub enum ConstraintType {
    LessEqual,
    GreaterEqual,
    Equal,
}

impl ConstraintType {
    fn sense(&self) -> c_char {
        use ConstraintType::*;
        match self {
            &LessEqual => '<' as c_char,
            &GreaterEqual => '>' as c_char,
            &Equal => '=' as c_char,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum ObjectiveType {
    Minimize,
    Maximize
}

impl ObjectiveType {
    fn sense(&self) -> c_int {
        use ObjectiveType::*;
        match self {
            &Minimize => 1,
            &Maximize => -1,
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum OptimizationStatus {}

pub struct Solution<'a, 'b: 'a> {
    model: &'a Model<'b>
}

impl<'a, 'b: 'a> Solution<'a, 'b> {
    pub fn value(&self) -> Result<f64, &str> {
        unsafe {
            let mut val = 0.0;
            code_to_result(GRBgetdblattr(self.model.inner, name("ObjVal"), &mut val),
                           self.model.env.inner).map(|_| val)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::ptr;

    #[test]
    fn raw_calls() {
        let mut env = ptr::null_mut();
        unsafe {
            assert_eq!(GRBloadenv(&mut env, ptr::null()),
                       0,
                       "failed to instantiate environment");
        }
    }

    #[test]
    fn create_env() {
        let mut env = Env::new();
        assert!(env.inner != ptr::null_mut());
    }

    #[test]
    fn create_model() {
        let mut env = Env::new();
        let mut model = Model::new(&env).unwrap();
        assert!(model.inner != ptr::null_mut());
    }

    #[test]
    fn add_var() {
        let mut env = Env::new();
        let mut model = Model::new(&env).unwrap();
        model.add_var(1.0, VariableType::Binary).unwrap();
    }

    #[test]
    fn add_con() {
        let mut env = Env::new();
        let mut model = Model::new(&env).unwrap();
        let x = model.add_var(1.0, VariableType::Binary).unwrap();
        let y = model.add_var(1.0, VariableType::Binary).unwrap();
        let z = model.add_var(1.0, VariableType::Binary).unwrap();

        let con = Constraint::build().sum(&[x, y]).plus(z, 2.0).equals(3.0);
        model.add_con(con).unwrap();
    }

    #[test]
    fn mip1() {
        let mut env = Env::new();
        let mut model = Model::new(&env).unwrap();
        let x = model.add_var(1.0, VariableType::Binary).unwrap();
        let y = model.add_var(1.0, VariableType::Binary).unwrap();
        let z = model.add_var(1.0, VariableType::Binary).unwrap();

        model.add_con(Constraint::build().plus(x, 1.0).plus(y, 2.0).plus(z, 3.0).is_less_than(4.0)).unwrap();
        model.add_con(Constraint::build().sum(&[x, y]).is_greater_than(1.0)).unwrap();

        model.set_objective_type(ObjectiveType::Maximize);

        let sol = model.optimize().unwrap();
        assert_eq!(sol.value().unwrap(), 1.0);
    }
}
