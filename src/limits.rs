use crate::processor::{Context, Process, ProcessDecision, Result as ProcessResult, Titles};

pub struct Limiter {
    skip: u64,
    limit: Option<u64>,
    skipped: u64,
    passed: u64,
    next: Box<dyn Process>,
}

impl Limiter {
    pub fn create_process(
        skip: u64,
        limit: Option<u64>,
        next: Box<dyn Process>,
    ) -> Box<dyn Process> {
        if skip == 0 && limit.is_none() {
            next
        } else {
            Box::new(Limiter {
                skip,
                limit,
                skipped: 0,
                passed: 0,
                next,
            })
        }
    }
}

impl Process for Limiter {
    fn complete(&mut self) -> ProcessResult<()> {
        Ok(())
    }
    fn process(&mut self, context: Context) -> ProcessResult<ProcessDecision> {
        if self.skipped < self.skip {
            self.skipped += 1;
            Ok(ProcessDecision::Continue)
        } else if let Some(limit) = self.limit {
            if self.passed >= limit {
                Ok(ProcessDecision::Break)
            } else {
                self.next.process(context)?;
                self.passed += 1;
                if self.passed >= limit {
                    Ok(ProcessDecision::Break)
                } else {
                    Ok(ProcessDecision::Continue)
                }
            }
        } else {
            self.next.process(context)
        }
    }
    fn start(&mut self, titles_so_far: Titles) -> ProcessResult<()> {
        self.next.start(titles_so_far)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use std::rc::Rc;

    use super::*;
    use crate::processor::{Context, Titles};

    #[test]
    fn take_will_take_as_much_as_needed() -> ProcessResult<()> {
        struct Next(Rc<RefCell<usize>>);
        impl Process for Next {
            fn complete(&mut self) -> ProcessResult<()> {
                Ok(())
            }
            fn process(&mut self, _: Context) -> ProcessResult<ProcessDecision> {
                *self.0.borrow_mut() += 1;
                Ok(ProcessDecision::Continue)
            }
            fn start(&mut self, _: Titles) -> ProcessResult<()> {
                Ok(())
            }
        }
        let data = Rc::new(RefCell::new(0));
        let next = Box::new(Next(data.clone()));
        let mut limiter = Limiter::create_process(5, Some(2), next);
        let mut results = Vec::new();
        for i in 0..10 {
            let input = Context::new_with_no_context(i.into());
            let decision = limiter.process(input)?;
            results.push(decision);
        }

        let binding = data.borrow();
        let data = &*binding;
        assert_eq!(data, &2);
        assert_eq!(results[0], ProcessDecision::Continue);
        assert_eq!(results[1], ProcessDecision::Continue);
        assert_eq!(results[2], ProcessDecision::Continue);
        assert_eq!(results[3], ProcessDecision::Continue);
        assert_eq!(results[4], ProcessDecision::Continue);
        assert_eq!(results[5], ProcessDecision::Continue);
        assert_eq!(results[6], ProcessDecision::Break);
        assert_eq!(results[7], ProcessDecision::Break);
        assert_eq!(results[8], ProcessDecision::Break);
        assert_eq!(results[9], ProcessDecision::Break);

        Ok(())
    }

    #[test]
    fn no_limit_will_just_skip() -> ProcessResult<()> {
        struct Next(Rc<RefCell<usize>>);
        impl Process for Next {
            fn complete(&mut self) -> ProcessResult<()> {
                Ok(())
            }
            fn process(&mut self, _: Context) -> ProcessResult<ProcessDecision> {
                *self.0.borrow_mut() += 1;
                Ok(ProcessDecision::Continue)
            }
            fn start(&mut self, _: Titles) -> ProcessResult<()> {
                Ok(())
            }
        }
        let data = Rc::new(RefCell::new(0));
        let next = Box::new(Next(data.clone()));
        let mut limiter = Limiter::create_process(5, None, next);
        let mut results = Vec::new();
        for i in 0..10 {
            let input = Context::new_with_no_context(i.into());
            let decision = limiter.process(input)?;
            results.push(decision);
        }

        let binding = data.borrow();
        let data = &*binding;
        assert_eq!(data, &5);
        assert_eq!(results[0], ProcessDecision::Continue);
        assert_eq!(results[1], ProcessDecision::Continue);
        assert_eq!(results[2], ProcessDecision::Continue);
        assert_eq!(results[3], ProcessDecision::Continue);
        assert_eq!(results[4], ProcessDecision::Continue);
        assert_eq!(results[5], ProcessDecision::Continue);
        assert_eq!(results[6], ProcessDecision::Continue);
        assert_eq!(results[7], ProcessDecision::Continue);
        assert_eq!(results[8], ProcessDecision::Continue);
        assert_eq!(results[9], ProcessDecision::Continue);

        Ok(())
    }
}
