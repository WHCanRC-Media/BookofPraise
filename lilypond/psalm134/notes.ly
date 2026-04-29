melody = \relative c' {
  \clef treble
  \key g \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  g'2 g4 fis4 e4 d4 g2 a2 b2 r2 \break

  % Line 2
  b2 b4 b4 a4 g4 c2 b2 a2 r2 \break

  % Line 3
  g2 a4 b4 a4 g4 e2 fis2 g2 r2 \break

  % Line 4
  d'2 b2 g2 a4 c4 b2 a2 g1 \bar "|."
}
