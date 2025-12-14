#[tarpc::service]
pub(crate) trait ChannelService {
    async fn send_data(data: String) -> String;
}
