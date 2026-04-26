+++
title = "Front Matter Highlight Sample"
draft = false
date = 2026-04-26T10:30:00+09:00

[extra]
theme = "docs"
tags = ["tombi", "frontmatter"]
+++

# Front Matter Sample

Open this file in VS Code with the Tombi extension enabled.

The `+++` block above should highlight as TOML inside Markdown front matter.

```toml
title = "Complex TOML Sample"
enabled = true
count = 42
hex = 0x2A
oct = 0o52
bin = 0b101010
ratio = 3.14
infinite = inf
not_a_number = nan
published = 2026-04-26T10:30:00+09:00
local_datetime = 2026-04-26T10:30:00
local_date = 2026-04-26
local_time = 10:30:00
tags = ["tombi", "markdown", "toml"]
numbers = [1, 2, 3, 4]
matrix = [[1, 2], [3, 4], [5, 6]]
schedule = [
  2026-04-26T10:30:00+09:00,
  2026-04-27T11:45:00+09:00,
]
inline = { nested = 1, label = "ok", enabled = true } # trailing comment
inline_nested = {
  level1 = { level2 = { answer = 42, active = false } },
  values = [1, 2, { nested = [3, 4, { deep = "value" }] }],
}

[server]
host = "127.0.0.1"
ports = [8080, 8081]
maintenance = { start = 2026-04-26, end = 2026-04-27 }

[server.metadata]
owner = "tombi"
labels = ["syntax", "highlight"]

[[users]]
name = "alice"
joined = 2026-04-26
roles = ["admin", "editor"]
profile = { active = true, score = 10, prefs = { theme = "light", alerts = [true, false] } }

[[users]]
name = "bob"
joined = 2026-04-27
roles = ["viewer"]
profile = { active = false, score = 7, prefs = { theme = "dark", alerts = [false] } }
```
