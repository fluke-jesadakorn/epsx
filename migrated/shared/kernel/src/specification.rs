//! Specification pattern for composable business rules.
//!
//! Source: apps/backend/src/domain/shared_kernel/specification.rs

/// Specification pattern for business rules.
pub trait Specification<T> {
    fn is_satisfied_by(&self, obj: &T) -> bool;

    fn and<S: Specification<T>>(self, other: S) -> AndSpecification<T, Self, S>
    where
        Self: Sized,
    {
        AndSpecification::new(self, other)
    }

    fn or<S: Specification<T>>(self, other: S) -> OrSpecification<T, Self, S>
    where
        Self: Sized,
    {
        OrSpecification::new(self, other)
    }

    fn not(self) -> NotSpecification<T, Self>
    where
        Self: Sized,
    {
        NotSpecification::new(self)
    }
}

/// AND combination of two specifications.
pub struct AndSpecification<T, L: Specification<T>, R: Specification<T>> {
    left: L,
    right: R,
    _phantom: std::marker::PhantomData<T>,
}

impl<T, L: Specification<T>, R: Specification<T>> AndSpecification<T, L, R> {
    pub fn new(left: L, right: R) -> Self {
        Self {
            left,
            right,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T, L: Specification<T>, R: Specification<T>> Specification<T> for AndSpecification<T, L, R> {
    fn is_satisfied_by(&self, obj: &T) -> bool {
        self.left.is_satisfied_by(obj) && self.right.is_satisfied_by(obj)
    }
}

/// OR combination of two specifications.
pub struct OrSpecification<T, L: Specification<T>, R: Specification<T>> {
    left: L,
    right: R,
    _phantom: std::marker::PhantomData<T>,
}

impl<T, L: Specification<T>, R: Specification<T>> OrSpecification<T, L, R> {
    pub fn new(left: L, right: R) -> Self {
        Self {
            left,
            right,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T, L: Specification<T>, R: Specification<T>> Specification<T> for OrSpecification<T, L, R> {
    fn is_satisfied_by(&self, obj: &T) -> bool {
        self.left.is_satisfied_by(obj) || self.right.is_satisfied_by(obj)
    }
}

/// NOT specification (negation).
pub struct NotSpecification<T, S: Specification<T>> {
    spec: S,
    _phantom: std::marker::PhantomData<T>,
}

impl<T, S: Specification<T>> NotSpecification<T, S> {
    pub fn new(spec: S) -> Self {
        Self {
            spec,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T, S: Specification<T>> Specification<T> for NotSpecification<T, S> {
    fn is_satisfied_by(&self, obj: &T) -> bool {
        !self.spec.is_satisfied_by(obj)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct IsPositive;
    impl Specification<i32> for IsPositive {
        fn is_satisfied_by(&self, obj: &i32) -> bool {
            *obj > 0
        }
    }

    struct IsEven;
    impl Specification<i32> for IsEven {
        fn is_satisfied_by(&self, obj: &i32) -> bool {
            obj % 2 == 0
        }
    }

    #[test]
    fn and_combination() {
        let s = IsPositive.and(IsEven);
        assert!(s.is_satisfied_by(&2));
        assert!(!s.is_satisfied_by(&-2));
        assert!(!s.is_satisfied_by(&3));
    }

    #[test]
    fn or_combination() {
        let s = IsPositive.or(IsEven);
        assert!(s.is_satisfied_by(&-2));
        assert!(s.is_satisfied_by(&3));
    }

    #[test]
    fn not_negation() {
        let s = IsPositive.not();
        assert!(!s.is_satisfied_by(&1));
        assert!(s.is_satisfied_by(&-1));
    }
}
