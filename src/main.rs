use std::ops::{Add, AddAssign};
use std::time::{Instant};

#[derive(Debug, Copy, Clone)]
struct Vector<T, const M : usize> {
    elements : [T ; M]
}

trait Init {
    fn zero() -> Self;
    fn one() -> Self;
}

impl Init for f32 {
    fn zero() -> f32 {
        0.0f32
    }
    fn one() -> f32 {
        1.0f32
    }
}

impl Init for f64 {
    fn zero() -> f64 {
        0.0f64
    }
    fn one() -> f64 {
        1.0f64
    }
}

impl<T, const M : usize>  Vector<T, M> where T : Copy + Init {
    fn zeros() -> Vector<T, M> {
        Vector{elements: [T::zero(); M]}
    }
    fn ones() -> Vector<T, M> {
        Vector{elements: [T::one(); M]}
    }
}

impl<T, const M : usize> 
Add<Vector<T, M>> for Vector<T, M>
    where T : Copy + Init + Add<Output = T> + AddAssign
 {
    type Output = Vector<T, M>;
    fn add(self, other: Vector<T, M> ) -> Vector<T, M> {
        let mut v : Vector<T, M>  = Vector::zeros();
        for i in 0..M {
            v.elements[i] = self.elements[i] + other.elements[i];
        }
        v
    }
}

impl<T, const M : usize> 
AddAssign<Vector<T, M>> for Vector<T, M>
    where T : Copy + Init + AddAssign
 {
    fn add_assign(&mut self, other: Vector<T, M> ) {
        for i in 0..M {
             self.elements[i] += other.elements[i];
        }
    }
}

//////////////////////////////////////////////////////////////////////


trait LazyElementEval<const N : usize> { 
    type Output;
    fn eval(&self, offset : usize) -> Self::Output;
}

//////////////////////////////////////////////////////////////////////

#[derive(Debug, Copy, Clone)]
struct SmartVector<T, const M : usize> {
    elements : [T ; M]
}

impl<T, const M : usize>  SmartVector<T, M> where T : Copy + Init + AddAssign {
    fn zeros() -> SmartVector<T, M> {
        SmartVector{elements: [T::zero(); M]}
    }
    fn ones() -> SmartVector<T, M> {
        SmartVector{elements: [T::one(); M]}
    }
    /*
    fn assign<U>(&mut self, arg : &U) where U : LazyElementEval<M, Output=T> {
        for i in 0.. M {
            self.elements[i] = arg.eval(i);
        }
    }
    */
    fn add_assign<U>(&mut self, arg : &U) where U : LazyElementEval<M, Output=T>  {
        for i in 0.. M {
            self.elements[i] += arg.eval(i);
        }
    }
}

impl<T, const M : usize> LazyElementEval<M> for SmartVector<T, M> 
    where T : Copy
{
    type Output = T;
    fn eval(&self, offset : usize) -> Self::Output {
        self.elements[offset]
    }
}

impl<'a, 'b, T, U, const M : usize> 
Add<&'b U> for &'a SmartVector<T, M>
    where T : Copy + Init + Add<Output = T> + AddAssign,
    U : LazyElementEval<M, Output=T>
 {
    type Output = AddBinOp<'a, 'b, SmartVector<T, M>, U, T, M>;
    fn add(self, other: &'b U ) -> Self::Output {
        AddBinOp{left: self, right: other}
    }
}

// Not possible thanks to orphan impls
/*
impl <'a, 'b, A, B, T> Add<&'b B> for &'a A 
where 
    A : LazyElementEval<Output=T>,
    B : LazyElementEval<Output=T>
{
    type Output = AddBinOp<'a, 'b, A, B, T>;
    fn add(self, other: &'b B ) -> Self::Output {
        AddBinOp{left: self, right: other}
    }
}
*/

//////////////////////////////////////////////////////////////////////
struct AddBinOp<'a, 'b, Left, Right, T, const N : usize> 
    where Left : LazyElementEval<N, Output=T>,
    Right : LazyElementEval<N, Output=T>
{
    left : &'a Left,
    right : &'b Right,
}

impl<'a, 'b, Left, Right, T, const N : usize> LazyElementEval<N> for AddBinOp<'a, 'b, Left, Right , T, N> 
    where Left : LazyElementEval<N, Output=T>,
    Right : LazyElementEval<N, Output=T>,
    T: Add<Output = T>
{
    type Output = T;
    fn eval(&self, offset : usize) -> Self::Output {
        self.left.eval(offset) + self.right.eval(offset)
    }
}

impl <'a, 'b, 'c, 'd, Left, Right, T, U, const N : usize> Add<&'c U> for &'d AddBinOp<'a, 'b, Left, Right, T, N> 
    where Left : LazyElementEval<N, Output=T>,
    Right : LazyElementEval<N, Output=T>,
    U : LazyElementEval<N, Output=T>,
    T: Add<Output = T>
{
    type Output = AddBinOp<'d, 'c, AddBinOp<'a, 'b, Left, Right, T, N> , U, T, N>;
    fn add(self, other: &'c U ) -> Self::Output {
        AddBinOp{left: self, right: other}
    }
}

/*
impl<'a, 'b, T, U, const M : usize> 
Add<&'b U> for &'a SmartVector<T, M>
    where T : Copy + Init + Add<Output = T> + AddAssign,
    U : LazyElementEval<Output=T>
 {
    type Output = AddBinOp<'a, 'b, SmartVector<T, M>, U, T>;
    fn add(self, other: &'b U ) -> AddBinOp<'a, 'b, SmartVector<T, M>, U, T> {
        AddBinOp{left: self, right: other}
    }
}
*/
const N : usize = 4;

//////////////////////////////////////////////////////////////////////

fn test1() {
    // Simple and nice, but probably slow and no way to improve.
    let before = Instant::now();
    let mut sum : Vector<f64, N> = Vector::ones();
    let a : Vector<f64, N> = Vector::ones();
    let b : Vector<f64, N> = Vector::ones();
    let c : Vector<f64, N> = Vector::ones();
    let d : Vector<f64, N> = Vector::zeros();

    for _ in 0..10000 {
        sum += a + b + c + d;
    }
    let after = Instant::now();
    println!("{:?}", after.duration_since(before));
    //println!("{:?}", sum);
}

fn test2() {
    // This appears to be much faster, and still typesafe. What are the downsides?
    // - Add needs to be implemented twice on SmartVector and AddBinOp for every binary operation. This
    //   is likely because Add/Sub/Mul/etc are foreign traits we because of "orphan implementations" we
    //   can't implement generically. 
    // - Each binary operation on vectors requires references to it's operands. This seems to require
    //   explicit borrows to on each operand for every binary operation, resulting in beatiful expressions
    //   like  &(&(&1 + &2) + &3). Yuck.
    // - Asssign can't be overloaded, so we use a function for assignment rather the operand:
    //   Z.assign(&(&X + &Y))
    //
    // What are the next steps?
    // - Don't bother using standard math operators so that binary op implementations don't need to be duplicated
    // - Try implementing more binary operations, scalar operations


    let before = Instant::now();
    let mut sum : SmartVector<f64, N> = SmartVector::ones();
    let a : SmartVector<f64, N> = SmartVector::ones();
    let b : SmartVector<f64, N> = SmartVector::ones();
    let c : SmartVector<f64, N> = SmartVector::ones();
    let d : SmartVector<f64, N> = SmartVector::zeros();

    for _ in 0..10000 {
        sum.add_assign(&(&(&a + &b) + &(&c + &d)));
    }

    let after = Instant::now();
    println!("{:?}", after.duration_since(before));
    //println!("{:?}", sum);
}

fn main() {
    test1();
    test2();
}
