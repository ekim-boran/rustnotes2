#[test]
fn mytest() {
    #[derive(Debug)]
    struct Foo {
        a: String,
        b: String,
        c: String,
    }
    struct S<'a> {
        a: &'a String,
        b: &'a String,
    };

    fn asd(a: &mut String, b: &S) {
        a.push_str(&b.a);
    }

    let mut x = Foo {
        a: "asd".to_string(),
        b: "asd".to_string(),
        c: "asd".to_string(),
    };
    let s = vec![S { a: &x.a, b: &x.b }];
    asd(&mut x.c, &s[0]);
    println!("{:?}", x);
}
