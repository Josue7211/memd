use super::*;

pub(crate) async fn run_awareness_command(args: &AwarenessArgs) -> anyhow::Result<()> {
    let response = read_project_awareness(args).await?;
    if args.summary {
        println!("{}", render_project_awareness_summary(&response));
    } else {
        print_json(&response)?;
    }
    Ok(())
}
