melody = \relative c' {
  \clef treble
  \key f \major
  \cadenzaOn
  \omit Staff.TimeSignature

  % Line 1
  g'2 g4 g4 d2 d2 f4 g4 a4 g4 f2 g2 r2 \break

  % Line 2
  bes2 a2 g2 f2 d2 f4 g4 a4 g4 f2 g2 r2 \break

  % Line 3
  d'2 d4 d4 g,2 c2 c4 bes4 a4 g4 f2 d4 \break

  % Line 4
  g4 g4 f4 g2 f2 g4 f4 g4 a4 bes2 a2 r2 \break

  % Line 5
  bes2 a4 g4 f2 d2 f4 f4 g4 f4 e2 d2 r2 \break

  % Line 6
  d'2 c4 bes4 a2 f2 g4 bes4 c4 bes4 a2 g1 \bar "|."
}
