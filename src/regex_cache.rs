use std::cell::RefCell;
use std::ops::Deref;
use std::rc::Rc;

use cached::Cached;
use cached::SizedCache;
use regex::{Error, Regex};

type OptionalCache = Option<Rc<RefCell<SizedCache<String, Rc<Result<Regex, Error>>>>>>;
#[derive(Clone)]
pub struct RegexCache {
    cache: OptionalCache,
}
pub trait RegexCompile {
    fn compile_regex(&self, regex: &str) -> Rc<Result<Regex, Error>>;
}
impl RegexCompile for RegexCache {
    fn compile_regex(&self, regex: &str) -> Rc<Result<Regex, Error>> {
        match &self.cache {
            None => Rc::new(Regex::new(regex)),
            Some(cache) => cache
                .deref()
                .borrow_mut()
                .cache_get_or_set_with(regex.into(), || Rc::new(Regex::new(regex)))
                .clone(),
        }
    }
}
impl RegexCache {
    pub fn new(size: usize) -> Self {
        let cache = if size > 0 {
            Some(Rc::new(RefCell::new(SizedCache::with_size(size))))
        } else {
            None
        };
        RegexCache { cache }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_compile_with_no_cache() -> Result<(), Error> {
        let cach = RegexCache::new(0);
        match cach.compile_regex("[a-z]+").deref() {
            Ok(regex) => {
                assert_eq!(regex.is_match("hello"), true);
                assert_eq!(regex.is_match("1234"), false);
            }
            Err(e) => return Err(e.clone()),
        }
        Ok(())
    }

    #[test]
    fn can_recompile_with_cache() -> Result<(), Error> {
        let cach = RegexCache::new(10);
        match cach.compile_regex("[a-z]+").deref() {
            Ok(regex) => {
                assert_eq!(regex.is_match("hello"), true);
                assert_eq!(regex.is_match("1234"), false);
            }
            Err(e) => return Err(e.clone()),
        }

        match cach.compile_regex("[a-z]+").deref() {
            Ok(regex) => {
                assert_eq!(regex.is_match("hello"), true);
                assert_eq!(regex.is_match("1234"), false);
            }
            Err(e) => return Err(e.clone()),
        }
        match cach.compile_regex("[a-z]+").deref() {
            Ok(regex) => {
                assert_eq!(regex.is_match("hello"), true);
                assert_eq!(regex.is_match("1234"), false);
            }
            Err(e) => return Err(e.clone()),
        }
        match cach.compile_regex("[a-z]+").deref() {
            Ok(regex) => {
                assert_eq!(regex.is_match("hello"), true);
                assert_eq!(regex.is_match("1234"), false);
            }
            Err(e) => return Err(e.clone()),
        }
        Ok(())
    }
}
