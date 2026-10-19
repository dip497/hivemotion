# Music and sound sources

A track that gets a copyright claim after launch can mute or block the film on the platform that
matters most. Choose for the licence first, the sound second.

| Source | Licence | Risk |
|---|---|---|
| CC0 libraries (for example Kenney for sound effects) | Public domain dedication | None; still record the source |
| YouTube Audio Library | Free for YouTube; some tracks need attribution | Claim-safe on YouTube; can't be redistributed in a repo |
| Free stock music sites (for example Pixabay) | Free, no attribution | A track can be registered with Content ID later, after you have used it |
| Paid libraries (Epidemic, Soundstripe, ...) | Subscription | Claims are cleared per channel; check the terms |
| AI music services | Varies by plan and provider | Check commercial rights and training provenance |
| Your own or commissioned music | Yours | None; keep the agreement |

## Record every asset

For each track, effect, font, image and 3D model, keep one line in the film folder (for example
`credits.yaml`): file, source URL, date fetched, licence and its text or link, attribution
required (yes or no), and a SHA-256 of the file. The credits in
[deliver-and-publish](../../deliver-and-publish/SKILL.md) are written from it.

Keep tracks out of git if their licence forbids redistribution; the film folder lists them and
anyone can fetch them again.
