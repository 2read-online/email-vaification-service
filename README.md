# Email Verification Service

A service is written in Rust which gets messages from Redis Streams with
email and verification hash and sends email by using Mailgun service.

## Configuration

The service must be configured by environment variables

| name  | default value  | description                      |
|-------|----------------|----------------------------------|
| REDIS_URL | redis://redis:6379/0 | URL to connect to Redis |
| STREAM_KEY | /auth/login | name of the Redis stream to get messages |
| STREAM_GROUP | email-verification | Group name of consumers. For mor deails see XGRPOUP in Redis documentation |
| VERIFICATION_URL | https://2read.online/auth/verificate | URL for verification request on backend size |
| MAILGUN_DOMAIN | 2read.online | You domain on Mailgun to send emails |
| MAILGUN_API_KEY| | secret API key |
| MAILGUN_FROM | | text for 'FROM' in the emails |
| MAILGUN_SUBJECT | EMail Verification | subject of the emails |
| MAILGUN_TEMPLATE | | Template of the email on Mailgun |

## Redis Message Format

The services expect from the Redis steam messages of the following format

```json
{
  "email": "some@example.com",
  "verification_hash": "<some hash>"
}
```