melody = \relative c' {
  \clef treble
  \key g \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  g'2 d2 e4 fis4 g4 b2 b4 c4 a4 g2 r2 \break

  % Line 2
  d'2 d2 b4 c4 d4 b2 d4 c4 c4 b2 r2 \break

  % Line 3
  d2 c2 b4 a4 g4 e2 fis4 g4 e4 d2 r2 \break

  % Line 4
  d2 g2 a4 b4 c2 b4 g2 a4 b4 c4 a2 g1 \bar "|."
}
