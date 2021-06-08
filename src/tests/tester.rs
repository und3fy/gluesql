use {
    crate::{
        ast::{IndexItem, SetExpr, Statement, TableFactor},
        executor::{execute, Payload},
        parse_sql::parse,
        plan::plan,
        result::Result,
        store::{GStore, GStoreMut},
        translate::translate,
    },
    async_trait::async_trait,
    std::{cell::RefCell, fmt::Debug, rc::Rc},
};

pub async fn run<T: 'static + Debug, U: GStore<T> + GStoreMut<T>>(
    cell: Rc<RefCell<Option<U>>>,
    sql: &str,
    indexes: Option<Vec<IndexItem>>,
) -> Result<Payload> {
    let storage = cell.replace(None).unwrap();

    macro_rules! try_run {
        ($expr: expr) => {
            match $expr {
                Ok(v) => v,
                Err(e) => {
                    cell.replace(Some(storage));

                    return Err(e);
                }
            }
        };
    }

    let parsed = try_run!(parse(sql));
    let statement = try_run!(translate(&parsed[0]));
    let statement = try_run!(plan(&storage, statement).await);

    if let Some(expected) = indexes {
        let found = find_indexes(&statement);

        if expected.len() != found.len() {
            panic!(
                "num of indexes does not match: expected({}) != found({})",
                expected.len(),
                found.len(),
            );
        }

        for expected_index in expected {
            if !found
                .iter()
                .any(|found_index| *found_index == &expected_index)
            {
                panic!("index does not exist: {:#?}", expected_index)
            }
        }
    }

    match execute(storage, &statement).await {
        Ok((storage, payload)) => {
            cell.replace(Some(storage));

            Ok(payload)
        }
        Err((storage, error)) => {
            cell.replace(Some(storage));

            Err(error)
        }
    }
}

fn find_indexes(statement: &Statement) -> Vec<&IndexItem> {
    let body = match statement {
        Statement::Query(query) => &query.body,
        _ => {
            return vec![];
        }
    };

    let table_factor = match body {
        SetExpr::Select(select) => &select.from.relation,
        _ => {
            return vec![];
        }
    };

    let index = match table_factor {
        TableFactor::Table {
            index: Some(index), ..
        } => index,
        _ => {
            return vec![];
        }
    };

    vec![index]
}

/// If you want to make your custom storage and want to run integrate tests,
/// you should implement this `Tester` trait.
///
/// To see how to use it,
/// * [tests/sled_storage.rs](https://github.com/gluesql/gluesql/blob/main/tests/sled_storage.rs)
///
/// Actual test cases are in [/src/tests/](https://github.com/gluesql/gluesql/blob/main/src/tests/),
/// not in `/tests/`.
#[async_trait]
pub trait Tester<T: 'static + Debug, U: GStore<T> + GStoreMut<T>> {
    fn new(namespace: &str) -> Self;

    fn get_cell(&mut self) -> Rc<RefCell<Option<U>>>;
}

#[macro_export]
macro_rules! test_case {
    ($name: ident, $content: expr) => {
        pub async fn $name<T, U>(mut tester: impl tests::Tester<T, U>)
        where
            T: 'static + std::fmt::Debug,
            U: GStore<T> + GStoreMut<T>,
        {
            use std::rc::Rc;

            let cell = tester.get_cell();

            #[allow(unused_macros)]
            macro_rules! run {
                ($sql: expr) => {
                    tests::run(Rc::clone(&cell), $sql, None).await.unwrap()
                };
            }

            #[allow(unused_macros)]
            macro_rules! count {
                ($count: expr, $sql: expr) => {
                    match tests::run(Rc::clone(&cell), $sql, None).await.unwrap() {
                        Payload::Select { rows, .. } => assert_eq!($count, rows.len()),
                        Payload::Delete(num) => assert_eq!($count, num),
                        Payload::Update(num) => assert_eq!($count, num),
                        _ => panic!("compare is only for Select, Delete and Update"),
                    };
                };
            }

            #[allow(unused_macros)]
            macro_rules! test {
                ($expected: expr, $sql: expr) => {
                    let found = tests::run(Rc::clone(&cell), $sql, None).await;

                    test($expected, found);
                };
            }

            #[allow(unused_macros)]
            macro_rules! test_idx {
                ($expected: expr, $indexes: expr, $sql: expr) => {
                    let found = tests::run(Rc::clone(&cell), $sql, Some($indexes)).await;

                    test($expected, found);
                };
            }

            #[allow(unused)]
            fn test(expected: Result<Payload>, found: Result<Payload>) {
                let (expected, found): (Payload, Payload) = match (expected, found) {
                    (Ok(a), Ok(b)) => (a, b),
                    (a, b) => {
                        assert_eq!(a, b);

                        return;
                    }
                };

                let (expected, found) = match (expected, found) {
                    (
                        Payload::Select {
                            labels: expected_labels,
                            rows: a,
                        },
                        Payload::Select {
                            labels: found_labels,
                            rows: b,
                        },
                    ) => {
                        assert_eq!(expected_labels, found_labels);

                        (a, b)
                    }
                    (a, b) => {
                        assert_eq!(a, b);

                        return;
                    }
                };

                assert_eq!(
                    expected.len(),
                    found.len(),
                    "\n[err: number of rows]\nexpected: {:?}\n   found: {:?}",
                    expected,
                    found
                );

                let rows = expected.into_iter().zip(found.into_iter()).enumerate();

                for (i, (expected, found)) in rows.into_iter() {
                    let Row(expected) = expected;
                    let Row(found) = found;

                    assert_eq!(
                        expected.len(),
                        found.len(),
                        "\n[err: size of row] row index: {}\nexpected: {:?}\n   found: {:?}",
                        i,
                        expected,
                        found
                    );

                    expected
                        .iter()
                        .zip(found.iter())
                        .for_each(|(expected_val, found_val)| {
                            if matches!((expected_val, found_val), (&Value::Null, &Value::Null)) {
                                return;
                            }

                            assert_eq!(
                                expected_val, found_val,
                                "\n[err: value] row index: {}\nexpected: {:?}\n   found: {:?}",
                                i, expected, found
                            );
                        });
                }
            }

            $content.await
        }
    };
}
