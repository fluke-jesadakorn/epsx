// kernel extraction wave9 — moved verbatim from apps/backend/src/domain/shared_kernel/specification.rs
/// Specification pattern for business rules
/// Encapsulates business rules that can be combined and reused
pub trait Specification<T> {
    /// Check if the specification is satisfied by the given object
    fn is_satisfied_by(&self, obj: &T) -> bool;
    
    /// Combine with another specification using AND logic
    fn and<S: Specification<T>>(self, other: S) -> AndSpecification<T, Self, S>
    where
        Self: Sized,
    {
        AndSpecification::new(self, other)
    }
    
    /// Combine with another specification using OR logic
    fn or<S: Specification<T>>(self, other: S) -> OrSpecification<T, Self, S>
    where
        Self: Sized,
    {
        OrSpecification::new(self, other)
    }
    
    /// Negate this specification
    fn not(self) -> NotSpecification<T, Self>
    where
        Self: Sized,
    {
        NotSpecification::new(self)
    }
}

/// AND combination of two specifications
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

/// OR combination of two specifications
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

/// NOT specification (negation)
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
