# PiHole-domain-aggregator
It agregates lists from various sources to some huge lists.
Those lists can represent domains of a certin topic that shall be blocked by the [PiHole](https://pi-hole.net/).

## Data processing
It breaks the fetched lists down into lines and those lines to atomic entries.
The entries are converted into punicode if needed.
All characters that aren't alphanumeric or a dash/dot are cut off.
The remaining entries are validated as in [rfc1035 section 2.3.1.](https://datatracker.ietf.org/doc/html/rfc1035#section-2.3.1) defined syntax.
The valid domains are stored both with and without the prefix `www`. If a custom prefix or suffix has been configured, this will also be added. This does not impact the result of the domain.

## FAQ

Q: Did anyone really ask you these questions?</br>
A: No.

Q: Why do you add/remove the subdomain `www.`? That's stupid.</br>
A: They're technically diffrent and i know that they could have different purposes. But i want to get sure that i can't go to websites that i want to be blocked. Mostly the point to the same thing and that's why i block a bit more to get sure i cant't reach that domain by accident. If i really need to reach that specific website i can still whitelist the domain.