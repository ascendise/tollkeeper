/// Information about the source trying to access the resource, read by [gates](super::Gate) to match
/// descriptions
pub trait Suspect {
    fn client_ip(&self) -> &str;
    fn user_agent(&self) -> &str;
    fn target_host(&self) -> &str;
    fn target_path(&self) -> &str;
}
