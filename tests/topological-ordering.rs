use std::collections::{BTreeMap, HashMap};

use pearce_kelly::ToplogicalOrdering;

// Invariants:
// After successfully adding an edge x→y to an ordering o, then:
// o.index(x) < o.index(y)

// If adding a vertex to a cycle would create a cycle, then adding it must fail.

#[test]
fn example_empty() {
    let ord = ToplogicalOrdering::<char, ()>::default();

    assert_eq!(ord.iter().count(), 0)
}

#[test]
fn example_1() {
    let mut ord = ToplogicalOrdering::default();
    let a_ix = ord.add_node('a');
    let b_ix = ord.add_node('b');
    println!("a: {:?}; b:{:?};", a_ix, b_ix);

    ord.add_edge(a_ix, b_ix, ()).expect("add edge");

    let ranks = ord
        .iter()
        .enumerate()
        .map(|(ix, a)| (ord.node_weight(*a).cloned().expect("node weight"), ix))
        .collect::<BTreeMap<_, _>>();

    println!("Ranks: {:?}", ranks);
    assert!(
        ranks[&'a'] < ranks[&'b'],
        "a:{:?} < {:?}",
        ranks[&'a'],
        ranks[&'b']
    );
}

#[test]
fn example_abcd() {
    let mut ord = ToplogicalOrdering::default();
    let a_ix = ord.add_node('a');
    let b_ix = ord.add_node('b');
    let c_ix = ord.add_node('c');
    let d_ix = ord.add_node('d');

    println!("a: {:?}; b:{:?}; c:{:?}; d:{:?}", a_ix, b_ix, c_ix, d_ix);

    println!("Add c→d");
    ord.add_edge(c_ix, d_ix, ()).expect("add edge c→d");
    println!(
        "Ordering now: {:?}",
        ord.iter()
            .filter_map(|a| ord.node_weight(*a))
            .collect::<Vec<_>>()
    );
    println!();

    println!("Add a→b");
    ord.add_edge(a_ix, b_ix, ()).expect("add edge a→b");
    println!(
        "Ordering now: {:?}",
        ord.iter()
            .filter_map(|a| ord.node_weight(*a))
            .collect::<Vec<_>>()
    );
    println!();

    println!("Add b→c");
    ord.add_edge(b_ix, c_ix, ()).expect("add edge b→c");
    println!(
        "Ordering now: {:?}",
        ord.iter().map(|a| ord.node_weight(*a)).collect::<Vec<_>>()
    );
    println!();

    let ranks = ord
        .iter()
        .enumerate()
        .map(|(ix, a)| (ord.node_weight(*a).cloned().expect("node weight"), ix))
        .collect::<BTreeMap<_, _>>();

    println!("Ranks: {:?}", ranks);
    assert!(
        ranks[&'a'] < ranks[&'c'],
        "a:{:?} < c:{:?}",
        ranks[&'a'],
        ranks[&'c']
    );

    assert!(
        ranks[&'b'] < ranks[&'d'],
        "b:{:?} < d:{:?}",
        ranks[&'a'],
        ranks[&'b']
    );

    assert!(
        ranks[&'a'] < ranks[&'d'],
        "a:{:?} < d:{:?}",
        ranks[&'a'],
        ranks[&'b']
    );
}

#[test]
fn example_xyzabc() {
    let mut ord = ToplogicalOrdering::default();
    let mut ixes = HashMap::new();
    for w in 'a'..='c' {
        ixes.insert(w, ord.add_node(w));
    }
    for w in 'x'..='z' {
        ixes.insert(w, ord.add_node(w));
    }

    println!("ixes: {:?}", ixes);

    for (src, dst) in [('a', 'b'), ('x', 'y'), ('b', 'c'), ('y', 'z'), ('c', 'x')]
        .iter()
        .cloned()
    {
        println!("Add {}({:?}) → {}({:?})", src, ixes[&src], dst, ixes[&dst]);
        ord.add_edge(ixes[&src], ixes[&dst], ()).expect("add edge");
        println!(
            "Ordering now: {:?}",
            ord.iter()
                .filter_map(|a| ord.node_weight(*a))
                .collect::<Vec<_>>()
        );
        println!();
    }

    let ranks = ord
        .iter()
        .enumerate()
        .map(|(ix, a)| (ord.node_weight(*a).cloned().expect("node weight"), ix))
        .collect::<BTreeMap<_, _>>();

    println!("Ranks: {:?}", ranks);
    assert!(
        ranks[&'a'] < ranks[&'c'],
        "a:{:?} < c:{:?}",
        ranks[&'a'],
        ranks[&'c']
    );

    assert!(
        ranks[&'x'] < ranks[&'z'],
        "x:{:?} < z:{:?}",
        ranks[&'x'],
        ranks[&'z']
    );

    assert!(
        ranks[&'b'] < ranks[&'y'],
        "b:{:?} < y:{:?}",
        ranks[&'b'],
        ranks[&'y']
    );
}

#[test]
fn example_1_rev() {
    let mut ord = ToplogicalOrdering::default();
    let b_ix = ord.add_node('b');
    let a_ix = ord.add_node('a');
    println!("a: {:?}; b:{:?};", a_ix, b_ix);

    ord.add_edge(a_ix, b_ix, ()).expect("add edge");

    let ranks = ord
        .iter()
        .enumerate()
        .map(|(ix, a)| (ord.node_weight(*a).cloned().expect("node weight"), ix))
        .collect::<BTreeMap<_, _>>();

    println!("Ranks: {:?}", ranks);
    assert!(
        ranks[&'a'] < ranks[&'b'],
        "a:{:?} < {:?}",
        ranks[&'a'],
        ranks[&'b']
    );
}

#[test]
fn example_2() {
    let mut ord = ToplogicalOrdering::default();
    let a_ix = ord.add_node('a');
    let b_ix = ord.add_node('b');
    let c_ix = ord.add_node('c');
    println!("a: {:?}; b:{:?}; c:{:?};", a_ix, b_ix, c_ix);

    ord.add_edge(a_ix, b_ix, ()).expect("Add edge");
    ord.add_edge(a_ix, c_ix, ()).expect("Add edge");

    let ranks = ord
        .iter()
        .enumerate()
        .map(|(ix, a)| (a, ix))
        .collect::<BTreeMap<_, _>>();

    println!("Ranks: {:?}", ranks);
    assert!(
        ranks[&a_ix] < ranks[&b_ix],
        "a:{:?} < b:{:?}",
        ranks[&a_ix],
        ranks[&b_ix]
    );

    assert!(
        ranks[&a_ix] < ranks[&c_ix],
        "a:{:?} < c:{:?}",
        ranks[&a_ix],
        ranks[&c_ix]
    );
}

#[test]
fn example_2_rev() {
    let mut ord = ToplogicalOrdering::default();
    let c_ix = ord.add_node('c');
    let b_ix = ord.add_node('b');
    let a_ix = ord.add_node('a');
    println!("a: {:?}; b:{:?}; c:{:?};", a_ix, b_ix, c_ix);

    ord.add_edge(a_ix, b_ix, ()).expect("Add edge");
    ord.add_edge(a_ix, c_ix, ()).expect("Add edge");

    let ranks = ord
        .iter()
        .enumerate()
        .map(|(ix, a)| (a, ix))
        .collect::<BTreeMap<_, _>>();

    println!("Ranks: {:?}", ranks);
    assert!(
        ranks[&a_ix] < ranks[&b_ix],
        "a:{:?} < b:{:?}",
        ranks[&a_ix],
        ranks[&b_ix]
    );

    assert!(
        ranks[&a_ix] < ranks[&c_ix],
        "a:{:?} < c:{:?}",
        ranks[&a_ix],
        ranks[&c_ix]
    );
}

#[test]
fn inserts_each_item_only_once() {
    let mut ord = ToplogicalOrdering::default();
    let a_ix = ord.add_node('a');
    let b_ix = ord.add_node('b');
    let c_ix = ord.add_node('c');
    println!("a: {:?}; b:{:?}; c:{:?};", a_ix, b_ix, c_ix);

    ord.add_edge(a_ix, b_ix, ()).expect("Add edge");
    ord.add_edge(a_ix, c_ix, ()).expect("Add edge");

    assert_eq!(ord.iter().filter(|v| **v == a_ix).count(), 1);
}

#[test]
#[ignore]
fn must_fail_on_cycle_insertion_2_node() {
    todo!()
}

#[test]
#[ignore]
fn must_fail_on_cycle_insertion_3_node() {
    todo!()
}
