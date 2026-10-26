# Sign in with ChatGPT test fixtures

`test-signing-key.der` is a throwaway 2048-bit RSA private key (PKCS#8 DER), made for these tests with
`openssl genpkey -algorithm RSA -pkeyopt rsa_keygen_bits:2048 -outform DER`. `fake-openai` signs its test ID tokens
with it and publishes the public half in its JWKS. It protects nothing and is trusted by nothing outside the tests:
Emmet checks real ID tokens only against OpenAI's published keys.
