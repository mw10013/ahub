#[test]
fn test_it() {
    #[derive(Debug)]
    #[allow(dead_code)]
    struct S {
        id: i64,
        name: String,
    }
    let data = vec![
        S {
            id: 7,
            name: "Fee".to_string(),
        },
        S {
            id: 11,
            name: "Fi".to_string(),
        },
    ];
    let mut m = std::collections::HashMap::<i64, S>::new();
    let mut coll = Vec::<&S>::new();
    for d in data {
        m.insert(d.id, d);
    }
    coll.push(m.get(&7).unwrap());
    dbg!(&m);
    dbg!(&coll);
}
