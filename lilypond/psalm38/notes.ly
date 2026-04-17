melody = \relative c' {
  \clef treble
  \key bes \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  bes'2 a2 g4 a4 bes4 c4 d2 c2 \break

  % Line 2
  r4 d2 c4 bes2 a2 r2 \break

  % Line 3
  c4 c4 d2 bes4 c4 a2 g2 r2 \break

  % Line 4
  d'2 d2 c4 bes4 g4 a4 bes2 a2 \break

  % Line 5
  r4 bes2 a4 g2 f2 r2 \break

  % Line 6
  bes2 d2 c4 bes4 g2 a2 g1 \bar "|."
}
