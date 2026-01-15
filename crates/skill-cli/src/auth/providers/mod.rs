//! Authentication provider implementations.

pub mod oauth2;
pub mod api_key;
pub mod aws;

pub use oauth2::OAuth2Provider;
pub use api_key::ApiKeyProvider;
pub use aws::AwsProvider;
