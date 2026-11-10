# Vault format (version 1)

A GitVault vault is a plain git repository. Everything sensitive is encrypted; the repository
contains only ciphertext and a header with key-derivation settings.

```
<vault repo>/
├─ .gitattributes          "* binary"  (git never text-merges encrypted files)
├─ .gitvault/
│  └─ vault.json           plaintext header (no secrets)
└─ secrets/
   └─ <secret-id>.enc      one encrypted file per secret (id = UUID)
```

The GitHub repository is created **private** and tagged with the topic `gitvault-vault`, which is
how the app finds your vaults.

## `.gitvault/vault.json`

```json
{
  "format": 1,
  "vault_id": "4ae5ed41-…",
  "name": "Personal vault",
  "description": "",
  "created_at": 1790000000,
  "kdf": { "memory_kib": 65536, "iterations": 3, "parallelism": 1 },
  "password_salt": "<base64, 16 bytes>",
  "recovery_salt": "<base64, 16 bytes>",
  "wrapped_by_password": { "nonce": "<base64>", "ciphertext": "<base64>" },
  "wrapped_by_recovery": { "nonce": "<base64>", "ciphertext": "<base64>" }
}
```

The vault name and description are visible to anyone who can read the repository.

## Key hierarchy

```
master password ──Argon2id(password_salt, kdf)──► KEK ──AES-256-GCM unwrap──► vault key
recovery key    ──HKDF-SHA256(recovery_salt)────► KEK ──AES-256-GCM unwrap──► vault key
vault key       ──AES-256-GCM, fresh 96-bit nonce per write──► secret files
```

- The **vault key** is 32 random bytes, generated once per vault.
- Changing the master password re-wraps the vault key; no secret is re-encrypted.
- The **recovery key** is 160 random bits shown once as 8 groups of 4 base32 characters.
- Wrapping uses associated data `gitvault:wrap:v1`.

## `secrets/<id>.enc`

```json
{ "v": 1, "nonce": "<base64>", "ciphertext": "<base64>" }
```

The ciphertext is the JSON of the whole secret (name, kind, environment, tags, value, notes,
expiry, favorite, timestamps, version). Associated data is `gitvault:v1:<vault-id>:<secret-id>`, so
a file copied to another vault or renamed fails authentication.

Because names and notes are inside the ciphertext, the repository reveals only how many
secrets exist and when they changed.

## Commits and history

- One commit per change, message contains **ids only** (`secret: update 7f3c9a10`), never names.
- A secret's history is `git log -- secrets/<id>.enc`; restoring writes an old version back as a new
  version, including for deleted secrets.
- `sync` = fetch, merge, push. Because every secret is its own binary file, two devices only
  conflict when they edit the **same secret**; the app then keeps both sides and asks which to keep.
