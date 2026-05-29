//! Shared dev constants.

/// **Dev only.** The Ed25519 **public** key the bundled dev keypair verifies
/// against. Lives here — the one crate both apps depend on — so `auth` (which also
/// holds the matching private key) and every resource server use the *same* default
/// without drifting; a mismatch would silently reject every token in dev.
/// Production supplies real keys via env and never reads this.
pub const DEV_PUBLIC_KEY_PEM: &str = "-----BEGIN PUBLIC KEY-----\nMCowBQYDK2VwAyEAHfPOjd2Y3m1BLM5nBJBMZFAlfWt69WL1NY8XyYeGfeo=\n-----END PUBLIC KEY-----\n";
