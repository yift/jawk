use std::{str::FromStr, sync::Arc};

use indexmap::IndexMap;

use crate::{
    json_value::JsonValue,
    output::{get_value_or_values, Output},
    selection::{SelectionParseError, UnnamedSelection},
};

#[derive(Clone)]
pub struct Grouper {
    group_by: UnnamedSelection,
}

impl FromStr for Grouper {
    type Err = SelectionParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let group_by = UnnamedSelection::from_str(s)?;
        Ok(Grouper { group_by })
    }
}

impl Grouper {
    pub fn start(&self, rows_titles: Arc<Vec<String>>, output: Box<dyn Output>) -> Box<dyn Output> {
        let data = IndexMap::new();
        let output = if let Some(output) = output.without_titles() {
            output
        } else {
            output
        };
        let grouper = ActiveGrouper {
            data,
            rows_titles: rows_titles.clone(),
            output,
            group_by: self.group_by.clone(),
        };
        Box::new(grouper)
    }
}
struct ActiveGrouper {
    data: IndexMap<String, Vec<JsonValue>>,
    rows_titles: Arc<Vec<String>>,
    output: Box<dyn Output>,
    group_by: UnnamedSelection,
}
impl Output for ActiveGrouper {
    fn output_row(&mut self, value: &JsonValue, row: Vec<Option<JsonValue>>) -> std::fmt::Result {
        if let Some(key) = self.group_by.name(value) {
            let data = get_value_or_values(value, row, &self.rows_titles);
            self.data.entry(key).or_default().push(data);
        }

        Ok(())
    }

    fn done(&mut self) -> std::fmt::Result {
        let mut data = IndexMap::new();
        for (key, value) in self.data.iter() {
            let value = value.clone().into();
            data.insert(key.clone(), value);
        }

        let value = data.into();
        self.output.output_row(&value, vec![])?;
        self.output.done()
    }

    fn without_titles(&self) -> Option<Box<dyn Output>> {
        None
    }
}

#[cfg(test)]
mod tests {
    use std::{mem, sync::Mutex};

    use super::*;

    #[test]
    fn test_from_str_valid_name() -> Result<(), SelectionParseError> {
        let str = ".a";

        let grouper = Grouper::from_str(str)?;

        let mut input = IndexMap::new();
        input.insert("a".to_string(), JsonValue::String("yes".into()));
        input.insert("b".to_string(), JsonValue::String("no".into()));
        assert_eq!(
            grouper.group_by.name(&input.into()),
            Some("yes".to_string())
        );

        Ok(())
    }

    #[test]
    fn test_from_str_invalid_name() -> Result<(), SelectionParseError> {
        let str = "(";

        let result = Grouper::from_str(str);

        assert!(matches!(result, Err(_)));

        Ok(())
    }

    #[test]
    fn test_start_removes_headers() -> Result<(), SelectionParseError> {
        let str = ".a";
        let grouper = Grouper::from_str(str)?;
        let called_by = Arc::new(Mutex::new(None));

        struct MockOutput {
            called_by: Arc<Mutex<Option<bool>>>,
            me: bool,
        }
        impl Output for MockOutput {
            fn output_row(&mut self, _: &JsonValue, _: Vec<Option<JsonValue>>) -> std::fmt::Result {
                let mut called_by = self.called_by.lock().unwrap();
                let _ = mem::replace(&mut *called_by, Some(self.me));
                Ok(())
            }
            fn without_titles(&self) -> Option<Box<dyn Output>> {
                Some(Box::new(MockOutput {
                    called_by: self.called_by.clone(),
                    me: !self.me,
                }))
            }
        }
        let first_object = MockOutput {
            called_by: called_by.clone(),
            me: false,
        };

        let mut started = grouper.start(Arc::new(vec![]), Box::new(first_object));

        started.output_row(&JsonValue::null(), vec![]).unwrap();
        started.done().unwrap();

        assert_eq!(*called_by.lock().unwrap(), Some(true));
        Ok(())
    }

    #[test]
    fn test_group_the_values_correctly() -> Result<(), SelectionParseError> {
        let str = ".a";
        let grouper = Grouper::from_str(str)?;
        let got_values = Arc::new(Mutex::new(Vec::new()));

        struct MockOutput {
            got_values: Arc<Mutex<Vec<IndexMap<String, JsonValue>>>>,
        }
        impl Output for MockOutput {
            fn output_row(
                &mut self,
                value: &JsonValue,
                _: Vec<Option<JsonValue>>,
            ) -> std::fmt::Result {
                let mut got_values = self.got_values.lock().unwrap();
                let map: IndexMap<String, JsonValue> = value.clone().try_into().unwrap();
                got_values.push(map);
                Ok(())
            }
            fn without_titles(&self) -> Option<Box<dyn Output>> {
                None
            }
        }
        let first_object = MockOutput {
            got_values: got_values.clone(),
        };

        let mut started = grouper.start(
            Arc::new(vec!["title-1".to_string(), "title-2".to_string()]),
            Box::new(first_object),
        );

        started
            .output_row(&JsonValue::null(), vec![None, None])
            .unwrap();
        let mut value = IndexMap::new();
        value.insert("a".to_string(), JsonValue::String("one".into()));
        started
            .output_row(&value.into(), vec![None, Some(100.into())])
            .unwrap();
        let mut value = IndexMap::new();
        value.insert("a".to_string(), JsonValue::String("two".into()));
        started
            .output_row(&value.into(), vec![Some(10.into()), Some(150.into())])
            .unwrap();
        let mut value = IndexMap::new();
        value.insert("a".to_string(), JsonValue::String("one".into()));
        started
            .output_row(&value.into(), vec![Some(40.into()), None])
            .unwrap();
        started.done().unwrap();

        assert_eq!(got_values.lock().unwrap().len(), 1);
        let value = got_values.lock().unwrap();
        let value = value.first().unwrap();
        assert_eq!(value.len(), 2);
        let one: Vec<JsonValue> = value.get("one").unwrap().clone().try_into().unwrap();
        let two: Vec<JsonValue> = value.get("two").unwrap().clone().try_into().unwrap();
        assert_eq!(one.len(), 2);
        assert_eq!(two.len(), 1);
        let one_1: IndexMap<String, JsonValue> = one.get(0).unwrap().clone().try_into().unwrap();
        assert_eq!(one_1.len(), 1);
        assert_eq!(one_1.get("title-2").cloned(), Some(100.into()));
        let one_2: IndexMap<String, JsonValue> = one.get(1).unwrap().clone().try_into().unwrap();
        assert_eq!(one_2.len(), 1);
        assert_eq!(one_2.get("title-1").cloned(), Some(40.into()));
        let two: IndexMap<String, JsonValue> = two.get(0).unwrap().clone().try_into().unwrap();
        assert_eq!(two.len(), 2);
        assert_eq!(two.get("title-1").cloned(), Some(10.into()));
        assert_eq!(two.get("title-2").cloned(), Some(150.into()));
        Ok(())
    }
    #[test]
    fn test_without_titles_return_nothing() -> Result<(), SelectionParseError> {
        let str = ".a";
        let grouper = Grouper::from_str(str)?;
        struct MockOutput;
        impl Output for MockOutput {
            fn output_row(&mut self, _: &JsonValue, _: Vec<Option<JsonValue>>) -> std::fmt::Result {
                Ok(())
            }
            fn without_titles(&self) -> Option<Box<dyn Output>> {
                None
            }
        }
        let first_object = MockOutput {};
        let started = grouper.start(
            Arc::new(vec!["title-1".to_string(), "title-2".to_string()]),
            Box::new(first_object),
        );

        let ret = started.without_titles();

        assert_eq!(ret.is_none(), true);
        Ok(())
    }
}
