use anyhow::Result;

pub async fn execute(name: Option<&str>, template: Option<&str>, list: bool) -> Result<()> {
    println!("TODO: Implement init command");
    println!("  name: {:?}", name);
    println!("  template: {:?}", template);
    println!("  list: {}", list);
    Ok(())
}
