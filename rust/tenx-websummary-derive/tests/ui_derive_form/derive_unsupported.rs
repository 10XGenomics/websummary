use tenx_websummary_derive::HtmlForm;

#[derive(HtmlForm)]
enum Foo {
    Bar {},
    Bar2,
}

#[derive(HtmlForm)]
enum Foo2 {
    Bar(u8),
    Bar2,
}

#[derive(HtmlForm)]
struct Boo(u8);

fn main() {}
