use coarsetime::{Clock, Duration, UnixTimeStamp};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::common::VerificationOptions;
use crate::error::*;
use crate::serde_additions;

pub const DEFAULT_TIME_TOLERANCE_SECS: u64 = 900;

/// Type representing the fact that no application-defined claims is necessary.
#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct NoCustomClaims {}

/// A set of JWT claims.
///
/// The `CustomClaims` parameter can be set to `NoCustomClaims` if only standard claims are used,
/// or to a user-defined type that must be `serde`-serializable if custom claims are required.
#[derive(Serialize, Deserialize)]
pub struct JWTClaims<CustomClaims> {
    /// Time the claims were created at
    #[serde(
        rename = "iat",
        default,
        skip_serializing_if = "Option::is_none",
        with = "self::serde_additions::unix_timestamp"
    )]
    pub issued_at: Option<UnixTimeStamp>,

    /// Time the claims expire at
    #[serde(
        rename = "exp",
        default,
        skip_serializing_if = "Option::is_none",
        with = "self::serde_additions::unix_timestamp"
    )]
    pub expires_at: Option<UnixTimeStamp>,

    /// Time the claims will be invalid until
    #[serde(
        rename = "nbf",
        default,
        skip_serializing_if = "Option::is_none",
        with = "self::serde_additions::unix_timestamp"
    )]
    pub invalid_before: Option<UnixTimeStamp>,

    /// Issuer - This can be set to anything application-specific
    #[serde(rename = "iss", default, skip_serializing_if = "Option::is_none")]
    pub issuer: Option<String>,

    /// Subject - This can be set to anything application-specific
    #[serde(rename = "sub", default, skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,

    /// Audience
    #[serde(rename = "aud", default, skip_serializing_if = "Option::is_none")]
    pub audience: Option<String>,

    /// JWT identifier
    ///
    /// That property was originally designed to avoid replay attacks, but keeping
    /// all previously sent JWT token IDs is unrealistic.
    ///
    /// Replay attacks are better addressed by keeping only the timestamp of the last
    /// valid token for a user, and rejecting anything older in future tokens.
    #[serde(rename = "jti", default, skip_serializing_if = "Option::is_none")]
    pub jwt_id: Option<String>,

    /// Custom (application-defined) claims
    #[serde(flatten)]
    pub custom: CustomClaims,
}

impl<CustomClaims> JWTClaims<CustomClaims> {
    pub(crate) fn validate(&self, options: &VerificationOptions) -> Result<(), Error> {
        let now = Clock::now_since_epoch();
        let time_tolerance = options
            .time_tolerance
            .unwrap_or_else(|| Duration::from_secs(DEFAULT_TIME_TOLERANCE_SECS));

        if let Some(reject_before) = options.reject_before {
            ensure!(now <= reject_before, JWTError::OldTokenReused);
        }
        if let Some(time_issued) = self.issued_at {
            ensure!(time_issued <= now + time_tolerance, JWTError::ClockDrift);
            if let Some(max_validity) = options.max_validity {
                ensure!(
                    now <= time_issued || now - time_issued <= max_validity,
                    JWTError::TokenIsTooOld
                );
            }
        }
        if !options.accept_future {
            if let Some(invalid_before) = self.invalid_before {
                ensure!(now >= invalid_before, JWTError::TokenNotValidYet);
            }
        }
        if let Some(expires_at) = self.expires_at {
            ensure!(
                now - time_tolerance <= expires_at,
                JWTError::TokenHasExpired
            );
        }
        if let Some(required_issuer) = &options.required_issuer {
            if let Some(issuer) = &self.issuer {
                ensure!(issuer == required_issuer, JWTError::RequiredIssuerMismatch);
            } else {
                bail!(JWTError::RequiredIssuerMissing);
            }
        }
        if let Some(required_subject) = &options.required_subject {
            if let Some(subject) = &self.subject {
                ensure!(
                    subject == required_subject,
                    JWTError::RequiredSubjectMismatch
                );
            } else {
                bail!(JWTError::RequiredSubjectMissing);
            }
        }
        Ok(())
    }

    /// Set the token as not being valid until `unix_timestamp`
    pub fn invalid_before(mut self, unix_timestamp: UnixTimeStamp) -> Self {
        self.invalid_before = Some(unix_timestamp);
        self
    }

    /// Set the issuer
    pub fn with_issuer(mut self, issuer: impl ToString) -> Self {
        self.issuer = Some(issuer.to_string());
        self
    }

    /// Set the subject
    pub fn with_subject(mut self, subject: impl ToString) -> Self {
        self.issuer = Some(subject.to_string());
        self
    }

    /// Set the audience
    pub fn with_audience(mut self, audience: impl ToString) -> Self {
        self.issuer = Some(audience.to_string());
        self
    }

    /// Set the JWT identifier
    pub fn with_jwt_id(mut self, jwt_id: impl ToString) -> Self {
        self.issuer = Some(jwt_id.to_string());
        self
    }
}

pub struct Claims;

impl Claims {
    /// Create a new set of claims, without custom data, expiring in `valid_for`.
    pub fn create(valid_for: Duration) -> JWTClaims<NoCustomClaims> {
        let now = Some(Clock::now_since_epoch());
        JWTClaims {
            issued_at: now,
            expires_at: Some(now.unwrap() + valid_for),
            invalid_before: now,
            audience: None,
            issuer: None,
            jwt_id: None,
            subject: None,
            custom: NoCustomClaims {},
        }
    }

    /// Create a new set of claims, with custom data, expiring in `valid_for`.
    pub fn with_custom_claims<CustomClaims: Serialize + DeserializeOwned>(
        custom_claims: CustomClaims,
        valid_for: Duration,
    ) -> JWTClaims<CustomClaims> {
        let now = Some(Clock::now_since_epoch());
        JWTClaims {
            issued_at: now,
            expires_at: Some(now.unwrap() + valid_for),
            invalid_before: now,
            audience: None,
            issuer: None,
            jwt_id: None,
            subject: None,
            custom: custom_claims,
        }
    }
}
