(comment) @comment

"=" @operator

"," @punctuation.delimiter

; Keys the Playdate SDK documents for launcher card and icon animations.
((key) @keyword
  (#match? @keyword "^(loopCount|frames|introFrames)$"))

((key) @property
  (#not-match? @property "^(loopCount|frames|introFrames)$"))

(frame image: (number) @number)

; xN repeats the frame N times.
(repeat) @operator
