use mcli::MyCli;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cli = MyCli::new("localhost".to_owned(), 6379)?;
    let _ = cli.ping()?;
    let _ = cli.ping_str("hello, world".to_owned())?;

    cli.set("a".to_owned(), "b").expect("fuck set");
    if let Some(v) = cli.get("a").expect("get b") {
        println!("{}", String::from_utf8(v)?);
    }
    Ok(())
}
