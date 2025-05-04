#[derive(Default, Clone, Component)]
pub struct State {}

pub async fn start_azalea(
    address: &str,
) -> Result<()> {
    let client = AzaleaClient::new(account, server_address, server_port).await?;
    let state = State::default();
    let mut client = client.connect(state).await?;

    // Example of sending a message
    client.send_message("Hello from Azalea!").await?;

    Ok(())
}