melody = \relative c' {
  \clef treble
  \key g \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  e1 g2 e2 d2 e2 g2 fis2 e1 \break

  % Line 2
  g1 a2 a2 b2 g2 a2 a2 b1 \break

  % Line 3
  b1 c2 b2 d2 b2 a2 a2 g1 \break

  % Line 4
  b1 g2 a2 g2 fis2 e2 d2 e1 \bar "|."
}
