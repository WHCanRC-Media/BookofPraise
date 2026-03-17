melody = \relative c' {
  \clef treble
  \key g \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  g'2 g4 a4 b2 b4 c2 c4 b2 a2 r2 \break

  % Line 2
  b2 c4 b4 a4 g4 fis2 g2 a2 r2 \break

  % Line 3
  d2 c4 b4 a2 fis2 g4 fis4 e2 d2 r2 \break

  % Line 4
  b'2 c4 b4 a4 g4 b2 a2 g1 \bar "|."
}
