//! Elimination-backoff stack.

mod base;
mod elim;
mod treiber_stack;

/// Elimination-backoff stack based on Treiber's stack.
pub type ElimStack<T> = base::ElimStack<T, treiber_stack::TreiberStack<T>>;

#[cfg(test)]
mod test {
    use std::thread::scope;

    use base::Stack;

    use super::*;

    #[test]
    fn push() {
        let stack = ElimStack::default();

        scope(|scope| {
            let mut handles = Vec::new();
            for _ in 0..10 {
                let handle = scope.spawn(|| {
                    for i in 0..10_000 {
                        stack.push(i);
                        assert!(stack.pop().is_some());
                    }
                });
                handles.push(handle);
            }
        });

        assert!(stack.pop().is_none());
    }
}
