melody = \relative c' {
  \clef treble
  \key g \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  b'4 d4 b4 g4 a4 d,2 d4 g4 a4 b4 c4 a2 \break

  % Line 2
  a4 b4 a4 b4 cis4 d4 b4 a4 g4 fis2 e2 d2 \break

  % Line 3
  d4 d4 g4 e4 e4 c'4 a4 fis4 \break

  % Line 4
  d4 g4 a4 b4 d8( c8) b2 a2 g2 \bar "|."
}
