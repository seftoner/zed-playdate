(comment) @comment

"=" @operator

; Keys the Playdate SDK documents (pdc adds pdxversion and buildtime).
((key) @keyword
  (#match? @keyword "^(name|author|description|bundleID|version|buildNumber|imagePath|launchSoundPath|contentWarning|contentWarning2|menuTitleFont|menuMessageFont|pdxversion|buildtime)$"))

((key) @property
  (#not-match? @property "^(name|author|description|bundleID|version|buildNumber|imagePath|launchSoundPath|contentWarning|contentWarning2|menuTitleFont|menuMessageFont|pdxversion|buildtime)$"))

; Integers.
(entry
  key: (key) @_key
  value: (value) @number
  (#match? @_key "^(buildNumber|pdxversion|buildtime)$"))

; Identifiers and paths.
(entry
  key: (key) @_key
  value: (value) @string.special
  (#match? @_key "^(bundleID|imagePath|launchSoundPath|menuTitleFont|menuMessageFont)$"))

; Free text.
(entry
  key: (key) @_key
  value: (value) @string
  (#not-match? @_key "^(buildNumber|pdxversion|buildtime|bundleID|imagePath|launchSoundPath|menuTitleFont|menuMessageFont)$"))
