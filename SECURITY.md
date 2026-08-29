# security

## scope

nag is a unix command-line tool that wraps arbitrary commands and sends notifications. its attack surface is limited:

- it spawns exactly the command you give it — no shell interpolation, no PATH manipulation
- it writes only to stderr (summary line) and to the terminal title via osc escape sequences
- optional webhook dispatch sends a fixed json payload to a url you provide
- notification icons are written to `/tmp` at runtime from bytes compiled into the binary

## reporting a vulnerability

if you find a security issue, **do not open a public issue**.

email: `security@[your-domain]` — or open a [github private security advisory](https://github.com/programmersd21/nag/security/advisories/new).

include:
- a description of the issue
- steps to reproduce
- potential impact

you will receive a response within 72 hours. once confirmed and fixed, a patch release will be published and the issue will be disclosed publicly.

## known non-issues

- nag inherits the calling user's permissions — it does not escalate privileges
- webhook urls are passed explicitly by the user; nag does not read urls from untrusted sources
- `NAG_WEBHOOK_URL` is an environment variable set by the user; treat it with the same trust as other shell environment
