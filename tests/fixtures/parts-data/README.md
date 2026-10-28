# Parts data pack test keys

Throwaway Ed25519 keys for the parts data pack's signature check (plan section 12, session 3B), made for these tests
with `openssl genpkey -algorithm ed25519 -outform DER`:

- `test-signing-key.der` (PKCS#8 DER) and `test-signing-key.pub` (the raw 32-byte public key). Until session 6A makes
  the real key, `emmet_core::parts_data::TRUSTED_KEY` is this public key, and the tests sign packs with the private one.
- `other-signing-key.der` and `.pub`: a second key, for the test that refuses a pack signed by a key Emmet doesn't
  trust.

They protect nothing. Builds ship only the bundled pack, which is checked by its sha256, and nothing downloads packs
until 6A replaces `TRUSTED_KEY` with the real public key, whose private half never enters the repo.
