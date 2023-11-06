use tenx_websummary_derive::HtmlTemplate;

#[derive(HtmlTemplate)]
enum Foo {
    Bar,
}

#[derive(HtmlTemplate)]
struct Boo(u8);

fn main() {}
