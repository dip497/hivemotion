# Data hygiene before capture

A launch film is public forever. Anything on screen in any frame can be paused and read.

## Before capturing

- **Isolate the stage.** A fresh profile, a temporary home folder, no logged-in accounts, no
  real config files. Do not capture in the machine's everyday session.
- **Seed demo data written for the film.** Project, file, user and team names that are plausible
  and fictional. No real customers, colleagues, emails, avatars or company names.
- **Scrub everything the UI shows by default:** window titles, tabs and tiles left open from
  real work, file paths (they often contain a user name), project and branch names, host names
  and IP addresses, recent-file lists, terminal prompts, notifications, clipboard contents.
- **No secrets anywhere near the stage:** no tokens in environment variables, shell history or
  config the app might print.
- **Hide anything a viewer could misread as a claim:** counters, prices, version numbers that
  will change.

## After capturing

- Look at every take's stills at beats and through every pan: zoomed-in shots reveal text a
  wide shot hides.
- `hivemotion check` makes a contact sheet per aspect; read it for names, paths and logos.
- If anything private shows, capture again. Do not blur it: a blur still says "something was here".
