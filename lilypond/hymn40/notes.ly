melody = \relative c' {
  \clef treble
  \key g \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  g'4 b4 a4 g4 d4 e4 fis4 g4 \break

  % Line 2
  b4 b4 d4 c4 b4 a4 g4 a4 \break

  % Line 3
  b4 g4 g4 c4 b4 a2 g4 \break

  % Line 4
  d4 e4 fis4 g4 fis4 e4 e4 d4 \break

  % Line 5
  e4 fis4 g4 a4 g4 fis4 fis4 e4 \break

  % Line 6
  b'4 d4 g,4 c4 b4 a2 g4 \bar "|."
}
