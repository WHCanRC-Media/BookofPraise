melody = \relative c' {
  \clef treble
  \key g \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  e4 b'4 fis4 g4 a4 g4 fis4 e4 \break

  % Line 2
  g4 a4 b4 c4 a4 b2 \break

  % Line 3
  b4 d4 g,4 fis4 g4 a4 b4 c4 \break

  % Line 4
  a4 b4 e,8( fis8) g4 fis4 e2 \bar "|."
}
