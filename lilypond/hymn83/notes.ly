melody = \relative c' {
  \clef treble
  \key f \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  c2 f2 g2 a2 f2 c'2 c2 a1 f2 \break

  % Line 2
  a2 g2 c2 a2 d2 c2 bes2 c1 r2 \break

  % Line 3
  c,2 f2 g2 a2 f2 c'2 c2 a1 f2 \break

  % Line 4
  a2 g2 c2 a2 d2 c2 bes2 c1 r2 \break

  % Line 5
  g2 g2 g2 e2 c2 f2 g2 a1 f2 \break

  % Line 6
  a2 g2 a2 g2 f2 e2 d2 c1 r2 \break

  % Line 7
  c2 f2 g2 a2 f2 c'2 c2 a1 f2 \break

  % Line 8
  c'2 f,2 g2 e2 f2 a2 g2 f1 \bar "|."
}
