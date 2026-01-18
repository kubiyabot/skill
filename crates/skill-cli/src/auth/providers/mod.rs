//! Authentication provider implementations.

pub mod oauth2;
pub mod api_key;
pub mod aws;

// These re-exports are part of the public API
#[allow(unused_imports)]
pub use oauth2::OAuth2Provider;
#[allow(unused_imports)]
pub use api_key::ApiKeyProvider;
#[allow(unused_imports)]
pub use aws::AwsProvider;
