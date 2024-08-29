use arith::Field;

use crate::{
    selectors::Selector,
    variable::{VariableColumn, VariableIndex, VariableOne, VariableZero, Variables},
};

/// Constraint system for the vanilla plonk protocol.
///
/// Vanilla plonk gate:
///
/// q_l * a + q_r * b + q_o * c + q_m * a * b + q_c = 0
///
/// where
/// - `a`, `b`, `c` are the variables of the constraint system.
/// - `q_l`, `q_r`, `q_o`, `q_m` are the coefficients of the constraint system.
/// - `q_c` is the constant term of the constraint system.
///
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct ConstraintSystem<F> {
    /// selectors
    pub q_l: Selector<F>,
    pub q_r: Selector<F>,
    pub q_o: Selector<F>,
    pub q_m: Selector<F>,
    pub q_c: Selector<F>,

    /// those are the indexes of the witnesses
    pub a: VariableColumn,
    pub b: VariableColumn,
    pub c: VariableColumn,

    /// the actual witnesses
    pub variables: Variables<F>,
}

impl<F: Field> ConstraintSystem<F> {
    /// initialize a new constraint system with default constants
    #[inline]
    pub fn init() -> Self {
        let mut cs = ConstraintSystem::default();

        let zero_var = cs.variables.new_variable(F::zero());
        let one_var = cs.variables.new_variable(F::one());

        // assert the first witness is 0
        {
            cs.q_l.push(F::one());
            cs.q_r.push(F::zero());
            cs.q_o.push(F::zero());
            cs.q_m.push(F::zero());
            cs.q_c.push(F::zero());

            cs.a.push(zero_var);
            cs.b.push(zero_var);
            cs.c.push(zero_var);
        }
        // assert the second witness is 1
        {
            cs.q_l.push(F::one());
            cs.q_r.push(F::zero());
            cs.q_o.push(F::zero());
            cs.q_m.push(F::zero());
            cs.q_c.push(-F::one());

            cs.a.push(one_var);
            cs.b.push(zero_var);
            cs.c.push(zero_var);
        }
        cs
    }

    /// create a new variable
    #[inline]
    pub fn new_variable(&mut self, f: F) -> VariableIndex {
        self.variables.new_variable(f)
    }

    /// get the field element of a variable
    #[inline]
    pub fn get_value(&self, index: VariableIndex) -> F {
        self.variables.witnesses[index]
    }

    /// constant gate
    #[inline]
    pub fn constant_gate(&mut self, c: &F) -> VariableIndex {
        let var_c = self.new_variable(*c);

        self.q_l.push(F::one());
        self.q_r.push(F::zero());
        self.q_o.push(F::zero());
        self.q_m.push(F::zero());
        self.q_c.push(-*c);

        self.a.push(var_c);
        self.b.push(VariableZero);
        self.c.push(VariableZero);

        var_c
    }

    /// Assert the variable is zero
    #[inline]
    pub fn assert_zero(&mut self, a: &VariableIndex) {
        let a_val = self.get_value(*a);
        assert!(a_val == F::zero(), "a should be zero");

        self.q_l.push(F::one());
        self.q_r.push(F::zero());
        self.q_o.push(F::zero());
        self.q_m.push(F::zero());
        self.q_c.push(F::zero());

        self.a.push(*a);
        self.b.push(VariableZero);
        self.c.push(VariableZero);
    }

    /// Assert the variable is one
    #[inline]
    pub fn assert_one(&mut self, a: &VariableIndex) {
        let a_val = self.get_value(*a);
        assert!(a_val == F::one(), "a should be one");

        self.q_l.push(F::one());
        self.q_r.push(F::zero());
        self.q_o.push(-F::one());
        self.q_m.push(F::zero());
        self.q_c.push(F::zero());

        self.a.push(*a);
        self.b.push(VariableZero);
        self.c.push(VariableZero);
    }

    /// Assert the variable is binary
    ///
    /// this is handled by constraint `a * (a - 1) = 0`
    #[inline]
    pub fn assert_binary(&mut self, a: &VariableIndex) {
        let a_val = self.get_value(*a);
        assert!(
            a_val == F::zero() || a_val == F::one(),
            "a should be binary"
        );

        self.q_l.push(-F::one());
        self.q_r.push(F::zero());
        self.q_o.push(F::zero());
        self.q_m.push(F::one());
        self.q_c.push(F::zero());

        self.a.push(*a);
        self.b.push(*a);
        self.c.push(VariableZero);
    }

    /// Assert the variable is not zero
    ///
    /// this is handled by adding a new variable `a_inv` and asserting `a * a_inv = 1`
    #[inline]
    pub fn assert_none_zero(&mut self, a: &VariableIndex) {
        let a_val = self.get_value(*a);
        assert!(a_val != F::zero(), "a should not be zero");
        let a_inv = a_val.inv().unwrap(); // safe unwrap
        let a_inv_var = self.new_variable(a_inv);

        self.q_l.push(F::zero());
        self.q_r.push(F::zero());
        self.q_o.push(-F::one());
        self.q_m.push(F::one());
        self.q_c.push(F::zero());

        self.a.push(*a);
        self.b.push(a_inv_var);
        self.c.push(VariableOne);
    }

    /// addition gate: return the variable index of a + b
    #[inline]
    pub fn addition_gate(&mut self, a: &VariableIndex, b: &VariableIndex) -> VariableIndex {
        let a_val = self.get_value(*a);
        let b_val = self.get_value(*b);
        let c_val = a_val + b_val;
        let c = self.new_variable(c_val);

        self.assert_addition(a, b, &c);
        c
    }

    /// assert addition is correct: c = a + b
    #[inline]
    pub fn assert_addition(&mut self, a: &VariableIndex, b: &VariableIndex, c: &VariableIndex) {
        let a_val = self.get_value(*a);
        let b_val = self.get_value(*b);
        let c_val = self.get_value(*c);

        self.q_l.push(F::one());
        self.q_r.push(F::one());
        self.q_o.push(-F::one());
        self.q_m.push(F::zero());
        self.q_c.push(F::zero());

        self.a.push(*a);
        self.b.push(*b);
        self.c.push(*c);
    }

    /// subtraction gate: return the variable index of a - b
    #[inline]
    pub fn subtraction_gate(&mut self, a: &VariableIndex, b: &VariableIndex) -> VariableIndex {
        let a_val = self.get_value(*a);
        let b_val = self.get_value(*b);
        let c_val = a_val - b_val;
        let c = self.new_variable(c_val);

        self.assert_subtraction(a, b, &c);

        c
    }

    /// assert subtraction is correct: c = a - b
    #[inline]
    pub fn assert_subtraction(&mut self, a: &VariableIndex, b: &VariableIndex, c: &VariableIndex) {
        self.assert_addition(c, b, a)
    }

    /// multiplication gate: return the variable index of a * b
    #[inline]
    pub fn multiplication_gate(&mut self, a: &VariableIndex, b: &VariableIndex) -> VariableIndex {
        let a_val = self.get_value(*a);
        let b_val = self.get_value(*b);
        let c_val = a_val * b_val;
        let c = self.new_variable(c_val);

        self.assert_multiplication(a, b, &c);

        c
    }

    /// assert multiplication is correct: c = a * b
    #[inline]
    pub fn assert_multiplication(
        &mut self,
        a: &VariableIndex,
        b: &VariableIndex,
        c: &VariableIndex,
    ) {
        let a_val = self.get_value(*a);
        let b_val = self.get_value(*b);
        let c_val = self.get_value(*c);

        self.q_l.push(F::zero());
        self.q_r.push(F::zero());
        self.q_o.push(-F::one());
        self.q_m.push(F::one());
        self.q_c.push(F::zero());

        self.a.push(*a);
        self.b.push(*b);
        self.c.push(*c);
    }

    /// division gate: return the variable index of a / b
    #[inline]
    pub fn division_gate(&mut self, a: &VariableIndex, b: &VariableIndex) -> VariableIndex {
        self.assert_none_zero(b);
        let a_val = self.get_value(*a);
        let b_val = self.get_value(*b);
        let c_val = a_val * b_val.inv().unwrap(); // safe unwrap
        let c = self.new_variable(c_val);

        self.assert_division(a, b, &c);

        c
    }

    /// assert division is correct: c = a / b
    #[inline]
    pub fn assert_division(&mut self, a: &VariableIndex, b: &VariableIndex, c: &VariableIndex) {
        self.assert_multiplication(c, b, a)
    }

    /// check the constraint system is satisfied
    #[inline]
    pub fn is_satisfied(&self) -> bool {
        let length = self.q_l.get_nv();

        if self.q_r.get_nv() != length {
            return false;
        }
        if self.q_o.get_nv() != length {
            return false;
        }
        if self.q_m.get_nv() != length {
            return false;
        }
        if self.q_c.get_nv() != length {
            return false;
        }

        for index in 0..length {
            let a = self.get_value(self.a[index]);
            let b = self.get_value(self.b[index]);
            let c = self.get_value(self.c[index]);

            let q_l = self.q_l.q[index];
            let q_r = self.q_r.q[index];
            let q_o = self.q_o.q[index];
            let q_m = self.q_m.q[index];
            let q_c = self.q_c.q[index];

            if a * q_l + b * q_r + c * q_o + a * b * q_m + q_c != F::zero() {
                println!("cs failed at row {}", index);
                println!("a: {:?}", a);
                println!("b: {:?}", b);
                println!("c: {:?}", c);
                println!("q_l: {:?}", q_l);
                println!("q_r: {:?}", q_r);
                println!("q_o: {:?}", q_o);
                println!("q_m: {:?}", q_m);
                println!("q_c: {:?}", q_c);
                return false;
            }
        }

        true
    }
}