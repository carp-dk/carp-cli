# Signing the macOS binaries

The release workflow signs and notarizes the two macOS binaries when the six
secrets below are set. Until they are, it publishes them unsigned and logs a
warning — releases are never blocked on this.

Doing it once takes about half an hour, most of it waiting on Apple.

## 1. The Developer ID Application certificate

This is the certificate for distributing outside the App Store. Neither
*Apple Development* (local only) nor *Apple Distribution* (App Store) will
work. Creating one needs the paid Apple Developer Program and the Account
Holder or Admin role.

**Make a certificate request.** Keychain Access → menu *Certificate Assistant*
→ *Request a Certificate From a Certificate Authority*. Enter your email, leave
CA Email blank, choose *Saved to disk*. This writes a `.certSigningRequest` and
puts the matching private key in your login keychain.

**Create the certificate.** [developer.apple.com][certs] → Certificates → **+**
→ **Developer ID Application** → upload the request → download the `.cer`.

**Install it.** Double-click the `.cer`. It joins the private key already in
your keychain — that pairing is the thing you need, and it only exists on the
machine that made the request.

**Export it.** Keychain Access → **My Certificates** (not *Certificates*, which
holds the public half alone and would produce a `.p12` that cannot sign) →
select *Developer ID Application: …* → right-click → *Export* → `.p12`, with a
password.

Confirm you have a usable identity, and copy the exact string it prints:

```sh
security find-identity -v -p codesigning
#  1) A1B2C3… "Developer ID Application: Your Org (AB12CD34EF)"
```

## 2. The App Store Connect API key

`notarytool` authenticates with this. An app-specific password also works, but
it is tied to one person's Apple ID and their 2FA, which does not survive them
leaving.

App Store Connect → *Users and Access* → *Integrations* → *App Store Connect
API* → *Team Keys* → **+**. The *Developer* role is enough to notarize; raise
it to *App Manager* if submission is refused.

Note the **Key ID** and the **Issuer ID** on that page, and download the `.p8`.
**Apple lets you download it exactly once** — losing it means revoking the key
and making another.

## 3. The secrets

Repository → Settings → Secrets and variables → Actions.

| Secret | Value |
| --- | --- |
| `APPLE_CERTIFICATE_P12` | The `.p12`, base64 encoded |
| `APPLE_CERTIFICATE_PASSWORD` | The password set when exporting it |
| `APPLE_SIGNING_IDENTITY` | `Developer ID Application: Your Org (AB12CD34EF)` |
| `APPLE_API_KEY_P8` | The `.p8`, base64 encoded |
| `APPLE_API_KEY_ID` | Key ID, ten characters |
| `APPLE_API_ISSUER_ID` | Issuer ID, a UUID |

Encode the two files:

```sh
base64 -i Certificates.p12 | tr -d '\n' | pbcopy
base64 -i AuthKey_ABC1234567.p8 | tr -d '\n' | pbcopy
```

## 4. Check it before trusting a release

Signing failures in CI are slow to diagnose, so it is worth signing one binary
locally first:

```sh
cargo build --release --bin carp
codesign --sign "Developer ID Application: Your Org (AB12CD34EF)" \
         --options runtime --timestamp --force target/release/carp
codesign --verify --strict --verbose=2 target/release/carp
```

`--options runtime` (hardened runtime) and `--timestamp` are both required by
notarization, and leaving either out fails it with a message that does not say
which one was missing.

Then notarize the same binary:

```sh
ditto -c -k --keepParent target/release/carp carp.zip
xcrun notarytool submit carp.zip \
  --key AuthKey_ABC1234567.p8 --key-id ABC1234567 --issuer <uuid> --wait
```

If it comes back `Invalid`, the reason is only ever in the log:

```sh
xcrun notarytool log <submission-id> \
  --key AuthKey_ABC1234567.p8 --key-id ABC1234567 --issuer <uuid>
```

## Why this matters more than it looks

It is tempting to treat signing as cosmetic. It is not. Measured on macOS 27
against an unsigned release build:

```
$ xattr -w com.apple.quarantine "0083;0;Safari;" carp.tar.gz
$ tar -xzf carp.tar.gz          # tar propagates the attribute
$ ./carp --version
Killed: 9                        # exit 137, no dialog, no explanation
$ spctl --assess --type execute --verbose=4 ./carp
./carp: rejected
```

An unsigned Rust binary is ad-hoc signed by the linker, which is enough to run
locally but not enough to survive quarantine. Anyone who downloads a release
through a browser and extracts it gets a binary that dies silently. `xattr -d
com.apple.quarantine` is the only way out until the binaries are signed, which
is what the README currently tells people to do.

## The stapling caveat

The binaries are signed and notarized but **not** stapled. `xcrun stapler`
attaches a notarization ticket to `.app` bundles, `.dmg` and `.pkg` only — a
bare command-line binary cannot hold one. Gatekeeper therefore asks Apple's
servers the first time a quarantined copy runs, so that first run needs
network.

Shipping a `.pkg` beside the tarball would remove that caveat: a `.pkg` can be
stapled, and then works offline. It costs a second certificate (*Developer ID
Installer*) and two more release artifacts.

[certs]: https://developer.apple.com/account/resources/certificates/list
