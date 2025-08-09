pep508_rs を利用していますが、自前実装にしましょう。


1. package 名が打たれている場合、次を返す
  - extras の補完候補
    - `[${1:extra_name}]`
  - バージョン比較記号を補完候補
    ("==", "Exact version match"),
    (">=", "Greater than or equal to"),
    ("<=", "Less than or equal to"),
    (">", "Greater than"),
    ("<", "Less than"),
    ("~=", "Compatible release"),
    ("!=", "Not equal to"),
  - 依存関係の補完候補
    - `; ${1:condition}`

2. extras の `[]` の中にカーソルがある場合、次を返す
  - extras の補完候補（https://pypi.org/pypi から取得する）
    - `${1:extra_name}`

3. extras の `[]` の外にカーソルがある場合、次を返す
  - バージョン比較バージョンを補完候補
    ("==", "Exact version match"),
    (">=", "Greater than or equal to"),
    ("<=", "Less than or equal to"),
    (">", "Greater than"),
    ("<", "Less than"),
    ("~=", "Compatible release"),
    ("!=", "Not equal to"),
  - 依存関係の補完候補
    - `; ${1:condition}`

4. バージョン比較記号の後にカーソルがある場合、次を返す
  - バージョン番号の補完候補（https://pypi.org/pypi から取得する）
    - `${1:version}`
