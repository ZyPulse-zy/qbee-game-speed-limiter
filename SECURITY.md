# Security

`qbee_game_speed_limiter.json` stores Web UI credentials in plain text on the local machine.

Recommendations:

- Use qBittorrent's localhost authentication bypass when possible.
- Do not commit your personal config file.
- Do not share logs if they contain private paths or usernames.
- Rotate exposed PeerBanHelper tokens or qBittorrent passwords.

For vulnerability reports, open a private security advisory on GitHub if available, or create an issue without including secrets.
